//! Project Manifest — Mirrors .rcproject bundle structure
//!
//! Corresponds to: project.json in Apple Reality Composer Pro

use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

/// Top-level project manifest (`project.json`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub name: String,
    pub version: String,
    pub default_scene: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

impl ProjectManifest {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            default_scene: "Main".to_string(),
            created: current_timestamp(),
            settings: std::collections::HashMap::new(),
        }
    }
}

fn current_timestamp() -> String {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}
