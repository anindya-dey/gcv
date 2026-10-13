//! Parsers for the supported input formats.

mod markdown;
mod structured;

use crate::model::Resume;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Json,
    Yaml,
    Toml,
}

impl Format {
    pub fn from_path(path: &str) -> Option<Format> {
        let lower = path.to_lowercase();
        let ext = lower.rsplit('.').next().unwrap_or("");
        match ext {
            "md" | "markdown" | "mdx" => Some(Format::Markdown),
            "json" => Some(Format::Json),
            "yaml" | "yml" => Some(Format::Yaml),
            "toml" => Some(Format::Toml),
            _ => None,
        }
    }

    pub fn sniff(content: &str) -> Format {
        let t = content.trim_start();
        if t.starts_with('{') {
            Format::Json
        } else if t.starts_with("---") || t.starts_with('#') || t.starts_with("<!--") {
            Format::Markdown
        } else if t.contains("[[") || t.contains(" = ") {
            Format::Toml
        } else {
            Format::Yaml
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Format::Markdown => "markdown",
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Toml => "toml",
        }
    }
}

pub fn parse(content: &str, format: Format) -> Result<Resume, String> {
    match format {
        Format::Markdown => markdown::parse(content),
        Format::Json => structured::json(content),
        Format::Yaml => structured::yaml(content),
        Format::Toml => structured::toml(content),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"{
      "basics": { "name": "Ada Lovelace", "label": "Mathematician" },
      "work": [{
        "name": "Analytical Engines",
        "position": "Mathematician",
        "highlights": ["Wrote the first algorithm"]
      }]
    }"#;

    const YAML: &str = "basics:\n  name: Ada Lovelace\n  label: Mathematician\nwork:\n  - name: Analytical Engines\n    position: Mathematician\n    highlights:\n      - Wrote the first algorithm\n";

    const TOML: &str = "[basics]\nname = \"Ada Lovelace\"\nlabel = \"Mathematician\"\n\n[[work]]\nname = \"Analytical Engines\"\nposition = \"Mathematician\"\nhighlights = [\"Wrote the first algorithm\"]\n";

    #[test]
    fn structured_formats_agree() {
        for (f, content) in [
            (Format::Json, JSON),
            (Format::Yaml, YAML),
            (Format::Toml, TOML),
        ] {
            let r = parse(content, f).unwrap_or_else(|e| panic!("{}: {e}", f.label()));
            assert_eq!(r.basics.name.as_deref(), Some("Ada Lovelace"));
            assert_eq!(r.work.len(), 1);
            assert_eq!(r.work[0].name, "Analytical Engines");
            assert_eq!(r.work[0].highlights.len(), 1);
        }
    }

    const MD: &str = r#"---
meta:
  theme: neon
  accent: cyan
---

# Ada Lovelace

**Mathematician** · London, UK
[ada@example.com](mailto:ada@example.com) · [github.com/ada](https://github.com/ada)

## Summary

First programmer.

## Experience

### Mathematician · Analytical Engines
_1842 – 1843 · London_

- Wrote the first algorithm

`Notes` `Math`
"#;

    #[test]
    fn markdown_parses_convention() {
        let r = parse(MD, Format::Markdown).unwrap();
        assert_eq!(r.basics.name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(r.basics.label.as_deref(), Some("Mathematician"));
        assert_eq!(r.meta.theme.as_deref(), Some("neon"));
        assert_eq!(r.meta.accent.as_deref(), Some("cyan"));
        assert_eq!(r.basics.email.as_deref(), Some("ada@example.com"));
        assert!(r
            .basics
            .profiles
            .iter()
            .any(|p| p.url.as_deref() == Some("https://github.com/ada")));
        assert_eq!(r.basics.summary.as_deref(), Some("First programmer."));
        assert_eq!(r.work.len(), 1);
        assert_eq!(r.work[0].position.as_deref(), Some("Mathematician"));
        assert_eq!(r.work[0].name, "Analytical Engines");
        assert_eq!(r.work[0].highlights.len(), 1);
        assert_eq!(r.work[0].keywords, vec!["Notes", "Math"]);
    }

    #[test]
    fn format_detection() {
        assert_eq!(Format::from_path("resume.md"), Some(Format::Markdown));
        assert_eq!(Format::from_path("resume.json"), Some(Format::Json));
        assert_eq!(Format::from_path("cv.YAML"), Some(Format::Yaml));
        assert_eq!(Format::from_path("a/b/c.toml"), Some(Format::Toml));
        assert_eq!(Format::from_path("README"), None);
        assert_eq!(Format::sniff("{\"a\":1}"), Format::Json);
        assert_eq!(Format::sniff("---\nmeta: {}\n---\n# Hi"), Format::Markdown);
    }
}
