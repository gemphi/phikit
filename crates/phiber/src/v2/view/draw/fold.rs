//! A folding header's gestures — Blender's `handle_panel_header`
//! (`editors/interface/interface_panel.cc`), for any strip that folds a card:
//!
//! - click: flip this panel (`BFold::flip`);
//! - Ctrl-click: open this panel, close the rest of its region
//!   (`panels_collapse_all`, `BFold::solo`);
//! - press and drag across headers: the first flips, and every header the
//!   pointer crosses takes that same state (`panel_drag_collapse`,
//!   `BFold::sweep_to`).
//!
//! Only listeners live here; the strip's look is `card.rs`'s.

use super::ctx::BRenderCtx;
use crate::v2::view::prim::BFold;
use gpui::prelude::*;
use gpui::{App, ClickEvent, Context, DragMoveEvent, IntoElement, Pixels, Point, Render, Stateful, Window};
use std::rc::Rc;

/// A drag across panel headers, carrying the state it sets them to.
#[derive(Clone, Copy)]
struct BFoldSweep {
    open: bool,
}

/// A sweep draws nothing under the pointer: the headers it flips are the
/// feedback, as in Blender.
struct BSweepGhost;

impl Render for BSweepGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

pub(super) fn gestures(strip: Stateful<gpui::Div>, fold: &BFold, ctx: &BRenderCtx) -> Stateful<gpui::Div> {
    let click = {
        let handler = ctx.handler.clone();
        let click = fold.click();
        let solo = fold.solo();
        move |event: &ClickEvent, window: &mut Window, app: &mut App| {
            let actions = if event.modifiers().control { &solo } else { &click };
            for action in actions {
                (Rc::clone(&handler))(action, window, app);
            }
        }
    };
    // The drag starts on this header: it flips, and carries its new state.
    let start = {
        let handler = ctx.handler.clone();
        let flip = fold.flip();
        move |_: &BFoldSweep, _: Point<Pixels>, window: &mut Window, app: &mut App| {
            (Rc::clone(&handler))(&flip, window, app);
            app.new(|_| BSweepGhost)
        }
    };
    // Any header the sweep passes over takes the sweep's state.
    let over = {
        let handler = ctx.handler.clone();
        let fold = fold.clone();
        move |event: &DragMoveEvent<BFoldSweep>, window: &mut Window, app: &mut App| {
            let target = event.drag(app).open;
            if target != fold.open && event.bounds.contains(&event.event.position) {
                (Rc::clone(&handler))(&fold.sweep_to(target), window, app);
            }
        }
    };
    strip
        .cursor(gpui::CursorStyle::PointingHand)
        .on_click(click)
        .on_drag(BFoldSweep { open: !fold.open }, start)
        .on_drag_move::<BFoldSweep>(over)
}
