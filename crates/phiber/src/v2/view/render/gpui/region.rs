//! Region walker — orientation + fixed size from resolved UI style. Painted
//! header/strip regions receive explicit host-corner ownership; ordinary
//! layout regions are transparent so their pane or specialized content owns
//! the visible surface.

use super::{
    color::{resolved_gradient, to_hsla},
    ctx::BRenderCtx,
    surface::{BChildCornerSupport, BSurfaceOpacity, child_surface_plan, compatible_inner_radius},
    walk,
};
use crate::v2::host::BHostType;
use crate::v2::part::strip;
use crate::v2::region::BRegionType;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::render::{BAxis, BPlacement, BRegionLayout};
use gpui::prelude::*;
use gpui::{AnyElement, Background, div, px};

pub(super) const fn surface_opacity(kind: BRegionType, host: BHostType) -> BSurfaceOpacity {
    match kind {
        BRegionType::Header | BRegionType::ToolHeader if !matches!(host, BHostType::Topbar) => {
            BSurfaceOpacity::Opaque
        }
        BRegionType::NavBar
        | BRegionType::Channels
        | BRegionType::Scrubbing
        | BRegionType::AssetShelfHeader => BSurfaceOpacity::Opaque,
        _ => BSurfaceOpacity::Transparent,
    }
}

fn region_bg(kind: BRegionType, c: &BRenderCtx) -> Background {
    match kind {
        // All vertical icon rails (Properties navbar, activity bar, other
        // navbars) are transparent. Only hovered/active items carry paint so
        // the strip reads as part of the enclosing host area, not a separate
        // chrome piece.
        BRegionType::NavBar => gpui::transparent_black().into(),
        BRegionType::Channels
        | BRegionType::Scrubbing
        | BRegionType::AssetShelfHeader => resolved_gradient(c.style.strip_gradient(c.host)),
        BRegionType::Header | BRegionType::ToolHeader => {
            resolved_gradient(c.style.host_header_surface(c.host))
        }
        _ => gpui::transparent_black().into(),
    }
}

pub fn view(kind: BRegionType, children: &[BView], c: &BRenderCtx) -> AnyElement {
    let axis = kind.axis();
    let placement = kind.placement();
    let mut el = match (axis, placement) {
        (BAxis::Row, BPlacement::Top | BPlacement::Bottom) => {
            // Header rows: flex_row, items_center. No justify_between —
            // the Center pane has flex_1 which naturally pushes Left and
            // Right to opposite ends without overlap.
            div().flex().flex_row().items_center()
        }
        (BAxis::Row, _) => div().flex().flex_row(),
        (BAxis::Column, _) => div().flex().flex_col().items_stretch(),
    }
    .bg(region_bg(kind, c))
    .gap(px(c.style.spacing.gap_xs));

    // Vertical rails use pad_md vertical padding so the first tab icon aligns
    // with the first accordion header in the adjacent content region. Header
    // strips remain vertically centred in their fixed height.
    el = if kind == BRegionType::NavBar && c.host == BHostType::Buts {
        // Properties navbar (vertical strip): flush horizontally so rail tabs
        // meet both edges of the rail (layout tests pin this at x=0). Top
        // padding matches the content region's pad_sm so the first icon
        // sits level with the first accordion header. Bottom padding balances
        // the top so the flex spacer centres the chevron correctly.
        el.pt(px(c.style.spacing.pad_sm))
            .pb(px(c.style.spacing.pad_sm))
            .gap(px(c.style.spacing.gap_none))
    } else if matches!(kind, BRegionType::NavBar | BRegionType::Tools) {
        el.py(px(c.style.spacing.pad_md))
    } else if matches!(kind, BRegionType::Header | BRegionType::ToolHeader) {
        el.px(px(c.style.spacing.pad_md))
    } else if c.host == BHostType::Buts && kind == BRegionType::Center {
        // Properties content: small top/bottom pad so the first and last
        // accordion don't touch the edges. Right-edge padding gives the
        // content column a small gutter against the pane edge (keeps the
        // last property row from butting against the border).
        el.py(px(c.style.spacing.pad_sm))
            .pr(px(c.style.spacing.pad_sm))
    } else if kind == BRegionType::Center && c.host.fills_content() {
        // Content-fill hosts (3D viewport + every node graph + image / UV /
        // seq / clip + timelines): the content region paints wall-to-wall.
        // No outer padding —
        // the viewport's blue clear + the graph's grid must reach every
        // edge of the pane. Overlays (gizmos, toolbar, sidebar, HUD) are
        // absolute-positioned so they don't need a padded parent.
        el
    } else {
        el.p(px(c.style.spacing.pad_md))
    };

    // The header owns the only horizontal separator at the header/content
    // seam. Host stacking allocates no gap and content regions add no top rule.
    if matches!(kind, BRegionType::Header | BRegionType::ToolHeader) {
        el = el
            .border_b(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border));
    }

    let fixed_size = match kind {
        BRegionType::Header | BRegionType::ToolHeader | BRegionType::AssetShelfHeader => {
            Some(c.style.dimensions.header_height)
        }
        BRegionType::Footer | BRegionType::Execute => Some(c.style.dimensions.footer_height),
        BRegionType::Scrubbing => Some(c.style.dimensions.scrubbing_height),
        BRegionType::AssetShelf => Some(c.style.dimensions.asset_shelf_height),
        BRegionType::Tools => {
            // The tool rail shares the strip expand key: widened to the
            // sidebar label width when expanded (Blender's toolbar flyout
            // shows labels), metric width when collapsed.
            if strip::is_expanded(c.s, c.host, "tool") {
                Some(c.style.dimensions.sidebar_min_width)
            } else {
                Some(c.style.dimensions.toolbar_width)
            }
        }
        BRegionType::NavBar => {
            // Every navbar rail (Buts Properties, Prefs) shares the same
            // expand key — widened to show icon + label when expanded,
            // metric icon-only width when collapsed.
            if strip::is_expanded(c.s, c.host, "navbar") {
                Some(c.style.dimensions.sidebar_min_width)
            } else {
                Some(c.style.dimensions.navbar_width)
            }
        }
        BRegionType::UI => Some(c.style.dimensions.panel_width),
        BRegionType::Channels => Some(c.style.dimensions.channels_width),
        BRegionType::ToolProps => Some(c.style.dimensions.tool_props_width),
        _ => None,
    };
    el = if let Some(size) = fixed_size {
        match axis {
            BAxis::Row => el.h(px(size)).w_full(),
            BAxis::Column if matches!(kind, BRegionType::NavBar | BRegionType::Tools) => {
                el.w(px(size)).flex_shrink_0().h_full().min_h_0()
            }
            BAxis::Column => el.min_w(px(size)).w_full().h_full().min_h_0(),
        }
    } else {
        // Every canvas host (View3D + node graphs + image/UV/seq/clip)
        // makes its Center region a `.relative()` container so the absolute-
        // positioned Toolbar / Sidebar panes (see pane.rs) anchor to it
        // instead of falling through to the window root.
        if kind == BRegionType::Center && c.host.floats_side_panels() {
            // Only floating-side-panel hosts need a relative container so the
            // absolute Toolbar / Sidebar overlays anchor to the center region.
            el.flex_1().min_h_0().h_full().w_full().relative()
        } else {
            el.flex_1().min_h_0().h_full().w_full()
        }
    };

    let area_inner_radius =
        compatible_inner_radius(c.style.radius_for(BStyleRole::Area), c.style.strokes.border);
    let host_radius = c.style.radius_for(BStyleRole::Host).min(area_inner_radius);
    let plan = child_surface_plan(
        c.corners,
        surface_opacity(kind, c.host),
        BChildCornerSupport::Propagated,
        host_radius,
        0.0,
    );
    if plan.corners.any() {
        let radius = px(c.style.radius_for(BStyleRole::Region).min(plan.radius));
        if plan.corners.tl {
            el = el.rounded_tl(radius);
        }
        if plan.corners.tr {
            el = el.rounded_tr(radius);
        }
        if plan.corners.bl {
            el = el.rounded_bl(radius);
        }
        if plan.corners.br {
            el = el.rounded_br(radius);
        }
    }

    let dbg = format!("region-{:?}-{:?}", kind, c.host);
    let mut el = el
        .id(gpui::SharedString::from(dbg.clone()))
        .debug_selector(move || dbg.clone());
    // Tag the context with this region kind so pane children can distinguish
    // a header pane (content hugs, Left/Center/Right tri-cell) from a
    // content-region Center pane (stretches to fill).
    let child_ctx = c.with_region(kind);
    for ch in children {
        el = el.child(walk(ch, &child_ctx));
    }
    el.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{BSurfaceOpacity, surface_opacity};
    use crate::v2::host::BHostType;
    use crate::v2::region::BRegionType;

    #[test]
    fn only_intentionally_painted_regions_claim_host_corners() {
        for kind in [
            BRegionType::Center,
            BRegionType::Footer,
            BRegionType::Tools,
            BRegionType::ToolProps,
            BRegionType::UI,
            BRegionType::AssetShelf,
        ] {
            assert_eq!(
                surface_opacity(kind, BHostType::View3D),
                BSurfaceOpacity::Transparent
            );
        }
        assert_eq!(
            surface_opacity(BRegionType::Header, BHostType::View3D),
            BSurfaceOpacity::Opaque
        );
        assert_eq!(
            surface_opacity(BRegionType::Header, BHostType::Topbar),
            BSurfaceOpacity::Transparent
        );
        assert_eq!(
            surface_opacity(BRegionType::NavBar, BHostType::Buts),
            BSurfaceOpacity::Opaque
        );
    }
}
