//! How a plane is drawn, and how it is grabbed.
//!
//! The square is a grid of cells, each filled with the actual colour at its own
//! point, rather than stacked gradients. Every picker approximates the shading that
//! way because a 2-stop linear gradient cannot express a saturation sweep crossed
//! with a lightness one — and an approximation here would mean the square showed
//! one colour and set another. A grid is exact: a cell's fill and the write for a
//! pointer dropped on it come from the same `color_at`, so they cannot disagree.
//! The cost is visible banding at `CELLS` steps, a fair trade for a square that
//! does not lie.

use super::BPlane;
use crate::v2::theme::{BColor, BStyleRole};
use crate::v2::view::draw::ctx::BRenderCtx;
use crate::v2::view::draw::flex::{BEdge, BFill, BFlex, BFlexed, BLen, BRadius};
use crate::v2::view::draw::track::BTrackGhost;
use gpui::prelude::*;
use gpui::{App, Div, DragMoveEvent, Pixels, Point, Stateful, Window};
use std::rc::Rc;

/// Cells per side. Enough that the square reads as continuous, few enough that one
/// picker is 400 small divs and not forty thousand.
pub(super) const CELLS: usize = 20;

/// The parts of a plane.
enum BPlanePart {
    /// The square itself.
    Square,
    /// One row of cells.
    Row,
    /// One cell, filled with the colour at its own point.
    Cell(BColor),
}

impl BFlexed for BPlanePart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match self {
            // `fills()` for the width the popover offers, `keeps_size()` so that
            // width is all it takes from the arrangement: in a column the shrink
            // `fills()` sets applies to the HEIGHT, and a popover opening low in
            // the window has less room below the anchor than the picker wants. The
            // square was the one child with a large basis and a shrink, so it
            // absorbed the whole shortfall and came out 2px tall — its two borders
            // with nothing between them. Blender does not have the question:
            // `colorpicker_square` gives the square a flat `PICKER_H` and the
            // region is sized to its contents, so the popup moves to fit on screen
            // and the square never gives up a pixel.
            Self::Square => BFlex::column()
                .fills()
                .keeps_size()
                .min_width(BLen::Zero)
                .height(BLen::Measured(SQUARE_HEIGHT))
                .min_height(BLen::Measured(SQUARE_HEIGHT))
                .radius(BRadius::Role(BStyleRole::Field))
                .border(BEdge::All)
                .overflow(crate::v2::view::draw::flex::BOverflow::Clip),
            Self::Row => BFlex::row().width(BLen::Full).height(BLen::Share(1.0 / CELLS as f32)),
            Self::Cell(color) => BFlex::cell()
                .width(BLen::Share(1.0 / CELLS as f32))
                .height(BLen::Full)
                .fill(BFill::SwatchHsl(*color)),
        }
    }
}

/// How tall the square is drawn. A colour square is read as a square, and the
/// width it gets is whatever the panel gives it, so this is the one measure it
/// states for itself.
const SQUARE_HEIGHT: f32 = 120.0;

/// The plane, drawn and draggable.
pub fn view(plane: &BPlane, ctx: &BRenderCtx) -> gpui::AnyElement {
    let id = gpui::SharedString::from(format!("plane-{}", plane.key.encode()));
    let mut square = BPlanePart::Square.flex(ctx).div(ctx).id(id.clone()).debug_selector({
        let id = id;
        move || id.to_string()
    });
    for row in 0..CELLS {
        let down = (row as f32 + 0.5) / CELLS as f32;
        let mut line = BPlanePart::Row.flex(ctx).div(ctx);
        for column in 0..CELLS {
            let across = (column as f32 + 0.5) / CELLS as f32;
            line = line.child(BPlanePart::Cell(plane.color_at(across, down)).flex(ctx).div(ctx));
        }
        square = square.child(line);
    }
    draggable(square, plane.clone(), ctx).into_any_element()
}

/// Make `element` a draggable plane.
fn draggable(element: Stateful<Div>, plane: BPlane, ctx: &BRenderCtx) -> Stateful<Div> {
    let start = move |_: &BPlane, _: Point<Pixels>, _window: &mut Window, app: &mut App| {
        app.new(|_| BTrackGhost)
    };
    let over = {
        let handler = ctx.handler.clone();
        move |event: &DragMoveEvent<BPlane>, window: &mut Window, app: &mut App| {
            let plane = event.drag(app).clone();
            (Rc::clone(&handler))(&plane.adjust_at(event.bounds, event.event.position), window, app);
        }
    };
    element
        .cursor(gpui::CursorStyle::Crosshair)
        .on_drag(plane, start)
        .on_drag_move::<BPlane>(over)
}
