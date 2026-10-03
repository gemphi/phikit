//! Dragging a sidebar's inner edge to widen or narrow it.
//!
//! Blender ref: `region_azones_add` — every region edge is a resize handle.
//! The zone/area seams (`zone::seam`) resize the editor grid; this one resizes
//! a panel inside a host or modal, and writes one state key.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use crate::v2::state::BPath;
use gpui::prelude::*;
use gpui::{AnyElement, Context, IntoElement, Pixels, Point, Render, SharedString, Window, div, px};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub use crate::v2::view::size::k_sidebar_width;

/// Live drag of a panel edge: where it started and how wide the panel was.
#[derive(Clone)]
pub struct PanelDrag {
    /// The state key holding the width.
    pub key: String,
    /// The state key that holds whether the panel is dragged shut, and what
    /// it is written shut and open (`BPanelEdge`).
    pub hidden_key: String,
    pub shut: &'static str,
    pub open: &'static str,
    /// `1` when the panel grows with the pointer moving right (its grab
    /// strip on its trailing edge), `-1` when it grows leftwards.
    pub sign: f32,
    /// Below this the panel hides (`UI_UNIT_X`, `view::size::scale`).
    pub collapse: f32,
    pub start_width: f32,
    pub min: f32,
    pub max: f32,
    pub origin: Arc<Mutex<Option<Point<Pixels>>>>,
    pub dragging: Arc<AtomicBool>,
}

impl PanelDrag {
    /// Where the pointer at `now` leaves the panel: its width, or hidden
    /// (Blender's `region_scale_modal`, `view::size::scale`).
    pub fn scaled_at(&self, now: Point<Pixels>) -> crate::v2::view::size::BScaled {
        let mut guard = self.origin.lock().unwrap();
        let origin = *guard.get_or_insert(now);
        crate::v2::view::size::scale(self.start_width, self.sign * f32::from(now.x - origin.x), self.min, self.max, self.collapse)
    }

    /// What the shut key takes at this width.
    pub fn shut_value(&self, scaled: crate::v2::view::size::BScaled) -> &'static str {
        if scaled.hidden { self.shut } else { self.open }
    }
}

/// A sidebar column's grab edge: which edge it is on, and the key a drag
/// shut writes. The file browser's source list grows rightwards and hides
/// through `ctx.sidebar.hidden`; the viewport's N panel, left of its tab
/// strip, grows leftwards and a drag shut closes the region
/// (`part/sidebar/show.rs`).
#[derive(Clone, Debug, PartialEq)]
pub struct BPanelEdge {
    pub shut_key: BPath,
    pub shut: &'static str,
    pub open: &'static str,
    /// The grab strip is on the column's leading (left) edge.
    pub leading: bool,
}

impl BPanelEdge {
    /// A column left of what it resizes against: the trailing edge.
    pub fn trailing() -> Self {
        use crate::v2::state::val::{V_EMPTY, V_TRUE};
        Self { shut_key: crate::v2::view::size::k_sidebar_hidden(), shut: V_TRUE, open: V_EMPTY, leading: false }
    }

    /// A host's sidebar region, left of its tab strip.
    pub fn beside_strip(host: crate::v2::host::BHostType) -> Self {
        use crate::v2::state::val::{V_FALSE, V_TRUE};
        let shut_key = crate::v2::part::sidebar::show::BSidebarShown(host).key();
        Self { shut_key, shut: V_FALSE, open: V_TRUE, leading: true }
    }
}

struct BGhost {
    flag: Arc<AtomicBool>,
}

impl Drop for BGhost {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::Relaxed);
    }
}

impl Render for BGhost {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// The line every boundary shows — a zone's, an area's, a panel's: one
/// seam. `gap − seam_rest_under` thick in the theme's border colour at
/// rest; under the pointer (the `group` the handle joins) it swells to
/// `gap + seam_hover_over` in the theme's accent.
pub struct BSeamLine {
    pub vertical: bool,
    pub group: SharedString,
}

impl BSeamLine {
    pub fn render(self, ctx: &BRenderCtx) -> gpui::Div {
        let rest = px(ctx.style.resize.rest_thickness());
        // Never wider than the band that can be grabbed.
        let hover = px(ctx.style.resize.hover_thickness().min(ctx.style.resize.hit_width));
        let active = to_hsla(ctx.style.resize_seam_active_color());
        let vertical = self.vertical;
        // Nothing at rest: the boundary is already drawn, by each area's own
        // border either side of the gap, and a seam line painted on top of
        // that is a second line in the same 4px. The seam is a pointer
        // affordance, so it appears under the pointer and not before.
        //
        // It keeps its resting *width* so that swelling to `hover` grows it
        // evenly about the gap's midline — the band is centred on that
        // midline by `resize.inset`, and the line is centred in the band.
        let line = div().group_hover(self.group, move |s| {
            let s = s.bg(active);
            if vertical { s.w(hover) } else { s.h(hover) }
        });
        if vertical { line.w(rest).h_full() } else { line.h(rest).w_full() }
    }
}

/// A seam's zero-footprint overlay: `hit_width` wide, centred on the
/// boundary gap so the seam costs no layout space at all. `leading` places it
/// on the parent's trailing edge (a panel resizing itself); otherwise it hangs
/// off the leading edge (a slot resizing the boundary before it, as
/// `zone::seam` does). One geometry, so a panel edge and an area edge are the
/// same seam.
pub fn overlay(vertical: bool, leading: bool, ctx: &BRenderCtx) -> gpui::Div {
    let hit = px(ctx.style.resize.hit_width);
    let inset = px(ctx.style.resize.inset);
    let element = div().absolute();
    match (vertical, leading) {
        (true, true) => element.top_0().bottom_0().left(inset).w(hit),
        (true, false) => element.top_0().bottom_0().right(inset).w(hit),
        (false, true) => element.left_0().right_0().top(inset).h(hit),
        (false, false) => element.left_0().right_0().bottom(inset).h(hit),
    }
}

/// The grab strip on a panel's edge (`BPanelEdge`).
pub fn handle(key: BPath, width: f32, edge: BPanelEdge, ctx: &BRenderCtx) -> AnyElement {
    let group = SharedString::from("panel-resize");
    let line = BSeamLine { vertical: true, group: group.clone() }.render(ctx);
    let payload = PanelDrag {
        key: key.encode(),
        hidden_key: edge.shut_key.encode(),
        shut: edge.shut,
        open: edge.open,
        sign: if edge.leading { -1.0 } else { 1.0 },
        collapse: ctx.style.dimensions.row_height,
        start_width: width,
        min: ctx.style.dimensions.sidebar_min_width,
        max: ctx.style.dimensions.panel_width * 3.0,
        origin: Arc::new(Mutex::new(None)),
        dragging: Arc::new(AtomicBool::new(false)),
    };
    let grab = div()
        .id(SharedString::from("panel-resize-handle"))
        .debug_selector(|| "panel-resize-handle".to_string())
        .group(group)
        .flex()
        .items_center()
        .justify_center()
        .size_full()
        .occlude()
        .cursor(gpui::CursorStyle::ResizeLeftRight)
        .child(line)
        .on_drag(payload, |drag: &PanelDrag, _, _, cx| {
            drag.dragging.store(true, Ordering::Relaxed);
            let flag = drag.dragging.clone();
            cx.stop_propagation();
            cx.new(move |_| BGhost { flag })
        });
    // The seam floats over the boundary gap instead of widening it, so the
    // distance between a sidebar and the pane beside it is the same 4px a
    // zone boundary uses (`tests/chrome_geometry.rs`).
    overlay(true, edge.leading, ctx).child(grab).into_any_element()
}
