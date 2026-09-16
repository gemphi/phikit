//! Host walker — packs regions by placement (Blender's `screen_geom.cc`):
//! Top-placed regions (Header/ToolHeader) stack on top, Bottom-placed
//! (Footer/Execute) on the bottom, and the middle band lays Left-placed
//! regions (NavBar/Tools) and Right-placed regions (UI/Channels) beside the
//! flex Center region horizontally. Sets `ctx.host` so descendant walkers
//! blend with the host pane. The wrapper is layout-only: painted regions or
//! specialized content own surfaces, and adjacent bands allocate no gap.

use super::{
    ctx::BRenderCtx,
    region,
    surface::{
        BCellPosition, BChildCornerSupport, BCorners, BRowPosition, BSurfaceOpacity,
        child_surface_plan,
    },
    walk,
};
use crate::v2::host::BHostType;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::render::{BPlacement, BRegionLayout};
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

fn placement(v: &BView) -> BPlacement {
    match v {
        BView::Region { kind, .. } => kind.placement(),
        _ => BPlacement::Flex,
    }
}

pub fn view(kind: BHostType, children: &[BView], c: &BRenderCtx) -> AnyElement {
    let inner = c.with_host(kind);
    let mut top = Vec::new();
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut center = Vec::new();
    let mut bottom = Vec::new();
    for ch in children {
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
    let host_role_radius = c.style.radius_for(BStyleRole::Host);
    let area_plan = child_surface_plan(
        c.corners,
        BSurfaceOpacity::Opaque,
        BChildCornerSupport::Propagated,
        c.style.radius_for(BStyleRole::Area),
        c.style.strokes.border,
    );
    let host_radius = if area_plan.corners.any() {
        host_role_radius.min(area_plan.radius)
    } else {
        host_role_radius
    };

    let mut el = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .size_full()
        .rounded(px(host_radius));

    let mut row = 0usize;
    for ch in top {
        let ownership = BCorners::ALL.assign_row(BRowPosition::at(row, rows), child_opacity(ch));
        el = el.child(walk(ch, &inner.with_corners(ownership.child)));
        row += 1;
    }

    // Middle band: [Left rail | Center | Right rail] remains horizontal; the
    // transparent wrapper around a center region claims no surface corners.
    if has_band {
        let cells = left.len() + center.len() + right.len();
        let mut column = 0usize;
        let mut band = div().flex().flex_row().flex_1().min_h_0().w_full();
        for &ch in &left {
            let ownership = BCorners::ALL.assign_cell(
                BCellPosition::at(row, rows, column, cells),
                child_opacity(ch),
            );
            band = band.child(walk(ch, &inner.with_corners(ownership.child)));
            column += 1;
        }
        for &ch in &center {
            let ownership = BCorners::ALL.assign_cell(
                BCellPosition::at(row, rows, column, cells),
                child_opacity(ch),
            );
            band = band.child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .h_full()
                    .child(walk(ch, &inner.with_corners(ownership.child))),
            );
            column += 1;
        }
        for &ch in &right {
            let ownership = BCorners::ALL.assign_cell(
                BCellPosition::at(row, rows, column, cells),
                child_opacity(ch),
            );
            band = band.child(walk(ch, &inner.with_corners(ownership.child)));
            column += 1;
        }
        el = el.child(band);
        row += 1;
    }

    for ch in bottom {
        let ownership = BCorners::ALL.assign_row(BRowPosition::at(row, rows), child_opacity(ch));
        el = el.child(walk(ch, &inner.with_corners(ownership.child)));
        row += 1;
    }
    el.into_any_element()
}
