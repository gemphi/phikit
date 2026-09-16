//! View-state keys + accessors. Split out of `scene/mod.rs` so the parent
//! stays under the 200-line cap.
//!
//! Each `_key()` function returns the canonical `BPath` for a scene-state
//! entry. `view_*` fns read the current value from `BState` with a Blender
//! factory default and a documented clamp where relevant.

use crate::v2::state::{BPath, BState};

// ── Yaw / zoom / perspective ────────────────────────────────────────────

/// State key for the view yaw (degrees, 0..360). Written by the axis gizmo.
pub fn yaw_key() -> BPath {
    BPath::owned(vec!["view3d", "yaw"])
}

/// State key for the view zoom factor. Written by scroll-to-zoom and the
/// gizmo zoom buttons.
pub fn zoom_key() -> BPath {
    BPath::owned(vec!["view3d", "zoom"])
}

/// State key for the perspective / orthographic toggle.
/// `"true"` = perspective (the Blender default); absent or `"false"` =
/// orthographic.
pub fn perspective_key() -> BPath {
    BPath::owned(vec!["view3d", "perspective"])
}

/// Is the viewport in perspective mode? Defaults to `true` (Blender default).
pub fn is_perspective(s: &BState) -> bool {
    s.get(&perspective_key()) != Some("false")
}

// ── Pitch (elevation) state ─────────────────────────────────────────────
//
// The pitch axis tilts the camera up/down about the horizontal plane.
// 0° = looking level with the ground plane; +30° = looking down onto it
// (Blender's default user-perspective elevation).

/// State key for the view pitch (degrees). Written by the orbit up/down
/// gizmo controls and clamped to `[-90, 90]` on read.
pub fn pitch_key() -> BPath {
    BPath::owned(vec!["view3d", "pitch"])
}

/// Read the current view pitch in degrees, clamped to `[-90, 90]`.
/// Defaults to 30° — Blender's startup elevation angle.
pub fn view_pitch(s: &BState) -> f32 {
    s.get(&pitch_key())
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(30.0)
        .clamp(-90.0, 90.0)
}

/// Add `delta_deg` to the current pitch and clamp to `[-90, 90]`.
pub fn pitch_step(pitch: f32, delta_deg: f32) -> f32 {
    (pitch + delta_deg).clamp(-90.0, 90.0)
}

// ── Pan (view translation) state ────────────────────────────────────────
//
// The pan axis translates the view laterally in screen space without
// rotating the camera. Values are in the same normalized coord space as
// `project3` (roughly `[-1, 1]` covers the visible area at zoom = 1).

/// State key for horizontal pan (screen-x offset).
pub fn pan_x_key() -> BPath {
    BPath::owned(vec!["view3d", "pan", "x"])
}

/// State key for vertical pan (screen-y offset).
pub fn pan_y_key() -> BPath {
    BPath::owned(vec!["view3d", "pan", "y"])
}

/// Read the current view pan as `(x, y)` in screen-normalized units.
/// Defaults to `(0.0, 0.0)`.
pub fn view_pan(s: &BState) -> (f32, f32) {
    let x = s
        .get(&pan_x_key())
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.0);
    let y = s
        .get(&pan_y_key())
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.0);
    (x, y)
}

// ── File / import state keys ────────────────────────────────────────────

/// State key for the last file row clicked in the File_Browser. The
/// File_Browser writes the selected file's resolvable path here; the import
/// orchestration observes transitions of this key to trigger a blend load.
pub fn selected_key() -> BPath {
    BPath::owned(vec!["file", "selected"])
}

/// State key for the import-in-progress flag. Holds `"true"` while a blend
/// import is running, absent otherwise.
pub fn loading_key() -> BPath {
    BPath::owned(vec!["scene", "loading"])
}

/// State key for the current import **stage label** (Blender's job status
/// text, e.g. "Reading file", "Parsing meshes", "Building materials"). Absent
/// when no import is running. Purely descriptive — the loading gate is still
/// [`loading_key`].
pub fn load_stage_key() -> BPath {
    BPath::owned(vec!["scene", "load_stage"])
}

/// State key for the import **progress fraction** in `0.0..=1.0`, stored as a
/// short decimal string. Absent when no import is running. Drives the width
/// of the statusbar progress bar (Blender's `WM_progress`).
pub fn load_progress_key() -> BPath {
    BPath::owned(vec!["scene", "load_progress"])
}

/// The ordered import stages surfaced to the user. Each carries its label and
/// the progress fraction the bar reaches when that stage begins. Mirrors
/// Blender's file-read job phases (read → parse geometry → build materials →
/// done).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadStage {
    /// Opening the file / decoding the container (sidecar or SDNA header).
    Reading,
    /// Walking blocks and triangulating mesh geometry.
    ParsingMeshes,
    /// Resolving material slots and texture references.
    BuildingMaterials,
    /// Import settled — bar full, about to clear.
    Done,
}

impl LoadStage {
    /// Human-readable status label shown next to the progress bar.
    pub fn label(self) -> &'static str {
        match self {
            LoadStage::Reading => "Reading file",
            LoadStage::ParsingMeshes => "Parsing meshes",
            LoadStage::BuildingMaterials => "Building materials",
            LoadStage::Done => "Done",
        }
    }

    /// Progress fraction (0..=1) the bar reaches when this stage begins.
    pub fn progress(self) -> f32 {
        match self {
            LoadStage::Reading => 0.15,
            LoadStage::ParsingMeshes => 0.5,
            LoadStage::BuildingMaterials => 0.85,
            LoadStage::Done => 1.0,
        }
    }
}

/// Read the current import progress fraction (0..=1), or `None` when no import
/// is running. Clamped so a corrupt value can never overflow the bar.
pub fn view_load_progress(s: &BState) -> Option<f32> {
    s.get(&load_progress_key())
        .and_then(|v| v.parse::<f32>().ok())
        .map(|v| v.clamp(0.0, 1.0))
}

/// Read the current import stage label, or `None` when no import is running.
pub fn view_load_stage(s: &BState) -> Option<&str> {
    s.get(&load_stage_key())
}

/// State key for the Load_Error_State message (1..=256 chars) set when the
/// most recent import failed, absent when there is no error.
pub fn error_key() -> BPath {
    BPath::owned(vec!["scene", "error"])
}

/// State key for the path of the import currently reflected by the
/// Scene_Source. Compared against `selected_key` to decide whether a new
/// import must be initiated.
pub fn loaded_path_key() -> BPath {
    BPath::owned(vec!["scene", "loaded_path"])
}

/// True iff `name` ends (case-insensitive) with the `.blend` suffix. Shared
/// predicate used by both the File_Browser gate and the import trigger.
pub fn is_blend_path(name: &str) -> bool {
    let suffix = ".blend";
    name.len() >= suffix.len() && name[name.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

// ── Zoom clamp + view yaw/zoom accessors ────────────────────────────────

/// Zoom clamp range — how far the view can dolly in/out. 1.0 is the default
/// framing; below 1 pulls back, above 1 pushes in.
pub const ZOOM_MIN: f32 = 0.4;
pub const ZOOM_MAX: f32 = 3.0;

/// Read the current view yaw in degrees (default 30° — Blender's startup
/// user-perspective angle).
pub fn view_yaw(s: &BState) -> f32 {
    s.get(&yaw_key())
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(30.0)
}

/// Read the current view zoom factor (default 1.0), clamped to
/// `[ZOOM_MIN, ZOOM_MAX]` so a corrupt state value can never explode the
/// projection.
pub fn view_zoom(s: &BState) -> f32 {
    s.get(&zoom_key())
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(1.0)
        .clamp(ZOOM_MIN, ZOOM_MAX)
}

/// Multiply `zoom` by `factor` and clamp to the allowed range. Scroll up
/// zooms in (factor > 1), scroll down zooms out (factor < 1).
pub fn zoom_step(zoom: f32, factor: f32) -> f32 {
    (zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX)
}
