//! Paints the drawn scroller (`view::scroll`) over a scrolling container.
//!
//! The container keeps its own layout; it is given a persistent
//! `ScrollHandle` (one per scroll id) and an absolutely placed canvas child.
//! The canvas reads the handle when it paints, so the thumb follows every
//! scroll without the view re-rendering, and brightens while the pointer is
//! over the container (`V2D_SCROLL_MIN_ALPHA` otherwise).

use crate::v2::view::BView;
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use crate::v2::view::scroll;
use gpui::prelude::*;
use gpui::{ScrollHandle, SharedString, Stateful};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static HANDLES: RefCell<HashMap<SharedString, ScrollHandle>> = RefCell::new(HashMap::new());
}

/// The scroll handle for `id`, the same one every frame.
pub(super) fn handle(id: &SharedString) -> ScrollHandle {
    HANDLES.with(|h| h.borrow_mut().entry(id.clone()).or_default().clone())
}

/// Give a scrolling container (already `.id(id)`, `overflow_*_scroll`) its
/// handle and the scroller painted over its trailing edge.
pub(super) fn scroller(element: Stateful<gpui::Div>, id: &SharedString, ctx: &BRenderCtx) -> Stateful<gpui::Div> {
    let handle = handle(id);
    let colour = to_hsla(ctx.style.colors.text);
    let lit = to_hsla(ctx.style.colors.accent);
    // The scroller rides the container's right-hand gutter — the same inner
    // pad the content is already held off the edge by — so it sits in space
    // the content does not use instead of over the top of it. Centre line,
    // not edge inset: the thumb then swells about that line and takes half
    // its new thickness either side.
    let gutter = ctx.style.spacing.pad_md;
    let paint_handle = handle.clone();
    let bar = gpui::canvas(
        |_, _, _| (),
        move |_, _, window, _| {
            let area = paint_handle.bounds();
            let viewport = f32::from(area.size.height);
            let content = viewport + f32::from(paint_handle.max_offset().y);
            let offset = -f32::from(paint_handle.offset().y);
            let Some(t) = scroll::thumb(viewport, content, offset) else { return };
            let near = area.contains(&window.mouse_position());
            let alpha = if near { 1.0 } else { scroll::MIN_ALPHA };
            // One width, pointer or no pointer: the thumb lives inside the
            // gutter and swelling it there would put the extra thickness over
            // the content on one side and the area's border on the other.
            // The pointer changes its colour and its alpha instead. Narrower
            // than the gutter as well, so it clears both its edges.
            let width = scroll::THUMB_WIDTH.min(gutter);
            let centre = area.origin.x + area.size.width - gpui::px(gutter / 2.0);
            let rect = gpui::Bounds::new(
                gpui::point(centre - gpui::px(width / 2.0), area.origin.y + gpui::px(t.start)),
                gpui::size(gpui::px(width), gpui::px(t.len)),
            );
            let mut fill = if near { lit } else { colour };
            fill.a *= 0.35 * alpha;
            window.paint_quad(gpui::fill(rect, fill).corner_radii(gpui::px(width / 2.0)));
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_0();
    element.track_scroll(&handle).child(bar)
}

/// The scroll identity of a region column — named by what it holds, so a
/// column keeps its offset across rebuilds.
pub(super) fn scroll_id(prefix: &str, list: &[BView]) -> gpui::SharedString {
    let tag = list.first().map(first_text).unwrap_or_default();
    gpui::SharedString::from(format!("{prefix}-{tag}"))
}

fn first_text(view: &BView) -> String {
    match view {
        BView::Label { text } | BView::VLabel { text } | BView::Field { text } => text.clone(),
        BView::Button(ctl) => ctl.label.clone().unwrap_or_default(),
        BView::Split { list }
        | BView::Column { list }
        | BView::Row { list } => {
            list.first().map(first_text).unwrap_or_default()
        }
        BView::TableHeader { cell_list } | BView::TableRow { cell_list, .. } => cell_list
            .first()
            .map(|cell| first_text(&cell.view))
            .unwrap_or_default(),
        BView::SortHeader { label, .. } => label.clone(),
        // A region column is named by what it holds, as its panel role was.
        BView::Card(card) if matches!(card.kind, crate::v2::view::prim::BCardKind::Column(_)) => {
            card.body.first().map(first_text).unwrap_or_default()
        }
        BView::Card(card) => card.title().unwrap_or_default().to_string(),
        _ => String::new(),
    }
}
