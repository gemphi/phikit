//! Host walker — packs regions by placement (Blender's `screen_geom.cc`):
//! Top-placed regions (Header/ToolHeader) stack on top, Bottom-placed
//! (Footer/Execute) on the bottom, and the middle band lays Left-placed
//! regions (NavBar/Tools) and Right-placed regions (UI/Channels) beside the
//! flex Center region horizontally. Sets `ctx.host` so descendant walkers
//! blend with the host pane. The wrapper is layout-only: painted regions or
//! specialized content own surfaces, and adjacent bands allocate no gap.

use super::ctx::BRenderCtx;
use super::region;
use super::surface::{BCellPosition, BChildCornerSupport, BCorners, BRowPosition, BSurfaceOpacity, child_surface_plan};
use super::walk;
use super::flex::{BFlex, BFlexed, BLen, BRadius};
use crate::v2::host::BHostType;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::layout::{BPlacement, BRegionLayout};
use gpui::prelude::*;
use gpui::AnyElement;

/// The editor under the pointer — Blender's `CTX_wm_area` for a key press:
/// a key goes to the keymap of the region the pointer is over first
/// (`wm_event_do_handlers`), so G over the Timeline moves markers and F2
/// there renames one. Every host records itself as the pointer moves over
/// it; the keymap reads it (`app::keymap`).
pub struct BPointer;

thread_local! {
    static OVER: std::cell::Cell<Option<BHostType>> = const { std::cell::Cell::new(None) };
}

impl BPointer {
    pub fn host() -> Option<BHostType> {
        OVER.get()
    }

    pub fn set(host: Option<BHostType>) {
        OVER.set(host);
    }
}

fn placement(v: &BView) -> BPlacement {
    match v {
        BView::Region { kind, .. } => kind.placement(),
        _ => BPlacement::Flex,
    }
}

/// The pieces a host is built from: the column that stacks its bands, the
/// middle band that lays side regions beside the centre, and the cell that
/// holds a centre region in that band.
enum BHostPart {
    /// The host itself, rounded to the radius its area leaves it.
    Frame { radius: f32 },
    Band,
    CenterCell,
}

impl BFlexed for BHostPart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match *self {
            Self::Frame { radius } => BFlex::column()
                .fills()
                .min_height(BLen::Zero)
                .full()
                .radius(BRadius::All(radius)),
            Self::Band => BFlex::row().fills().min_height(BLen::Zero).width(BLen::Full),
            Self::CenterCell => BFlex::column().fills().unclamped().height(BLen::Full),
        }
    }
}

pub fn view(kind: BHostType, list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    let inner = ctx.with_host(kind);
    let mut top = Vec::new();
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut center = Vec::new();
    let mut bottom = Vec::new();
    for ch in list {
        if ch.is_empty() {
            continue;
        }
        match placement(ch) {
            BPlacement::Top => top.push(ch),
            BPlacement::Bottom => bottom.push(ch),
            BPlacement::Left => left.push(ch),
            BPlacement::Right => right.push(ch),
            BPlacement::Flex => center.push(ch),
        }
    }

    let child_opacity = |child: &BView| match child {
        BView::Region {
            kind: region_kind, ..
        } => region::surface_opacity(*region_kind, kind),
        _ => BSurfaceOpacity::Transparent,
    };
    let has_band = !left.is_empty() || !right.is_empty() || !center.is_empty();
    let rows = top.len() + usize::from(has_band) + bottom.len();
    let host_role_radius = ctx.style.radius_for(BStyleRole::Host);
    let area_plan = child_surface_plan(
        ctx.corners,
        BSurfaceOpacity::Opaque,
        BChildCornerSupport::Propagated,
        ctx.style.radius_for(BStyleRole::Area),
        ctx.style.strokes.border,
    );
    let host_radius = if area_plan.corners.any() {
        host_role_radius.min(area_plan.radius)
    } else {
        host_role_radius
    };

    let mut element = BHostPart::Frame { radius: host_radius }.flex(ctx).div(ctx);

    let mut row = 0usize;
    for ch in top {
        let ownership = BCorners::ALL.assign_row(BRowPosition::at(row, rows), child_opacity(ch));
        element = element.child(walk(ch, &inner.with_corners(ownership.child)));
        row += 1;
    }

    // Middle band: [Left rail | Center | Right rail] remains horizontal; the
    // transparent wrapper around a center region claims no surface corners.
    if has_band {
        let cell_list = left.len() + center.len() + right.len();
        let mut column = 0usize;
        let mut band = BHostPart::Band.flex(ctx).div(ctx);
        for &ch in &left {
            let ownership = BCorners::ALL.assign_cell(
                BCellPosition::at(row, rows, column, cell_list),
                child_opacity(ch),
            );
            band = band.child(walk(ch, &inner.with_corners(ownership.child)));
            column += 1;
        }
        for &ch in &center {
            let ownership = BCorners::ALL.assign_cell(
                BCellPosition::at(row, rows, column, cell_list),
                child_opacity(ch),
            );
            band = band.child(
                BHostPart::CenterCell
                    .flex(ctx)
                    .div(ctx)
                    .child(walk(ch, &inner.with_corners(ownership.child))),
            );
            column += 1;
        }
        for &ch in &right {
            let ownership = BCorners::ALL.assign_cell(
                BCellPosition::at(row, rows, column, cell_list),
                child_opacity(ch),
            );
            band = band.child(walk(ch, &inner.with_corners(ownership.child)));
            column += 1;
        }
        element = element.child(band);
        row += 1;
    }

    for ch in bottom {
        let ownership = BCorners::ALL.assign_row(BRowPosition::at(row, rows), child_opacity(ch));
        element = element.child(walk(ch, &inner.with_corners(ownership.child)));
        row += 1;
    }
    element.on_mouse_move(move |_, _, _| BPointer::set(Some(kind))).into_any_element()
}
