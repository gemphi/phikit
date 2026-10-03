//! Waking the window when the tracer lands a frame, rather than every vsync
//! while it waits.

/// The render thread published a frame the window has not drawn yet; and
/// the view it shows is still refining. The app draws the window again
/// when a frame lands (`BPhiberApp::ensure_ray_ticker`), not every vsync
/// while it waits: a whole window laid out sixty times a second for images
/// that land a few times a second was most of what made a refining viewport
/// sluggish.
pub static FRESH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub static REFINING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A frame landed — the render thread's, or a final render's sample
/// (`scene/render_job.rs`): the window is drawn once for it.
pub fn landed() {
    FRESH.store(true, std::sync::atomic::Ordering::Release);
}

/// A frame landed since the last call.
pub fn take_fresh() -> bool {
    FRESH.swap(false, std::sync::atomic::Ordering::AcqRel)
}

/// A window draw begins: a viewport in it says whether its image is still
/// on its way (`REFINING`, set as it paints); a window without one — the
/// Rendering workspace, say — is not refining, however the last viewport
/// drawn left the flag.
pub fn drawing() {
    REFINING.store(false, std::sync::atomic::Ordering::Release);
}

/// Whether the viewport's image is still on its way.
pub fn refining() -> bool {
    REFINING.load(std::sync::atomic::Ordering::Acquire)
}

/// The ray ticker is running (`BPhiberApp::ensure_ray_ticker`). A paint
/// that leaves its view refining with no ticker asks for one more frame
/// itself, since the ticker is armed from `render`, which read this frame's
/// flag before the paint set it — at start-up, and after a file opened, the
/// first image waited for the first click.
pub static TICKING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn ticking() -> bool {
    TICKING.load(std::sync::atomic::Ordering::Acquire)
}

pub fn set_ticking(on: bool) {
    TICKING.store(on, std::sync::atomic::Ordering::Release);
}
