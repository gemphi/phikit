//! `project.json` — what the project is.
//!
//! Serialised with `serde_json`, not with `format!`. The two versions this
//! replaces both built their JSON by interpolation, and both were one
//! quotation mark from writing a file nothing could read: an object called
//! `He said "hi"` closed the `"name"` string early. One of them half knew
//! it and did `obj_name.replace('"', "'")` on the scene's entity names —
//! which silently renames the user's objects and still leaves `\` and the
//! control characters. A serialiser escapes all of it and renames nothing.

use serde::Serialize;

/// The manifest's own settings block. Values the app reads back, so they
/// are named here rather than assembled at the call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct BProjectSetting {
    pub physics_enabled: bool,
    pub grid_visible: bool,
    pub render_quality: &'static str,
}

impl Default for BProjectSetting {
    /// What a freshly imported project opens with.
    fn default() -> Self {
        Self { physics_enabled: false, grid_visible: true, render_quality: "high" }
    }
}

/// `project.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BManifest {
    pub name: String,
    pub version: &'static str,
    pub default_scene: String,
    /// The asset this was made from, as the loader refers to it.
    pub source: String,
    pub object_count: usize,
    pub vertex_count: usize,
    /// Seconds since the epoch. `None` when the clock is before it, which
    /// is written as `null` rather than as a zero that reads like 1970.
    pub created: Option<u64>,
    pub settings: BProjectSetting,
}

impl BManifest {
    /// The manifest format this crate writes. A reader that does not know
    /// it should say so rather than guess.
    pub const VERSION: &'static str = "1.0.0";

    /// The manifest for `name`, describing `source`.
    pub fn new(name: &str, source: &str, object_count: usize, vertex_count: usize) -> Self {
        Self {
            name: name.to_string(),
            version: Self::VERSION,
            default_scene: name.to_string(),
            source: source.to_string(),
            object_count,
            vertex_count,
            created: Self::now(),
            settings: BProjectSetting::default(),
        }
    }

    /// Now, in seconds since the epoch.
    fn now() -> Option<u64> {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .ok()
            .map(|since| since.as_secs())
    }

    /// The file's bytes: pretty printed, because a manifest is read by
    /// people as well as by the app, and newline terminated.
    pub fn text(&self) -> Result<String, serde_json::Error> {
        Ok(format!("{}\n", serde_json::to_string_pretty(self)?))
    }
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod manifest_tests;
