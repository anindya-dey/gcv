//! Resolve and load resume content from git hosts, URLs, local files, or stdin,
//! with an on-disk cache so repeat runs are instant and offline-friendly.

use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::parse::Format;

const CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default, Clone)]
pub struct FetchOpts {
    pub file: Option<String>,
    pub branch: Option<String>,
    pub refresh: bool,
    pub offline: bool,
    pub gitlab: bool,
}

pub struct Loaded {
    pub content: String,
    pub format: Format,
    pub label: String,
}

const DEFAULT_FILES: &[&str] = &[
    "resume.md",
    "resume.json",
    "resume.yaml",
    "resume.yml",
    "resume.toml",
    "cv.md",
    "cv.json",
    "cv.yaml",
    "cv.yml",
    "cv.toml",
    "resume/index.md",
    "resume/index.json",
];

pub fn load(source: &str, opts: &FetchOpts) -> Result<Loaded, String> {
    let source = source.trim();

    if source == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("failed to read stdin: {e}"))?;
        let format = Format::sniff(&buf);
        return Ok(Loaded {
            content: buf,
            format,
            label: "stdin".into(),
        });
    }

    let path = PathBuf::from(source);
    if path.exists() && path.is_file() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let format = Format::from_path(source).unwrap_or_else(|| Format::sniff(&content));
        return Ok(Loaded {
            content,
            format,
            label: path.display().to_string(),
        });
    }

    if source.starts_with("http://") || source.starts_with("https://") {
        return load_url(source, opts);
    }

    load_shorthand(source, opts)
}

fn load_url(source: &str, opts: &FetchOpts) -> Result<Loaded, String> {
    // A bare repo page like github.com/owner/repo -> resolve to a raw file.
    if let Some((host, owner, repo)) = parse_repo_page(source) {
        return resolve_repo(&host, &owner, &repo, "", opts);
    }
    let (content, _cached) =
        fetch(source, opts)?.ok_or_else(|| format!("not found (404): {source}"))?;
    let format = Format::from_path(source).unwrap_or_else(|| Format::sniff(&content));
    Ok(Loaded {
        content,
        format,
        label: source.to_string(),
    })
}

fn parse_repo_page(url: &str) -> Option<(String, String, String)> {
    let rest = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let mut parts = rest.split('/');
    let host = parts.next()?.to_string();
    if !host.contains('.') {
        return None;
    }
    let owner = parts.next()?;
    let repo = parts.next()?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    // Only treat as a repo page if there is no deeper file path.
    let remainder = parts.next();
    if remainder.is_some() {
        return None;
    }
    Some((host, owner.to_string(), repo.to_string()))
}

fn load_shorthand(source: &str, opts: &FetchOpts) -> Result<Loaded, String> {
    let segments: Vec<&str> = source.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Err("empty source".into());
    }

    let hosts = ["github.com", "gitlab.com", "codeberg.org", "bitbucket.org"];
    let (host, rest): (String, Vec<&str>) = if hosts.contains(&segments[0]) {
        (segments[0].to_string(), segments[1..].to_vec())
    } else {
        let default_host = if opts.gitlab {
            "gitlab.com"
        } else {
            "github.com"
        };
        (default_host.to_string(), segments.clone())
    };

    if rest.is_empty() {
        return Err("expected owner/repo".into());
    }

    let owner = rest[0];
    let repo = if rest.len() >= 2 { rest[1] } else { rest[0] };
    let path = if rest.len() >= 3 {
        rest[2..].join("/")
    } else {
        String::new()
    };

    resolve_repo(&host, owner, repo, &path, opts)
}

fn resolve_repo(
    host: &str,
    owner: &str,
    repo: &str,
    path: &str,
    opts: &FetchOpts,
) -> Result<Loaded, String> {
    let branches: Vec<String> = match &opts.branch {
        Some(b) => vec![b.clone()],
        None => vec!["main".to_string(), "master".to_string()],
    };

    let files: Vec<String> = if !path.is_empty() {
        vec![path.to_string()]
    } else if let Some(f) = &opts.file {
        vec![f.clone()]
    } else {
        DEFAULT_FILES.iter().map(|s| s.to_string()).collect()
    };

    let mut last_err: Option<String> = None;
    for file in &files {
        for branch in &branches {
            let url = raw_url(host, owner, repo, branch, file);
            match fetch(&url, opts) {
                Ok(Some((content, _cached))) => {
                    let format = Format::from_path(file).unwrap_or_else(|| Format::sniff(&content));
                    return Ok(Loaded {
                        content,
                        format,
                        label: format!("{host}/{owner}/{repo}/{file}"),
                    });
                }
                Ok(None) => {}
                Err(e) => last_err = Some(e),
            }
        }
    }
    if let Some(e) = last_err {
        return Err(format!(
            "could not fetch a resume from {host}/{owner}/{repo}: {e}"
        ));
    }
    Err(format!(
        "no resume file found in {host}/{owner}/{repo} (looked for {})",
        files.join(", ")
    ))
}

fn raw_url(host: &str, owner: &str, repo: &str, branch: &str, path: &str) -> String {
    match host {
        "gitlab.com" => format!("https://gitlab.com/{owner}/{repo}/-/raw/{branch}/{path}"),
        "codeberg.org" => format!("https://codeberg.org/{owner}/{repo}/raw/branch/{branch}/{path}"),
        "bitbucket.org" => format!("https://bitbucket.org/{owner}/{repo}/raw/{branch}/{path}"),
        _ => format!("https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}"),
    }
}

/// Returns Ok(Some((content, from_cache))) on success, Ok(None) on 404.
fn fetch(url: &str, opts: &FetchOpts) -> Result<Option<(String, bool)>, String> {
    if !opts.refresh {
        if let Some(c) = cache_get(url) {
            return Ok(Some((c, true)));
        }
    }
    if opts.offline {
        return Err(format!("offline: no cached copy of {url}"));
    }

    let resp = ureq::get(url)
        .set("User-Agent", concat!("gcv/", env!("CARGO_PKG_VERSION")))
        .set("Accept", "text/plain, application/json, */*")
        .call();

    match resp {
        Ok(r) => {
            let body = r
                .into_string()
                .map_err(|e| format!("failed to read response: {e}"))?;
            cache_put(url, &body);
            Ok(Some((body, false)))
        }
        Err(ureq::Error::Status(404, _)) | Err(ureq::Error::Status(403, _)) => Ok(None),
        Err(ureq::Error::Status(code, _)) => Err(format!("HTTP {code} for {url}")),
        Err(e) => Err(format!("network error for {url}: {e}")),
    }
}

fn cache_dir() -> Option<PathBuf> {
    let base = dirs::cache_dir()?;
    let dir = base.join("gcv");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn cache_path(url: &str) -> Option<PathBuf> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    let name = format!("{:016x}.cache", h.finish());
    Some(cache_dir()?.join(name))
}

fn cache_get(url: &str) -> Option<String> {
    let path = cache_path(url)?;
    let meta = std::fs::metadata(&path).ok()?;
    let modified = meta.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;
    if age > CACHE_TTL {
        return None;
    }
    std::fs::read_to_string(&path).ok()
}

fn cache_put(url: &str, content: &str) {
    if let Some(path) = cache_path(url) {
        let _ = std::fs::write(path, content);
    }
}
