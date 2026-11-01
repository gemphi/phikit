//! Global layout state — tracks which preset is active.

use std::sync::{Mutex, OnceLock};
use super::WorkspacePreset;

/// Free-function wrapper: maps a layout key string to a preset.
pub fn preset_for_key(key: &str) -> WorkspacePreset {
    NSWorkspaceState::preset_for_key(key)
}

/// Free-function wrapper: returns the current layout key.
pub fn current_layout_key() -> String {
    NSWorkspaceState::current_layout_key()
}

pub struct NSWorkspaceState;

impl NSWorkspaceState {

/// Resolves the active workspace preset from the current layout key.
pub fn active_preset() -> WorkspacePreset {
    let layout_key = active_layout_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    preset_for_key(&layout_key)
}

/// Maps a layout key string to a preset.
pub fn preset_for_key(key: &str) -> WorkspacePreset {
    match key {
        "compact" => WorkspacePreset::compact(),
        "macos" | "classic" => WorkspacePreset::classic(),
        "modern" => WorkspacePreset::modern(),
        "kero" => WorkspacePreset::phipro(),
        "minimal" => WorkspacePreset::minimal(),
        "editorial" => WorkspacePreset::editorial(),
        _ => WorkspacePreset::default(),
    }
}

/// Sets the active layout preset by key.
pub fn set_layout(key: &str) {
    *active_layout_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = key.to_string();
    println!("[WorkspaceLayout] Active preset: {}", key);
}

/// Returns the current layout key.
pub fn current_layout_key() -> String {
    active_layout_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

/// Returns true if we're in PhiPro (distraction-free).
pub fn is_phipro() -> bool {
    current_layout_key() == "kero"
}

} // impl NSWorkspaceState

// ─── Internal ────────────────────────────────────────────────────────

fn active_layout_store() -> &'static Mutex<String> {
    static STORE: OnceLock<Mutex<String>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new("default".to_string()))
}
