//! Markdown parser.
//!
//! The same `.md` file is meant to look good on GitHub *and* render well in
//! the terminal, so the body follows a light, readable convention:
//!
//! ```text
//! # Name
//! **Label** · Location
//! [email](mailto:) · [site](https://) · ...
//!
//! ## Experience
//! ### Role · Company
//! _Feb 2024 – Present · Remote_
//! Description paragraph.
//! - highlight
//! `Tech` `Chips`
//! ```
//!
//! Anything that doesn't match a known section is preserved as an extra
//! free-form section and rendered as-is.

use crate::inline;
use crate::model::*;

#[derive(serde::Deserialize, Default)]
struct Frontmatter {
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    basics: Basics,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sec {
    None,
    Summary,
    Capabilities,
    Experience,
    Projects,
    Skills,
    Education,
    Certifications,
    Languages,
    Interests,
    Awards,
    Extra,
}

const SEPARATORS: [char; 3] = ['·', '•', '|'];

fn split_sep(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if SEPARATORS.contains(&c) {
            parts.push(cur.trim().to_string());
            cur = String::new();
        } else {
            cur.push(c);
        }
    }
    parts.push(cur.trim().to_string());
    parts.into_iter().filter(|p| !p.is_empty()).collect()
}

fn section_of(title: &str) -> Sec {
    match normalize_section(title).as_str() {
        "summary" => Sec::Summary,
        "capabilities" => Sec::Capabilities,
        "experience" => Sec::Experience,
        "projects" => Sec::Projects,
        "skills" => Sec::Skills,
        "education" => Sec::Education,
        "certifications" => Sec::Certifications,
        "languages" => Sec::Languages,
        "interests" => Sec::Interests,
        "awards" => Sec::Awards,
        _ => Sec::Extra,
    }
}

fn code_spans(line: &str) -> Vec<String> {
    let spans = inline::parse(line);
    if spans.is_empty() || !spans.iter().any(|s| s.code) {
        return Vec::new();
    }
    if spans.iter().any(|s| !s.code && !s.text.trim().is_empty()) {
        return Vec::new();
    }
    spans
        .into_iter()
        .filter(|s| s.code)
        .map(|s| s.text)
        .collect()
}

fn is_italic_line(line: &str) -> bool {
    let t = line.trim();
    (t.starts_with('_') && t.ends_with('_') && t.len() > 2)
        || (t.starts_with('*') && t.ends_with('*') && !t.starts_with("**") && t.len() > 2)
}

pub fn parse(text: &str) -> Result<Resume, String> {
    let text = text.replace("\r\n", "\n");
    let (front, body) = split_frontmatter(&text);

    let mut resume = Resume::default();
    if let Some(f) = front {
        if let Ok(fm) = serde_yaml::from_str::<Frontmatter>(&f) {
            resume.meta = fm.meta;
            resume.basics = fm.basics;
        } else if let Ok(meta) = serde_yaml::from_str::<Meta>(&f) {
            resume.meta = meta;
        }
    }

    parse_body(body, &mut resume);
    Ok(resume)
}

fn split_frontmatter(text: &str) -> (Option<String>, &str) {
    let trimmed = text.trim_start_matches('\u{feff}');
    if !trimmed.starts_with("---") {
        return (None, trimmed);
    }
    let after = &trimmed[3..];
    let after = after.strip_prefix('\n').unwrap_or(after);
    if let Some(idx) = find_frontmatter_end(after) {
        let front = after[..idx].to_string();
        let rest = &after[idx..];
        let rest = rest.trim_start_matches('\n');
        let rest = rest
            .strip_prefix("---")
            .unwrap_or(rest)
            .trim_start_matches('\n');
        return (Some(front), rest);
    }
    (None, trimmed)
}

fn find_frontmatter_end(s: &str) -> Option<usize> {
    let mut idx = 0;
    for line in s.split_inclusive('\n') {
        if line.trim_end() == "---" {
            return Some(idx);
        }
        idx += line.len();
    }
    None
}

fn parse_body(body: &str, r: &mut Resume) {
    let mut section = Sec::None;
    let mut header_phase = true;
    let mut wb: Option<Work> = None;
    let mut eb: Option<Education> = None;
    let mut pb: Option<Project> = None;
    let mut extra_title = String::new();
    let mut extra_lines: Vec<String> = Vec::new();

    for raw in body.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed == "---" || trimmed == "***" {
            continue;
        }

        if let Some(title) = trimmed.strip_prefix("### ") {
            flush_extra(r, &mut extra_title, &mut extra_lines);
            header_phase = false;
            match section {
                Sec::Experience => {
                    flush_work(r, &mut wb);
                    let parts = split_sep(title);
                    let mut w = Work::default();
                    if parts.len() >= 2 {
                        w.position = Some(parts[0].clone());
                        w.name = parts[1..].join(" · ");
                    } else {
                        w.name = inline::to_plain(title);
                    }
                    wb = Some(w);
                }
                Sec::Projects => {
                    flush_project(r, &mut pb);
                    pb = Some(Project {
                        name: inline::to_plain(title),
                        ..Default::default()
                    });
                }
                Sec::Education => {
                    flush_edu(r, &mut eb);
                    let parts = split_sep(title);
                    let mut e = Education::default();
                    if parts.len() >= 2 {
                        e.study_type = Some(parts[0].clone());
                        e.area = Some(parts[1..].join(" · "));
                    } else {
                        e.institution = inline::to_plain(title);
                    }
                    eb = Some(e);
                }
                _ => {
                    extra_title = inline::to_plain(title);
                }
            }
            continue;
        }

        if let Some(title) = trimmed.strip_prefix("## ") {
            flush_work(r, &mut wb);
            flush_edu(r, &mut eb);
            flush_project(r, &mut pb);
            flush_extra(r, &mut extra_title, &mut extra_lines);
            header_phase = false;
            section = section_of(title);
            if section == Sec::Extra {
                extra_title = inline::to_plain(title);
            }
            continue;
        }

        if let Some(title) = trimmed.strip_prefix("# ") {
            if r.basics.name.is_none() {
                r.basics.name = Some(inline::to_plain(title));
            }
            continue;
        }

        if trimmed.starts_with("> ") {
            let text = inline::to_plain(trimmed.trim_start_matches("> "));
            let f = r.meta.footer.get_or_insert_with(String::new);
            if !f.is_empty() {
                f.push(' ');
            }
            f.push_str(&text);
            continue;
        }

        if header_phase {
            parse_header_line(trimmed, r);
            continue;
        }

        // inline-code-only line -> tech keywords for current entry
        let codes = code_spans(trimmed);
        if !codes.is_empty() {
            match section {
                Sec::Experience => {
                    if let Some(w) = wb.as_mut() {
                        w.keywords.extend(codes);
                    }
                }
                Sec::Projects => {
                    if let Some(p) = pb.as_mut() {
                        p.keywords.extend(codes);
                    }
                }
                _ => {}
            }
            continue;
        }

        if is_italic_line(trimmed) {
            let inner = trimmed.trim_matches(|c| c == '_' || c == '*');
            parse_meta_line(inner, section, &mut wb, &mut eb, &mut pb);
            continue;
        }

        if let Some(item) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            parse_bullet(item, section, r, &mut wb, &mut pb);
            continue;
        }

        // paragraph
        match section {
            Sec::Summary => {
                let s = r.basics.summary.get_or_insert_with(String::new);
                if !s.is_empty() {
                    s.push(' ');
                }
                s.push_str(&inline::to_plain(trimmed));
            }
            Sec::Experience => {
                if let Some(w) = wb.as_mut() {
                    if w.description.is_none() {
                        w.description = Some(inline::to_plain(trimmed));
                    }
                }
            }
            Sec::Projects => {
                if let Some(p) = pb.as_mut() {
                    if p.description.is_none() {
                        p.description = Some(inline::to_plain(trimmed));
                    }
                }
            }
            Sec::Extra => extra_lines.push(trimmed.to_string()),
            _ => {}
        }
    }

    flush_work(r, &mut wb);
    flush_edu(r, &mut eb);
    flush_project(r, &mut pb);
    flush_extra(r, &mut extra_title, &mut extra_lines);
}

fn parse_header_line(line: &str, r: &mut Resume) {
    if r.basics.label.is_none() {
        if let Some(b) = inline::first_bold(line) {
            r.basics.label = Some(b);
            let plain = inline::to_plain(line);
            if let Some(idx) = plain.find('·') {
                let loc = plain[idx + '·'.len_utf8()..].trim();
                if !loc.is_empty() {
                    r.basics
                        .location
                        .get_or_insert_with(Location::default)
                        .address = Some(loc.to_string());
                }
            }
        }
    }
    for (text, url) in inline::extract_links(line) {
        if url.starts_with("mailto:") {
            r.basics.email = Some(url.trim_start_matches("mailto:").to_string());
        } else {
            let network = network_of(&url).unwrap_or_else(|| text.clone());
            if r.basics.url.is_none() && network_is_primary(&url) {
                r.basics.url = Some(url.clone());
            }
            r.basics.profiles.push(Profile {
                network: Some(network),
                username: Some(text),
                url: Some(url),
            });
        }
    }
}

fn network_is_primary(url: &str) -> bool {
    let u = url.to_lowercase();
    u.contains("anindyadey.com") && !u.contains("blog.")
}

fn network_of(url: &str) -> Option<String> {
    let u = url.to_lowercase();
    let name = if u.contains("github.com") {
        "GitHub"
    } else if u.contains("gitlab.com") {
        "GitLab"
    } else if u.contains("linkedin.com") {
        "LinkedIn"
    } else if u.contains("twitter.com") || u.contains("x.com") {
        "X"
    } else if u.contains("stackoverflow.com") {
        "Stack Overflow"
    } else if u.contains("youtube.com") {
        "YouTube"
    } else if u.contains("instagram.com") {
        "Instagram"
    } else if u.contains("mastodon") {
        "Mastodon"
    } else if u.contains("blog.") {
        "Blog"
    } else {
        return None;
    };
    Some(name.to_string())
}

fn parse_meta_line(
    inner: &str,
    section: Sec,
    wb: &mut Option<Work>,
    eb: &mut Option<Education>,
    pb: &mut Option<Project>,
) {
    let parts = split_sep(inner);
    match section {
        Sec::Experience => {
            if let Some(w) = wb.as_mut() {
                if let Some(dates) = parts.first() {
                    let (s, e) = split_dates(dates);
                    w.start_date = Some(s);
                    w.end_date = e;
                }
                if let Some(loc) = parts.get(1) {
                    w.location = Some(loc.clone());
                }
            }
        }
        Sec::Education => {
            if let Some(e) = eb.as_mut() {
                if let Some(inst) = parts.first() {
                    e.institution = inst.clone();
                }
                if let Some(dates) = parts.get(1) {
                    let (s, en) = split_dates(dates);
                    e.start_date = Some(s);
                    e.end_date = en;
                }
            }
        }
        Sec::Projects => {
            if let Some(p) = pb.as_mut() {
                let mut roles = Vec::new();
                for (i, part) in parts.iter().enumerate() {
                    if part.contains("](") {
                        continue;
                    }
                    if i == 0 && looks_like_date(part) {
                        let (s, e) = split_dates(part);
                        p.start_date = Some(s);
                        p.end_date = e;
                    } else if !part.is_empty() {
                        roles.push(part.clone());
                    }
                }
                if !roles.is_empty() {
                    p.roles = roles;
                }
            }
        }
        _ => {}
    }
    // links on meta lines (projects)
    if section == Sec::Projects {
        if let Some(p) = pb.as_mut() {
            for (text, url) in inline::extract_links(inner) {
                if p.url.is_none() {
                    p.url = Some(url.clone());
                }
                p.links.push(Link { label: text, url });
            }
        }
    }
}

fn split_dates(s: &str) -> (String, Option<String>) {
    for dash in ['–', '—'] {
        if let Some(idx) = s.find(dash) {
            let start = s[..idx].trim().to_string();
            let end = s[idx + dash.len_utf8()..].trim().to_string();
            return (start, if end.is_empty() { None } else { Some(end) });
        }
    }
    if let Some(idx) = s.find(" to ") {
        return (
            s[..idx].trim().to_string(),
            Some(s[idx + 4..].trim().to_string()),
        );
    }
    (s.trim().to_string(), None)
}

fn looks_like_date(s: &str) -> bool {
    let s = s.trim();
    if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    s.len() == 7 && s.as_bytes().get(4) == Some(&b'-')
}

fn parse_bullet(
    item: &str,
    section: Sec,
    r: &mut Resume,
    wb: &mut Option<Work>,
    pb: &mut Option<Project>,
) {
    match section {
        Sec::Capabilities => {
            let name = inline::first_bold(item).unwrap_or_else(|| inline::to_plain(item));
            let metrics = inline::first_italic(item);
            let mut desc = inline::to_plain(item);
            if let Some(pos) = desc.find(&name) {
                desc.replace_range(pos..pos + name.len(), "");
            }
            if let Some(m) = &metrics {
                if let Some(pos) = desc.find(m.as_str()) {
                    desc.replace_range(pos..pos + m.len(), "");
                }
            }
            let desc = desc
                .trim()
                .trim_start_matches(['—', '-', '·'])
                .trim()
                .to_string();
            r.capabilities.push(Capability {
                name,
                description: if desc.is_empty() { None } else { Some(desc) },
                metrics,
            });
        }
        Sec::Skills => {
            let plain = inline::to_plain(item);
            let name = inline::first_bold(item)
                .map(|s| s.trim_end_matches(':').to_string())
                .unwrap_or_else(|| plain.clone());
            let rest = match plain.find(':') {
                Some(p) => plain[p + 1..].to_string(),
                None => String::new(),
            };
            let keywords = rest
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            r.skills.push(SkillGroup {
                name,
                level: None,
                keywords,
            });
        }
        Sec::Certifications => {
            let name = inline::first_bold(item).unwrap_or_else(|| inline::to_plain(item));
            let url = inline::extract_links(item).first().map(|(_, u)| u.clone());
            r.certificates.push(Certificate {
                name,
                date: None,
                issuer: None,
                url,
            });
        }
        Sec::Languages => {
            let language = inline::first_bold(item).unwrap_or_else(|| inline::to_plain(item));
            let plain = inline::to_plain(item);
            let fluency = plain
                .find('—')
                .map(|p| plain[p + '—'.len_utf8()..].trim().to_string());
            r.languages.push(Language { language, fluency });
        }
        Sec::Interests => {
            r.interests.push(Interest {
                name: inline::to_plain(item),
                keywords: Vec::new(),
            });
        }
        Sec::Awards => {
            let title = inline::first_bold(item).unwrap_or_else(|| inline::to_plain(item));
            r.awards.push(Award {
                title,
                ..Default::default()
            });
        }
        Sec::Experience => {
            if let Some(w) = wb.as_mut() {
                w.highlights.push(inline::to_plain(item));
            }
        }
        Sec::Projects => {
            if let Some(p) = pb.as_mut() {
                p.highlights.push(inline::to_plain(item));
            }
        }
        Sec::Extra => {
            r.extra.push(ExtraSection {
                title: String::new(),
                lines: vec![format!("- {}", item)],
            });
        }
        _ => {}
    }
}

fn flush_work(r: &mut Resume, wb: &mut Option<Work>) {
    if let Some(w) = wb.take() {
        if !w.name.is_empty() || !w.position.is_none() || !w.highlights.is_empty() {
            r.work.push(w);
        }
    }
}

fn flush_edu(r: &mut Resume, eb: &mut Option<Education>) {
    if let Some(e) = eb.take() {
        if !e.institution.is_empty() || e.study_type.is_some() {
            r.education.push(e);
        }
    }
}

fn flush_project(r: &mut Resume, pb: &mut Option<Project>) {
    if let Some(p) = pb.take() {
        if !p.name.is_empty() {
            r.projects.push(p);
        }
    }
}

fn flush_extra(r: &mut Resume, title: &mut String, lines: &mut Vec<String>) {
    if !title.is_empty() || !lines.is_empty() {
        r.extra.push(ExtraSection {
            title: std::mem::take(title),
            lines: std::mem::take(lines),
        });
    }
}
