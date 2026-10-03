//! The rows of a tree the list shows now.
//!
//! A long tree lays out only the rows in view and a few either side of it.
//! One spacer stands for the rows above and one for the rows below, so the
//! column keeps its whole height: the scroll range, the wheel and the
//! scroller are what they were. THYMUS's Outliner lists 288 rows, and laying
//! every one of them out each frame was most of a frame.
//!
//! The list also follows the picks. An object picked while the pointer is
//! off the list (a click in the viewport) has its row scrolled to the top,
//! as Blender's Show Active (`outliner_show_active`) puts it in view; the
//! pick has already opened the rows above it (`part::outliner::reveal`). A
//! click on the list itself leaves the list where it is, and so does an
//! object made active any other way, such as a file opening.

use super::super::ctx::BRenderCtx;
use super::super::scroller;
use crate::v2::state::OutlinerKey;
use crate::v2::view::BView;
use crate::v2::view::prim::BRowShade;
use gpui::{ScrollHandle, SharedString, point, px};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

/// Rows laid out past each end of the view, so a quick turn of the wheel
/// never shows the spacer before the next frame fills it.
const MARGIN: usize = 8;
/// A list this short is laid out whole.
const LONG: usize = 64;
/// Rows laid out before the list has had a layout to measure.
const FIRST: usize = 48;

/// What one list remembers between frames.
#[derive(Default)]
struct BFollow {
    /// The pick count (`ctx.outliner.reveal`) the list last showed.
    picks: Option<String>,
    hovered: Rc<Cell<bool>>,
}

impl BFollow {
    /// Is `picks` a pick to show: a new count, made off the list?
    fn picked(&mut self, picks: Option<String>) -> bool {
        let picked = picks.is_some() && picks != self.picks && !self.hovered.get();
        self.picks = picks;
        picked
    }
}

thread_local! {
    static FOLLOW: RefCell<HashMap<SharedString, BFollow>> = RefCell::new(HashMap::new());
}

/// The rows `first..last` of a tree list are laid out; the rest are spacers.
pub struct BRowWindow {
    pub handle: ScrollHandle,
    /// Set while the pointer is over the list (`on_hover`).
    pub hovered: Rc<Cell<bool>>,
    pub first: usize,
    pub last: usize,
    count: usize,
    pitch: f32,
    gap: f32,
}

impl BRowWindow {
    /// The window of `rows` for the list `key` (its area and host), after
    /// following the active row.
    pub fn new(key: SharedString, rows: &[BView], ctx: &BRenderCtx) -> Self {
        let handle = scroller::handle(&key);
        let gap = ctx.style.spacing.gap_xs;
        let pitch = ctx.style.dimensions.row_height + gap;
        let picks = ctx.state.get(OutlinerKey::Reveal).map(str::to_string);
        let hovered = Self::follow(key, rows, picks, &handle, pitch);
        let count = rows.len();
        let (first, last) = Self::span(count, -f32::from(handle.offset().y), f32::from(handle.bounds().size.height), pitch);
        Self { handle, hovered, first, last, count, pitch, gap }
    }

    /// The rows `first..last` to lay out of `count`, scrolled `scroll` down
    /// a view `view` tall.
    fn span(count: usize, scroll: f32, view: f32, pitch: f32) -> (usize, usize) {
        match () {
            _ if count < LONG => (0, count),
            _ if view <= 0.0 => (0, FIRST.min(count)),
            _ => {
                let first = ((scroll / pitch).floor().max(0.0) as usize).saturating_sub(MARGIN).min(count);
                let last = (((scroll + view) / pitch).ceil() as usize + MARGIN).min(count);
                (first, last.max(first))
            }
        }
    }

    /// Scroll the active row to the top after a pick made off the list. An
    /// object made active any other way (a file opening) scrolls nothing.
    fn follow(key: SharedString, rows: &[BView], picks: Option<String>, handle: &ScrollHandle, pitch: f32) -> Rc<Cell<bool>> {
        FOLLOW.with(|follow| {
            let mut follow = follow.borrow_mut();
            let entry = follow.entry(key).or_insert_with(|| BFollow { picks: picks.clone(), ..BFollow::default() });
            let picked = entry.picked(picks);
            let active = rows.iter().position(|row| matches!(row, BView::TreeRow { shade: BRowShade::Active, .. }));
            if let Some(i) = active.filter(|_| picked) {
                handle.set_offset(point(handle.offset().x, px(-(i as f32) * pitch)));
            }
            Rc::clone(&entry.hovered)
        })
    }

    /// How tall the rows are, all of them together — the laid-out ones and the
    /// spacers standing for the rest. A gap sits between rows, not after the
    /// last one.
    pub fn rows_height(&self) -> f32 {
        if self.count == 0 { 0.0 } else { self.count as f32 * self.pitch - self.gap }
    }

    /// The spacer standing for the rows above the window: their height less
    /// the one gap the column already puts after it.
    pub fn above(&self) -> Option<f32> {
        (self.first > 0).then(|| self.first as f32 * self.pitch - self.gap)
    }

    /// The spacer standing for the rows below the window.
    pub fn below(&self) -> Option<f32> {
        (self.last < self.count).then(|| (self.count - self.last) as f32 * self.pitch - self.gap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_list_lays_out_the_rows_in_view_and_a_margin() {
        assert_eq!(BRowWindow::span(20, 0.0, 300.0, 30.0), (0, 20), "a short list whole");
        assert_eq!(BRowWindow::span(288, 0.0, 0.0, 30.0), (0, FIRST), "before a layout");
        assert_eq!(BRowWindow::span(288, 0.0, 300.0, 30.0), (0, 10 + MARGIN));
        assert_eq!(BRowWindow::span(288, 3000.0, 300.0, 30.0), (100 - MARGIN, 110 + MARGIN));
        assert_eq!(BRowWindow::span(288, 8340.0, 300.0, 30.0), (278 - MARGIN, 288), "the foot");
    }

    #[test]
    fn a_pick_is_shown_once_and_only_off_the_list() {
        let mut follow = BFollow { picks: None, ..BFollow::default() };
        assert!(!follow.picked(None), "no pick yet");
        assert!(follow.picked(Some("1".into())), "a pick off the list");
        assert!(!follow.picked(Some("1".into())), "the same pick, the next frame");
        follow.hovered.set(true);
        assert!(!follow.picked(Some("2".into())), "a pick on the list");
        follow.hovered.set(false);
        assert!(follow.picked(Some("3".into())));
        assert!(!follow.picked(None), "a new file's state");
    }
}
