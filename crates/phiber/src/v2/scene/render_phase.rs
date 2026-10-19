//! What the render screen is showing, as one answer.
//!
//! The Render Result can be in one of nine states, and the UI was reading
//! the raw signals at each place that cared — `info_text` matched on
//! `(done, paused())` for its line, the canvas asked `has_image` before
//! painting, the header asked `rendering()` for its buttons — so no one
//! place said what the screen WAS, and the states nobody had written down
//! (no camera, cancelled, saved, nothing rendered yet) were not drawn at
//! all. This is that one place.
//!
//! The nine, and what each is grounded in:
//!
//! | phase | the manual / Blender |
//! |---|---|
//! | `Empty` | nothing rendered yet. A fresh document's Render Result slot is empty (`render_job::forget`), and the Image editor shows the slot with no image. |
//! | `NoCamera` | `render_job::plan` returns nothing without a camera; Blender refuses with "No camera found in scene" (`render/cameras`). |
//! | `Preparing` | the job is running and has put no pixels up yet. The status bar's Running Task is up (`interface/window_system/status_bar`, "Shows the progress of the currently running task (such as rendering or baking)"). |
//! | `Rendering` | samples climbing, the picture filling in. `Sample n/m`. |
//! | `Paused` | held after the current sample. Blender's render job has no pause — `WM_jobs` is stop or nothing — so this is phiber's, deliberately (`render_job::pause`). |
//! | `Cancelled` | stopped by the status bar's cancel button, the partial picture kept. The manual leans on this for animations: a job "can be canceled and resumed at the last rendered frame" (`render/output/introduction`). |
//! | `Finished` | every sample taken. |
//! | `Saved` | the finished picture written to a file. Stills "must be saved manually from the render buffer", so being saved is a state of the buffer the UI can report (`render/output/animation`). |
//! | `Failed` | the job ended with no picture — the tracer refused the scene (`phiray::Tracer::why_not`) or the scene reached it empty. Blender reports a failed render as an error in the status bar and keeps the slot; the reason here is `render_job::failure()`. |
//!
//! The order of the arms is the order the questions have to be asked in: a
//! cancelled render is also not running, a paused one is also unfinished, so
//! the narrower question comes first.

use super::render_status::BRenderStatus;

/// Where the render screen is. One value, read everywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BRenderPhase {
    /// Nothing has been rendered into this slot.
    Empty,
    /// The scene has no camera, so there is nothing to render through.
    NoCamera,
    /// Under way, with no picture up yet.
    Preparing,
    /// Under way, the picture filling in.
    Rendering,
    /// Held after its current sample.
    Paused,
    /// Stopped early; what it had is still up.
    Cancelled,
    /// Every sample taken.
    Finished,
    /// Finished and written to a file.
    Saved,
    /// The job ended without a picture: the tracer refused the scene, or the
    /// scene reached it empty. The reason is `render_job::failure()`.
    Failed,
}

/// What the screen is, from the signals the job and the scene carry.
///
/// `camera` is whether the scene has one to render through, `saved` whether
/// the picture up has been written to a file since it finished.
pub fn phase(status: Option<BRenderStatus>, running: bool, paused: bool, cancelled: bool, camera: bool, saved: bool) -> BRenderPhase {
    match (status, running) {
        // A job that ended with no image at all is a failure, not an empty
        // slot: the screen said "No render" after F12 on pose 26 while the
        // journal alone knew the GPU had declined the scene.
        (Some(BRenderStatus { has_image: false, done: true, .. }), false) => BRenderPhase::Failed,
        // Nothing in the slot: either there is no camera to fill it or it
        // has simply not been rendered. The camera comes first because it is
        // the reason, and "no camera" is actionable where "empty" is not.
        (None, _) | (Some(BRenderStatus { has_image: false, .. }), false) => match camera {
            false => BRenderPhase::NoCamera,
            true => BRenderPhase::Empty,
        },
        // Running: held, or starting, or filling in.
        (Some(status), true) => match (paused, status.has_image) {
            (true, _) => BRenderPhase::Paused,
            (_, false) => BRenderPhase::Preparing,
            _ => BRenderPhase::Rendering,
        },
        // Stopped with a picture: cancelled keeps what it had, a finished one
        // may since have been written out.
        (Some(status), false) => match (cancelled, status.done, saved) {
            (true, false, _) => BRenderPhase::Cancelled,
            (_, true, true) => BRenderPhase::Saved,
            (_, true, _) => BRenderPhase::Finished,
            // Not running, not cancelled, not done: the job went away mid
            // flight, so what is up is what it reached.
            _ => BRenderPhase::Cancelled,
        },
    }
}

impl BRenderPhase {
    /// Whether a picture is up — the canvas paints only for these.
    pub fn has_picture(self) -> bool {
        matches!(self, Self::Rendering | Self::Paused | Self::Cancelled | Self::Finished | Self::Saved)
    }

    /// Whether the job is still going, so the header offers Pause and Cancel
    /// and the status bar carries its Running Task.
    pub fn is_running(self) -> bool {
        matches!(self, Self::Preparing | Self::Rendering | Self::Paused)
    }

    /// Whether what is up can be written to a file (`render_save::save_to`
    /// refuses a render that is still running).
    pub fn can_save(self) -> bool {
        matches!(self, Self::Cancelled | Self::Finished | Self::Saved)
    }

    /// The tail of the info bar's line — `make_renderinfo_string`'s last
    /// field, which Blender fills with the engine's own status.
    pub fn info(self, taken: u32, target: u32) -> String {
        match self {
            Self::Empty => "No render".to_string(),
            Self::NoCamera => "No camera found in scene".to_string(),
            Self::Preparing => "Preparing…".to_string(),
            Self::Rendering => format!("Sample {taken}/{target}"),
            Self::Paused => format!("Paused at sample {taken}/{target}"),
            Self::Cancelled => format!("Cancelled at sample {taken}/{target}"),
            Self::Finished => "Finished".to_string(),
            Self::Saved => "Saved".to_string(),
            Self::Failed => format!("Render failed: {}", super::render_job::failure().unwrap_or_else(|| "no image".to_string())),
        }
    }

    /// What the canvas says over an empty slot, when there is no picture to
    /// show and the reason is worth saying.
    pub fn placeholder(self) -> Option<&'static str> {
        match self {
            Self::Empty => Some("No render — F12 to render this frame"),
            Self::NoCamera => Some("No camera found in scene"),
            Self::Preparing => Some("Preparing…"),
            Self::Failed => Some("Render failed — the reason is in the info bar above and the Info editor"),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "render_phase_tests.rs"]
mod tests;
