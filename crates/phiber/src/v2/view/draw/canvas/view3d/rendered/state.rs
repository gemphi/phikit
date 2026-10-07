//! What the render thread leaves behind between frames, on the thread that
//! paints: the worker, the keys of the scene and the view it was last given,
//! the image on show, and the three small logs the pacing reads
//! (`settle.rs`, `reframe.rs`, `clock.rs`). One concern, one file — and the
//! reason `mod.rs` no longer declares it inline.

use super::clock::BViewClock;
use super::reframe::BShotLog;
use super::settle::BSettle;
use super::worker::BRayWorker;
use gpui::RenderImage;
use std::cell::{Cell, RefCell};
use std::sync::Arc;

/// Longest rendered side in device pixels (keeps frames interactive).
pub(super) const MAX_SIDE: f32 = 1600.0;

#[derive(Default)]
pub(super) struct BRayState {
    pub worker: Option<Option<BRayWorker>>,
    pub scene: u64,
    pub view: u64,
    /// Shown image: (frame seq, view key, finished, image).
    pub image: Option<(u64, u64, bool, Arc<RenderImage>)>,
    /// Holds the traced size still while the viewport is being dragged.
    pub settle: BSettle,
    /// What each view posted lately was framed for (`reframe.rs`).
    pub shots: BShotLog,
    /// How long the posted view takes to arrive (`clock.rs`).
    pub clock: BViewClock,
}

thread_local! {
    pub(super) static STATE: RefCell<BRayState> = RefCell::new(BRayState::default());
    /// How many views have been posted to the tracer — each one restarts it.
    pub(super) static POSTED: Cell<usize> = const { Cell::new(0) };
}

/// How many times the tracer has been restarted with a new view. A window
/// drag should add at most a couple, however many frames it lasts.
pub fn views_posted() -> usize {
    POSTED.with(Cell::get)
}

/// The traced image on show, its size in device pixels and whether it is
/// of the view asked for last — a bordered view's is the region's pixels.
pub fn shown_image() -> Option<((u32, u32), bool)> {
    STATE.with(|s| {
        let s = s.borrow();
        let (_, v, _, img) = s.image.as_ref()?;
        let size = img.size(0);
        Some(((size.width.0 as u32, size.height.0 as u32), *v == s.view))
    })
}
