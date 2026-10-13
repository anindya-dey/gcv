//! Structured format parsers: JSON (JSON Resume), YAML, and TOML.

use crate::model::Resume;

pub fn json(s: &str) -> Result<Resume, String> {
    serde_json::from_str(s).map_err(|e| format!("invalid JSON: {e}"))
}

pub fn yaml(s: &str) -> Result<Resume, String> {
    serde_yaml::from_str(s).map_err(|e| format!("invalid YAML: {e}"))
}

pub fn toml(s: &str) -> Result<Resume, String> {
    toml::from_str(s).map_err(|e| format!("invalid TOML: {e}"))
}
