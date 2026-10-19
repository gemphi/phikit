//! Region walker — orientation + fixed size from resolved UI style. Painted
//! header/strip regions receive explicit host-corner ownership; ordinary
//! layout regions are transparent so their pane or specialized content owns
//! the visible surface.

use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BPlace, BRadius, BSpace};
use super::ctx::BRenderCtx;
use super::surface::{BChildCornerSupport, BSurfaceOpacity, child_surface_plan, compatible_inner_radius};
use super::walk;
mod pad;
use crate::v2::host::BHostType;
use crate::v2::part::strip;
use crate::v2::region::BRegionType;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::layout::{BAxis, BPlacement, BRegionLayout};
use gpui::prelude::*;
use gpui::AnyElement;

pub(super) const fn surface_opacity(kind: BRegionType, host: BHostType) -> BSurfaceOpacity {
    match kind {
        BRegionType::Header | BRegionType::ToolHeader | BRegionType::Footer | BRegionType::Execute
            if !matches!(host, BHostType::Topbar) =>
        {
            BSurfaceOpacity::Opaque
        }
        BRegionType::Scrubbing | BRegionType::AssetShelfHeader => BSurfaceOpacity::Opaque,
        // A content region paints nothing of its own, but in a content-fill
        // host it holds the canvas — and the canvas is opaque and reaches the
        // host's corners. Corner ownership has to follow what actually paints
        // there, not what the region's own background is: reading the region
        // as transparent handed the canvas no corners at all, so it squared
        // off the zone it sits in however carefully the canvas rounded itself.
        BRegionType::Center if host.fills_content() => BSurfaceOpacity::Opaque,
        _ => BSurfaceOpacity::Transparent,
    }
}

fn region_fill(kind: BRegionType) -> BFill {
    match kind {
        // The horizontal strips under a header — the scrub strip, the
        // asset shelf's tabs — take the strip shade.
        BRegionType::Scrubbing | BRegionType::AssetShelfHeader => BFill::HostStrip,
        // A footer is a strip like the header, on the same surface
        // (Blender draws both in `TH_HEADER`) — and so is the file
        // browser's execute bar: a strip of field and buttons, one surface
        // with the header strips over it.
        BRegionType::Header | BRegionType::ToolHeader | BRegionType::Footer | BRegionType::Execute => BFill::HostHeader,
        // Every vertical strip — the Properties navbar, the tool rail, the
        // editor side rail — takes the rail surface, which is the body of
        // the area it sits in (`rail_surface`): the strip has no colour of
        // its own, it shares the column's. What marks the Properties strip
        // is its selected tab, painted as the accordion heading beside it
        // (`tab_active_gradient`), not a shade behind the whole strip.
        BRegionType::NavBar | BRegionType::Tools => BFill::HostRail,
        // A content region paints nothing: the area's body under it is its
        // surface.
        _ => BFill::Clear,
    }
}

/// True when a region is a content header, and so closes with a rule along
/// its bottom edge. The global bars are not: the top bar draws its one line
/// on the area itself (`area`), which is the window's own edge rather than a
/// boundary inside an editor, and the status bar heads nothing.
fn ruled_header(kind: BRegionType, host: BHostType) -> bool {
    matches!(kind, BRegionType::Header | BRegionType::ToolHeader) && !host.pins_header_cells()
}

/// True when this is the status bar's header while a file opens: it then
/// holds the loading panel, so the region content-sizes (at least one bar
/// tall, growing to the panel) instead of clipping to one line or scrolling
/// sideways in its strip. See `part::statusbar` and `region::BRegionItem`.
fn header_loads(kind: BRegionType, ctx: &BRenderCtx) -> bool {
    kind == BRegionType::Header
        && ctx.host == BHostType::Statusbar
        && crate::v2::scene::view_load_progress(ctx.state).is_some()
}

/// True when a region's content is a vertical rail. Rails are laid out the
/// same way wherever they appear — the Activity bar, the tool strip, the
/// Properties navbar and the editor side rail are one element, not four.
fn rail_region(kind: BRegionType, host: BHostType) -> bool {
    matches!(kind, BRegionType::NavBar)
        || (kind == BRegionType::Tools && !host.tools_hold_panels())
        || (kind == BRegionType::Center && host == BHostType::Activity)
}

/// A region as drawn. Its layout follows from its kind, the host it sits in
/// (`c.host`), whether its rail is expanded, and which of the host's corners
/// it owns (`c.corners`).
struct BRegionFrame {
    kind: BRegionType,
}

impl BFlexed for BRegionFrame {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        let kind = self.kind;
        let axis = kind.axis();
        // The top bar wraps when the window is too narrow for its three
        // cells in one line (`filter::topbar_wraps`): the left cell keeps
        // the first line and the workspace tabs and the right-hand controls
        // go under it, the region growing a line. An editor's header does
        // not wrap — Blender clips and scrolls it — so an area's chrome
        // keeps its height whatever its width.
        let wraps = kind == BRegionType::Header && ctx.host.pins_header_cells() && crate::v2::part::filter::topbar_wraps(ctx.state);
        // The status bar's header, while a file opens, grows to its loading
        // panel the way a wrapping header grows to its lines — content-sized
        // with a one-bar floor — but keeps its ordinary single-line row flex.
        let grows = wraps || header_loads(kind, ctx);
        let flex = match (axis, kind.placement()) {
            // Header rows: the three panes sit at the start, the centre and
            // the end. `Between` states that directly rather than relying on
            // the Center pane's growth to push the other two apart — a host
            // with nothing in its middle paints no Center pane at all, which
            // left its right-hand controls stranded mid-row.
            (BAxis::Row, BPlacement::Top | BPlacement::Bottom) if wraps => {
                BFlex::row().wraps().align(BAlign::Center).justify(BJustify::Between)
            }
            (BAxis::Row, BPlacement::Top | BPlacement::Bottom) => {
                BFlex::row().align(BAlign::Center).justify(BJustify::Between)
            }
            (BAxis::Row, _) => BFlex::row(),
            (BAxis::Column, _) => BFlex::column().align(BAlign::Stretch),
        }
        .fill(region_fill(kind))
        .gap(BSpace::GapXs);

        let flex = pad::padded(flex, kind, ctx);

        let expanded = |key| strip::is_expanded(ctx.state, ctx.host, key);
        let fixed = match kind {
            BRegionType::Header | BRegionType::ToolHeader | BRegionType::AssetShelfHeader => {
                Some(BDim::HeaderHeight)
            }
            BRegionType::Footer => Some(BDim::FooterHeight),
            // The execute bar holds a field and buttons, so it is a strip
            // like a header; the footer's height is for read-outs.
            BRegionType::Execute => Some(BDim::HeaderHeight),
            BRegionType::Scrubbing => Some(BDim::ScrubbingHeight),
            BRegionType::AssetShelf => Some(BDim::AssetShelfHeight),
            // The tool rail and every navbar share the strip-expand keys:
            // widened to show labels when expanded (Blender's toolbar
            // flyout), a rail wide when collapsed.
            // A tools column of panels (the file browser's source list) is
            // as wide as the user dragged it, or a panel's width until then.
            BRegionType::Tools if ctx.host.tools_hold_panels() => None,
            BRegionType::Tools if expanded("tool") => Some(BDim::SidebarMinWidth),
            BRegionType::Tools => Some(BDim::ToolbarWidth),
            BRegionType::NavBar if expanded("navbar") => Some(BDim::SidebarMinWidth),
            BRegionType::NavBar => Some(BDim::NavbarWidth),
            BRegionType::UI => Some(BDim::PanelWidth),
            BRegionType::Channels => Some(BDim::ChannelsWidth),
            BRegionType::ToolProps => Some(BDim::ToolPropsWidth),
            _ => None,
        };
        if kind == BRegionType::Tools && ctx.host.tools_hold_panels() {
            // Dragged shut: the region keeps only the arrow that restores it.
            if crate::v2::view::size::sidebar_hidden(ctx.state) {
                return flex
                    .width(BLen::Dim(BDim::RowHeight))
                    .keeps_size()
                    .height(BLen::Full)
                    .min_height(BLen::Zero);
            }
            return flex
                .width(BLen::Measured(tools_column_width(ctx)))
                .keeps_size()
                .height(BLen::Full)
                .min_height(BLen::Zero);
        }
        // A header's rule is drawn on its bottom edge, and a border comes out
        // of the box — so the strip is a rule taller than the module and the
        // band above the line is still exactly `header_height`. Without this
        // every cap, field and read-out in every header rides a pixel high
        // (`chrome_geometry`).
        let ruled = ruled_header(kind, ctx.host);
        let ruled_height = ctx.style.dimensions.header_height + ctx.style.strokes.border;
        let flex = match (fixed, axis) {
            // A wrapping header is at least one line tall and grows a line
            // per wrap.
            (Some(_), BAxis::Row) if ruled && grows => {
                flex.min_height(BLen::Measured(ruled_height)).width(BLen::Full).keeps_size()
            }
            (Some(_), BAxis::Row) if ruled => flex.height(BLen::Measured(ruled_height)).width(BLen::Full),
            (Some(dim), BAxis::Row) if grows => flex.min_height(BLen::Dim(dim)).width(BLen::Full).keeps_size(),
            (Some(dim), BAxis::Row) => flex.height(BLen::Dim(dim)).width(BLen::Full),
            (Some(dim), BAxis::Column)
                if matches!(kind, BRegionType::NavBar | BRegionType::Tools) =>
            {
                flex.width(BLen::Dim(dim))
                    .keeps_size()
                    .height(BLen::Full)
                    .min_height(BLen::Zero)
            }
            (Some(dim), BAxis::Column) => flex
                .min_width(BLen::Dim(dim))
                .width(BLen::Full)
                .height(BLen::Full)
                .min_height(BLen::Zero),
            (None, _) => {
                let flex = flex.fills().min_height(BLen::Zero).full();
                // A floating-side-panel host's centre is the anchor its
                // absolute toolbar and sidebar position against; without it
                // they fell through to the window root.
                if kind == BRegionType::Center && ctx.host.floats_side_panels() {
                    flex.place(BPlace::Anchor)
                } else {
                    flex
                }
            }
        };

        let area_inner_radius =
            compatible_inner_radius(ctx.style.radius_for(BStyleRole::Area), ctx.style.strokes.border);
        let host_radius = ctx.style.radius_for(BStyleRole::Host).min(area_inner_radius);
        let plan = child_surface_plan(
            ctx.corners,
            surface_opacity(kind, ctx.host),
            BChildCornerSupport::Propagated,
            host_radius,
            0.0,
        );
        if plan.corners.any() {
            flex.radius(BRadius::Corners {
                tl: plan.corners.tl,
                tr: plan.corners.tr,
                bl: plan.corners.bl,
                br: plan.corners.br,
                radius: ctx.style.radius_for(BStyleRole::Region).min(plan.radius),
            })
        } else {
            flex
        }
    }
}

/// The width of a Tools region that holds panels (the file browser's source
/// list): the column the user dragged, or the sidebar minimum, plus the
/// region's gutter either side. The path strip's left pane takes the same
/// width, so its refresh ends where the list does and the directory field
/// starts where the file list does (`pane`).
pub(super) fn tools_column_width(ctx: &BRenderCtx) -> f32 {
    let width = crate::v2::view::size::sidebar_column_width(ctx.state, ctx.style.sidebar_min_width(), ctx.style.dimensions.sidebar_min_width);
    // The column and the one pad between it and the area edge; the region
    // beside it pads the inner edge (`BRegionFrame`).
    width + ctx.style.spacing.pad_md
}

pub fn view(kind: BRegionType, list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    let frame = BRegionFrame { kind }.flex(ctx);
    let dbg = format!("region-{:?}-{:?}", kind, ctx.host);
    let id = gpui::SharedString::from(dbg.clone());
    // A header strip too narrow for what it holds scrolls sideways under the
    // wheel and pans under a middle-drag, as Blender's do, instead of
    // clipping its right end — unless it is the top bar wrapping onto a
    // second line (`BRegionFrame`).
    let strip = matches!(kind, BRegionType::Header | BRegionType::ToolHeader | BRegionType::Footer | BRegionType::Execute);
    let wraps = kind == BRegionType::Header && ctx.host.pins_header_cells() && crate::v2::part::filter::topbar_wraps(ctx.state);
    // The loading panel is taller than one line, so it is not a sideways-
    // scrolling strip: let the region grow around it (like a wrapping header).
    let element = if strip && !wraps && !header_loads(kind, ctx) {
        pan::BStripPan::of(&format!("{dbg}-{:?}", ctx.area)).attach(frame.strip_scroller(id, ctx), ctx)
    } else {
        frame.div(ctx).id(id)
    };
    let mut element = element.debug_selector(move || dbg.clone());
    // Tag the context with this region kind so pane children can distinguish
    // a header pane (content hugs, Left/Center/Right tri-cell) from a
    // content-region Center pane (stretches to fill).
    let child_ctx = ctx.with_region(kind);
    // The canvas fills the region, so a floating rail or sidebar has to be
    // painted after it — otherwise the viewport covers its own chrome.
    // Paint order, and only for the region whose panes actually float: the
    // canvas fills the Center region, so a floating rail has to be painted
    // after it. A header's side panes are ordinary strip content
    // (`pane::is_floating_pane`).
    let floats = ctx.host.floats_side_panels() && kind == BRegionType::Center;
    let floating = |ch: &BView| {
        floats
            && matches!(ch, BView::Pane { kind, .. } if matches!(kind, crate::v2::pane::BPaneType::Left | crate::v2::pane::BPaneType::Right))
    };
    let ordered: Vec<&BView> = list
        .iter()
        .filter(|ch| !floating(ch))
        .chain(list.iter().filter(|ch| floating(ch)))
        .collect();
    for ch in ordered {
        element = element.child(walk(ch, &child_ctx));
    }
    element.into_any_element()
}

mod pan;

#[cfg(test)]
mod tests;
