//! Canonical resume model. Aligned with the JSON Resume schema where possible
//! so that JSON/YAML/TOML inputs deserialize directly; the Markdown parser
//! builds the same structure from a human-readable document.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Resume {
    pub meta: Meta,
    pub basics: Basics,
    pub capabilities: Vec<Capability>,
    #[serde(alias = "experience", alias = "employment")]
    pub work: Vec<Work>,
    pub projects: Vec<Project>,
    #[serde(alias = "skillCategories")]
    pub skills: Vec<SkillGroup>,
    pub education: Vec<Education>,
    #[serde(alias = "certifications", alias = "certificates")]
    pub certificates: Vec<Certificate>,
    pub languages: Vec<Language>,
    pub interests: Vec<Interest>,
    pub awards: Vec<Award>,
    #[serde(skip)]
    pub extra: Vec<ExtraSection>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Meta {
    pub theme: Option<String>,
    pub accent: Option<String>,
    pub density: Option<String>,
    pub width: Option<usize>,
    pub banner: Option<bool>,
    pub avatar: Option<String>,
    pub qr: Option<bool>,
    pub links: Option<bool>,
    pub icons: Option<bool>,
    pub borders: Option<String>,
    pub tagline: Option<String>,
    pub footer: Option<String>,
    pub sections: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Basics {
    pub name: Option<String>,
    pub label: Option<String>,
    pub image: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub location: Option<Location>,
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Location {
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub network: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Capability {
    #[serde(alias = "title")]
    pub name: String,
    pub description: Option<String>,
    pub metrics: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Work {
    #[serde(alias = "company")]
    pub name: String,
    #[serde(alias = "role")]
    pub position: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub summary: Option<String>,
    pub highlights: Vec<String>,
    pub keywords: Vec<String>,
    #[serde(rename = "startDate", alias = "start")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate", alias = "end")]
    pub end_date: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    pub name: String,
    pub description: Option<String>,
    pub summary: Option<String>,
    pub highlights: Vec<String>,
    pub keywords: Vec<String>,
    #[serde(rename = "startDate", alias = "start")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate", alias = "end")]
    pub end_date: Option<String>,
    pub url: Option<String>,
    pub roles: Vec<String>,
    pub entity: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    #[serde(skip)]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Default)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillGroup {
    pub name: String,
    pub level: Option<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Education {
    pub institution: String,
    pub url: Option<String>,
    pub area: Option<String>,
    #[serde(rename = "studyType", alias = "degree")]
    pub study_type: Option<String>,
    #[serde(rename = "startDate", alias = "start")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate", alias = "end")]
    pub end_date: Option<String>,
    pub score: Option<String>,
    pub courses: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Certificate {
    pub name: String,
    #[serde(alias = "date")]
    pub date: Option<String>,
    pub issuer: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Language {
    #[serde(alias = "name")]
    pub language: String,
    pub fluency: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Interest {
    pub name: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Award {
    pub title: String,
    pub date: Option<String>,
    pub awarder: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtraSection {
    pub title: String,
    pub lines: Vec<String>,
}

impl Resume {
    pub fn location_text(&self) -> Option<String> {
        let loc = self.basics.location.as_ref()?;
        if let Some(a) = &loc.address {
            if !a.trim().is_empty() {
                return Some(a.clone());
            }
        }
        let mut parts = Vec::new();
        for p in [&loc.city, &loc.region, &loc.country_code]
            .into_iter()
            .flatten()
        {
            if !p.trim().is_empty() {
                parts.push(p.clone());
            }
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    }

    pub fn humanize_date(s: &str) -> String {
        let s = s.trim();
        if s.is_empty() {
            return String::new();
        }
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        if s.len() == 7 && s.as_bytes()[4] == b'-' {
            if let (Ok(y), Ok(m)) = (s[0..4].parse::<i32>(), s[5..7].parse::<usize>()) {
                if (1..=12).contains(&m) {
                    return format!("{} {}", months[m - 1], y);
                }
            }
        }
        if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
            return s.to_string();
        }
        s.to_string()
    }

    pub fn date_range(start: Option<&str>, end: Option<&str>) -> String {
        let s = start.map(Self::humanize_date).unwrap_or_default();
        let e = match end {
            Some(x) if !x.trim().is_empty() => Self::humanize_date(x),
            _ => "Present".to_string(),
        };
        match (s.is_empty(), e.is_empty()) {
            (true, true) => String::new(),
            (true, false) => e,
            (false, true) => s,
            (false, false) => format!("{} – {}", s, e),
        }
    }

    /// Ordered list of section keys that have content, honoring meta.sections.
    pub fn section_order(&self) -> Vec<String> {
        let all = [
            "summary",
            "capabilities",
            "experience",
            "projects",
            "skills",
            "education",
            "certifications",
            "languages",
            "interests",
            "awards",
            "extra",
        ];
        let mut order: Vec<String> = match &self.meta.sections {
            Some(list) => {
                let mut v: Vec<String> = Vec::new();
                for item in list {
                    let key = normalize_section(item);
                    if !v.contains(&key) {
                        v.push(key);
                    }
                }
                for a in all {
                    if !v.contains(&a.to_string()) {
                        v.push(a.to_string());
                    }
                }
                v
            }
            None => all.iter().map(|s| s.to_string()).collect(),
        };
        order.retain(|k| self.has_section(k));
        order
    }

    pub fn has_section(&self, key: &str) -> bool {
        match key {
            "summary" => self
                .basics
                .summary
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false),
            "capabilities" => !self.capabilities.is_empty(),
            "experience" => !self.work.is_empty(),
            "projects" => !self.projects.is_empty(),
            "skills" => !self.skills.is_empty(),
            "education" => !self.education.is_empty(),
            "certifications" => !self.certificates.is_empty(),
            "languages" => !self.languages.is_empty(),
            "interests" => !self.interests.is_empty(),
            "awards" => !self.awards.is_empty(),
            "extra" => !self.extra.is_empty(),
            _ => false,
        }
    }
}

pub fn normalize_section(s: &str) -> String {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "summary" | "about" | "profile" | "objective" => "summary",
        "capabilities" | "expertise" | "what-i-do" | "strengths" => "capabilities",
        "experience" | "work" | "employment" | "career" => "experience",
        "projects" | "portfolio" => "projects",
        "skills" | "technologies" | "tech" | "stack" => "skills",
        "education" | "studies" => "education",
        "certifications" | "certificates" | "certification" | "certs" => "certifications",
        "languages" => "languages",
        "interests" | "hobbies" => "interests",
        "awards" | "honors" | "honours" => "awards",
        other => {
            return other.to_string();
        }
    }
    .to_string()
}
