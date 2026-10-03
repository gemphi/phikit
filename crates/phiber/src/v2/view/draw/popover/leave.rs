//! Closing a menu the pointer has left — Blender's menu blocks close once
//! the mouse strays far enough from them (`ui_handle_menu_event`,
//! `BLOCK_MOVEMOUSE_QUIT`): the panel and its anchor record where they were
//! painted, and the backdrop under them asks whether the pointer has gone.

use gpui::prelude::*;
use gpui::div;

/// Where an open popover's panel and anchor were painted, for closing it
/// when the pointer wanders off (`left`).
#[derive(Clone, Default)]
pub(super) struct BLeave {
    panel: std::rc::Rc<std::cell::Cell<Option<gpui::Bounds<gpui::Pixels>>>>,
    anchor: std::rc::Rc<std::cell::Cell<Option<gpui::Bounds<gpui::Pixels>>>>,
}

impl BLeave {
    /// How far past the panel the pointer may stray before the menu closes.
    const MARGIN: f32 = 64.0;

    fn probe(cell: std::rc::Rc<std::cell::Cell<Option<gpui::Bounds<gpui::Pixels>>>>) -> gpui::Div {
        div().absolute().inset_0().child(gpui::canvas(move |b, _, _| cell.set(Some(b)), |_, _, _, _| {}).size_full())
    }

    pub(super) fn panel_probe(&self) -> gpui::Div {
        Self::probe(self.panel.clone())
    }

    pub(super) fn anchor_probe(&self) -> gpui::Div {
        Self::probe(self.anchor.clone())
    }

    /// Whether `at` has left: beyond the margin round the panel, and off the
    /// anchor's row.
    pub(super) fn left(&self, at: gpui::Point<gpui::Pixels>) -> bool {
        let Some(panel) = self.panel.get() else { return false };
        let (x, y) = (f32::from(at.x), f32::from(at.y));
        if let Some(anchor) = self.anchor.get() {
            if y >= f32::from(anchor.origin.y) && y <= f32::from(anchor.origin.y + anchor.size.height) {
                return false;
            }
        }
        let (left, top) = (f32::from(panel.origin.x), f32::from(panel.origin.y));
        let (right, bottom) = (left + f32::from(panel.size.width), top + f32::from(panel.size.height));
        let dx = (left - x).max(x - right).max(0.0);
        let dy = (top - y).max(y - bottom).max(0.0);
        dx.max(dy) > Self::MARGIN
    }

    /// The full-viewport layer under the panel. It carries both ways a
    /// popover is dismissed by the pointer: a click that lands outside the
    /// panel, and a move that has strayed past [`Self::MARGIN`].
    ///
    /// The panel is `.occlude()`d, so neither event fires while the pointer
    /// is over it — a move that arrives here is already outside.
    pub(super) fn backdrop(
        &self,
        handler: &crate::v2::view::draw::ctx::BActionHandler,
        close: &crate::v2::view::prim::BAction,
    ) -> gpui::Stateful<gpui::Div> {
        let (click_handler, click_action) = (handler.clone(), close.clone());
        let (stray_handler, stray_action) = (handler.clone(), close.clone());
        let stray = self.clone();
        div()
            .id("popover-backdrop")
            .absolute()
            .inset_0()
            .size_full()
            .on_mouse_down(gpui::MouseButton::Left, move |_, w, cx| {
                (std::rc::Rc::clone(&click_handler))(&click_action, w, cx);
            })
            .on_mouse_move(move |e: &gpui::MouseMoveEvent, w, cx| {
                if stray.left(e.position) {
                    (std::rc::Rc::clone(&stray_handler))(&stray_action, w, cx);
                }
            })
    }
}
