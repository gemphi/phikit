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

/// One viewport's share of the tracer. Every 3D Viewport that traces —
/// the Animation workspace's camera view beside its 3D Viewport, two in
/// Material Preview — keeps its own view, image and size here, while the
/// one worker traces one view at a time. With a single view and image for
/// the whole window, two viewports posted their views over each other on
/// every frame, each restarting the other's trace, and neither settled.
#[derive(Default)]
pub(super) struct BRaySlot {
    /// Where the viewport was painted: how its next paint finds the slot.
    pub at: (f32, f32),
    /// The view it last posted.
    pub view: u64,
    /// Shown image: (frame seq, view key, finished, image).
    pub image: Option<(u64, u64, bool, Arc<RenderImage>)>,
    /// Holds the traced size still while the viewport is being dragged.
    pub settle: BSettle,
    /// The paint it was last seen on.
    seen: u64,
}

#[derive(Default)]
pub(super) struct BRayState {
    pub worker: Option<Option<BRayWorker>>,
    pub scene: u64,
    pub slots: Vec<BRaySlot>,
    /// The slot painted last.
    last: usize,
    /// The view the worker was last given, and whether it has finished it.
    current: (u64, bool),
    /// The newest frame taken from the worker.
    pub taken: u64,
    /// Viewport paints so far, the clock slots age by.
    paints: u64,
    /// What each view posted lately was framed for (`reframe.rs`).
    pub shots: BShotLog,
    /// How long the posted view takes to arrive (`clock.rs`).
    pub clock: BViewClock,
}

impl BRayState {
    /// How far a viewport may move between paints and keep its slot: an
    /// area edge dragged moves it a little each frame; another viewport is
    /// a whole area away.
    const REACH: f32 = 200.0;
    /// Paints a slot may go unseen before it is dropped with its image.
    const STALE: u64 = 240;

    /// The slot for the viewport painted at `at` — the nearest within
    /// [`Self::REACH`], or a new one — and the images of slots no longer
    /// painted, for the caller to drop.
    pub fn slot(&mut self, at: (f32, f32)) -> (usize, Vec<Arc<RenderImage>>) {
        self.paints += 1;
        let now = self.paints;
        let mut gone = Vec::new();
        self.slots.retain_mut(|slot| {
            let keep = now - slot.seen <= Self::STALE;
            if !keep && let Some((.., img)) = slot.image.take() {
                gone.push(img);
            }
            keep
        });
        let distance = |slot: &BRaySlot| (slot.at.0 - at.0).abs() + (slot.at.1 - at.1).abs();
        let near = self.slots.iter().enumerate().filter(|(_, slot)| distance(slot) < Self::REACH).min_by(|a, b| distance(a.1).total_cmp(&distance(b.1))).map(|(i, _)| i);
        let i = near.unwrap_or_else(|| {
            self.slots.push(BRaySlot::default());
            self.slots.len() - 1
        });
        (self.slots[i].at, self.slots[i].seen, self.last) = (at, now, i);
        (i, gone)
    }

    /// Whether slot `i` posts `key` now: its view changed, or the worker
    /// has finished another viewport's view while this one's is unfinished.
    /// So two viewports take turns, each refined to the end, rather than
    /// taking the worker from each other on every frame.
    pub fn should_post(&self, i: usize, key: u64) -> bool {
        let slot = &self.slots[i];
        let finished = slot.image.as_ref().is_some_and(|(_, v, done, _)| *v == key && *done);
        key != slot.view || (self.current.0 != key && self.current.1 && !finished)
    }

    /// Slot `i` has posted `key` and the worker is on it.
    pub fn posted(&mut self, i: usize, key: u64) {
        self.slots[i].view = key;
        self.current = (key, false);
    }

    /// Every slot posts again: the scene changed under all of them.
    pub fn repost(&mut self) {
        self.slots.iter_mut().for_each(|slot| slot.view = 0);
        self.current = (0, true);
    }

    /// A frame of `view` has landed (`done` when the view is finished): the
    /// worker's progress is noted, and the image — when there is one — goes
    /// to the slot that posted the view, the one painting (`BRayWorker::take`).
    /// Answers the image it replaced.
    pub fn land(&mut self, seq: u64, view: u64, done: bool, image: Option<Arc<RenderImage>>) -> Option<Arc<RenderImage>> {
        self.taken = seq;
        if self.current.0 == view {
            self.current.1 = done;
        }
        let slot = self.slots.iter_mut().find(|slot| slot.view == view)?;
        let (.., old) = slot.image.replace((seq, view, done, image?))?;
        Some(old)
    }
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
        let slot = s.slots.get(s.last)?;
        let (_, v, _, img) = slot.image.as_ref()?;
        let size = img.size(0);
        Some(((size.width.0 as u32, size.height.0 as u32), *v == slot.view))
    })
}
