//! Pane walker — tri-cell horizontal slot inside a region. Left hugs left,
//! Center flexes, Right hugs right. Uses `items_stretch` (default) so a
//! Canvas child in the Center pane can fill the full pane height; button
//! rows keep their own `h(h_button)` and still align cleanly.
//!
//! In every **canvas host** (View3D + node graphs + image/UV/seq/clip) the
//! Toolbar (Left) and Sidebar (Right) panes are rendered as absolute
//! overlays on top of the content so the canvas fills the full host area
//! (Blender's floating panel behaviour).

use super::flex::{BAlign, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BPlace, BSide, BSpace, BDim};
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::walk;
use crate::v2::pane::BPaneType;
use crate::v2::view::BView;
use crate::v2::view::layout::BPaneLayout;
use gpui::prelude::*;
use gpui::{AnyElement, Hsla, hsla};

/// True when this pane lives inside a floating-side-panel host's center
/// region and should be rendered as a floating absolute overlay instead of a
/// flex slot. The host set is owned by [`BHostType::floats_side_panels`] (the
/// viewport + graphs + image/uv/seq/clip) — timelines fill content but keep
/// their docked Channels list, so they are excluded.
fn is_floating_pane(kind: BPaneType, ctx: &BRenderCtx) -> bool {
    // Only the CONTENT region's side panes float over the canvas. A header's
    // Left / Right panes are part of the strip: floating them turned the
    // viewport's menu bar into an absolute column stacked over the canvas,
    // which is what made the menus look detached and overlapping.
    ctx.host.floats_side_panels()
        && ctx.region == Some(crate::v2::region::BRegionType::Center)
        && matches!(kind, BPaneType::Left | BPaneType::Right)
}

/// The fill every piece of floating chrome shares — the toolbar, the sidebar
/// and the navigation gizmos. Slightly translucent so the canvas shows
/// through, matching Blender's floating panels. (`BFill::Floating` is the
/// same colour for containers built through `BFlex`.)
pub fn floating_bg(ctx: &BRenderCtx) -> Hsla {
    let base = to_hsla(ctx.style.colors.bg_panel);
    hsla(base.h, base.s, base.l, ctx.style.effects.floating_alpha)
}

/// How much room a floating side strip takes, including its inset from the
/// canvas edge. Overlays that sit in the same corner read this so they keep
/// their distance from the strip instead of colliding with it.
pub fn floating_strip_room(is_right: bool, ctx: &BRenderCtx) -> f32 {
    if !ctx.host.floats_side_panels() {
        return 0.0;
    }
    strip_width(is_right, ctx) + panels_width(is_right, ctx) + 2.0 * ctx.style.spacing.pad_sm
}

/// The sidebar region beside a right-hand strip while it is open (the N panel).
fn panels_width(is_right: bool, ctx: &BRenderCtx) -> f32 {
    if is_right { crate::v2::part::sidebar::show::BSidebarShown(ctx.host).beside_width(ctx.state, ctx.style) } else { 0.0 }
}

/// The content width of a floating side strip.
///
/// Collapsed it is one button wide, so the icon in it is square — a floating
/// strip is not part of the docked header grid, so sizing it by the grid
/// module left each icon in a 36x26 slot. Expanded it takes a panel's width
/// and shows its labels.
pub(super) fn strip_width(is_right: bool, ctx: &BRenderCtx) -> f32 {
    let key = if is_right { "sidebar" } else { "tool" };
    if crate::v2::part::strip::is_expanded(ctx.state, ctx.host, key) {
        ctx.style.dimensions.panel_width
    } else {
        // A rail entry is the card-head module square.
        ctx.style.dimensions.card_head_height()
    }
}

/// A floor for a pinned header cell that must not clip: one icon slot per
/// control it holds, plus the gaps between them.
///
/// A flex cell with a zero basis has no content size of its own to fall back
/// on, so a narrow window shrank the top bar's right-hand cell straight
/// through its icons. Counting the controls gives that cell a real minimum
/// without measuring text — everything in it is a square icon button.
fn control_floor(list: &[BView], ctx: &BRenderCtx) -> f32 {
    fn count(v: &BView) -> usize {
        match v {
            BView::Button(_) | BView::MenuAnchor(_) | BView::Icon { .. } => 1,
            BView::Row { list } | BView::Column { list } | BView::Split { list } => list.iter().map(count).sum(),
            BView::Card(card) => card.body.iter().chain(&card.foot).map(count).sum(),
            BView::Part { body, .. } => count(body),
            BView::Popover { anchor, .. } => count(anchor),
            BView::ZoneToggleGroup { controls } => controls.len(),
            BView::WindowControls { .. } => super::caption::WINDOW_CONTROLS.len(),
            _ => 0,
        }
    }
    let n: usize = list.iter().map(count).sum();
    if n == 0 {
        return 0.0;
    }
    let slot = ctx.style.dimensions.button_height;
    let gap = ctx.style.spacing.gap_xs;
    n as f32 * slot + (n.saturating_sub(1)) as f32 * gap
}

/// The parts of a pane — one enum for the level, as every other level
/// states its parts (`BAreaPart`, `BCardPart`, `BModalPart`, `BTablePart`).
enum BPanePart {
    /// A floating side strip: the viewport's toolbar or sidebar.
    Float {
        side: BSide,
        /// The strip's content width (`strip_width`).
        strip: f32,
    },
    /// A docked pane: a cell of its region's row.
    Cell {
        kind: BPaneType,
        in_header: bool,
        /// The header divides its width into three equal cells (the top bar).
        pinned: bool,
        /// The least width the right cell's controls fit in (`control_floor`).
        floor: f32,
        /// The pane holds a typed field (a directory, a search): it takes the
        /// strip's slack so the field can stretch, instead of hugging.
        fielded: bool,
        /// The strip is laid out as the columns under it — the tool header of
        /// a host whose tools region holds panels: its left pane is that region
        /// (`tools_column_width`), and every pane pads itself as the region
        /// under it does.
        columns: bool,
        /// The top bar wraps (`region`): the left cell takes the whole first
        /// line, so the other two land on the second; every cell is one line
        /// tall, so the wrapped bar is two.
        wraps: bool,
    },
    /// The slot a content-region Center gives each child: the whole of it.
    Stretch,
}

impl BFlexed for BPanePart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            // No rail behind the entries: each is a button on the canvas
            // (`control_fill` floating), a gap apart, as the mock-ups draw the
            // toolbar and the sidebar tabs.
            Self::Float { side, strip } => BFlex::column()
                // The strip's foot is its pinned cluster, a cap gap (2px) off
                // the canvas's bottom edge, as a docked rail's is off its
                // region's (`region`).
                .place(BPlace::Float { side, inset: BSpace::PadSm, foot: BSpace::MenuTopGap })
                .gap(BSpace::GapXs)
                .width(BLen::Measured(strip))
                .fill(BFill::Clear),
            Self::Cell { kind, in_header, pinned, floor, fielded, columns, wraps } => {
            let base = BFlex::row().gap(BSpace::GapXs);
            // A strip centres its controls on one line — a cap, a button, a
            // field, an icon, a read-out all share the same centre in every
            // header, tool bar and footer. The line is the centre of the cap
            // band: the strip less the `menu_top_gap` above it, which the pane
            // pads off the top so a cap (menu, tab) fills the band and its
            // bottom edge is the strip's, and a shorter control centres on the
            // same line rather than on the strip's — the pixel that had fields
            // riding a step below the menus beside them.
            let base = if in_header {
                base.align(BAlign::Center).height(BLen::Full).pad_top(BSpace::MenuTopGap)
            } else {
                base.align(BAlign::Center)
            };
            if pinned && wraps {
                // Each cell one line tall; the left one the whole first line,
                // which is what puts the tabs and the controls on the second.
                let cell = base.height(BLen::Dim(BDim::HeaderHeight)).min_height(BLen::Zero);
                return match (kind.is_center(), kind.is_right()) {
                    (true, _) => cell.equal_cell().yields().min_width(BLen::Zero).overflow(BOverflow::Clip).justify(BJustify::Center),
                    (_, true) => cell.keeps_size().min_width(BLen::Measured(floor)).justify(BJustify::End),
                    _ => cell.width(BLen::Full).keeps_size().justify(BJustify::Start),
                };
            }
            let center = kind.is_center();
            if pinned {
                // A pinned header divides its width into three equal cells, so its
                // centre group sits on the window's centre line however wide the
                // menus beside it are.
                let cell = base.equal_cell().height(BLen::Full).min_height(BLen::Zero);
                if center {
                    // Its width is purely its share of the free space, so a window
                    // with none leaves it nothing and it clips.
                    cell.yields()
                        .min_width(BLen::Zero)
                        .overflow(BOverflow::Clip)
                        .justify(BJustify::Center)
                } else if kind.is_right() {
                    // The right cell gives nothing up: the zone toggles and window
                    // controls must be reachable at any width.
                    cell.keeps_size()
                        .min_width(BLen::Measured(floor))
                        .justify(BJustify::End)
                } else {
                    // Once the centre is down to nothing the menus clip, rather
                    // than pushing the right-hand controls off the screen.
                    cell.yields()
                        .min_width(BLen::Zero)
                        .overflow(BOverflow::Clip)
                        .justify(BJustify::Start)
                }
            } else if center {
                // Center takes the slot so Left/Right sit at the ends; in a header
                // it centres its hugging content instead of stretching it — unless
                // it holds a field, which takes the slack (the file browser's
                // directory and search).
                let cell = base.fills().height(BLen::Full).min_height(BLen::Zero);
                // In a strip of columns the centre is the content region: it
                // pads itself as that region does, so its field starts where
                // the file list does.
                let cell = if columns { cell.pad_left(BSpace::Seam) } else { cell };
                if in_header && !fielded {
                    // Never narrower than its controls: a header too narrow
                    // for them scrolls (`header_scrolls`) rather than laying
                    // them over the panes beside it — the Timeline's jump
                    // buttons sat on its frame field.
                    cell.justify(BJustify::Center)
                } else {
                    cell.min_width(BLen::Zero)
                }
            } else if kind.is_right() {
                let cell = base.keeps_size().justify(BJustify::End);
                if columns { cell.pad_x(BSpace::PadMd) } else { cell }
            } else if columns {
                // The left pane of a strip laid out as the columns under it (the
                // file browser's path strip over its source list) is the tools
                // region: as wide as it, padded as it is, its parts at either
                // end — so the folder moves start where the list does, the
                // refresh ends where it does, and the fields beside it start
                // where the file list does.
                base.keeps_size()
                    .width(BLen::Measured(super::region::tools_column_width(ctx)))
                    .pad_left(BSpace::PadMd)
                    .justify(BJustify::Between)
            } else if fielded {
                // A left pane with a field in it (the file browser's path strip)
                // is the strip's body: it grows, and the field grows with it.
                base.fills().min_width(BLen::Zero)
            } else {
                // Left pane: shrink-to-fit, never expand.
                base.keeps_size()
            }
            }
            Self::Stretch => BFlex::row().fills().unclamped().height(BLen::Full),
        }
    }
}

pub fn view(kind: BPaneType, list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    // Canvas-host Toolbar / Sidebar: an overlay on the canvas, not a flex slot.
    if is_floating_pane(kind, ctx) {
        let is_right = kind == BPaneType::Right;
        let side = if is_right { BSide::Right } else { BSide::Left };
        let id = format!("pane-floating-{:?}-{:?}", kind, ctx.host);
        let mut outer = BPanePart::Float { side, strip: strip_width(is_right, ctx) + panels_width(is_right, ctx) }
            .flex(ctx)
            .div(ctx)
            .id(gpui::SharedString::from(id.clone()))
            .debug_selector(move || id);
        // Everything inside a floating pane is floating chrome: it connects
        // to nothing, so its controls round fully instead of capping into an
        // edge (`tab::cap`).
        let ctx = &ctx.floating();
        for ch in list {
            outer = outer.child(walk(ch, ctx));
        }
        return outer.into_any_element();
    }

    // A header row lays out its three panes tri-cell: Left hugs left, Center
    // takes the gap (so Left/Right land at opposite ends) but its children hug
    // and centre rather than stretch, and Right hugs right. Only a
    // content-region Center stretches its child to fill (canvas / accordion
    // stack) — the split that stops header content ballooning into full-width
    // pills or overflowing the pane.
    // A footer is a strip like the header: its controls sit on the same line.
    let in_header = matches!(
        ctx.region,
        Some(crate::v2::region::BRegionType::Header)
            | Some(crate::v2::region::BRegionType::ToolHeader)
            | Some(crate::v2::region::BRegionType::Footer)
    );
    let pinned = in_header && ctx.host.pins_header_cells();
    let floor = if pinned && kind.is_right() { control_floor(list, ctx) } else { 0.0 };
    let center = kind.is_center();
    let dbg = format!("pane-{:?}-{:?}", kind, ctx.host);
    let fielded = in_header && list.iter().any(holds_field);
    let columns = ctx.region == Some(crate::v2::region::BRegionType::ToolHeader) && ctx.host.tools_hold_panels();
    let wraps = pinned && ctx.region == Some(crate::v2::region::BRegionType::Header) && crate::v2::part::filter::topbar_wraps(ctx.state);
    let mut element = BPanePart::Cell { kind, in_header, pinned, floor, fielded, columns, wraps }
        .flex(ctx)
        .div(ctx)
        .id(gpui::SharedString::from(dbg.clone()))
        .debug_selector(move || dbg);
    // Only a content-region Center stretches its child to fill the slot (a
    // property stack or a canvas). A header Center hugs its content so menus
    // and the mode dropdown keep their natural width.
    let stretch_child = center && !in_header;
    for ch in list {
        if stretch_child {
            element = element.child(BPanePart::Stretch.flex(ctx).div(ctx).child(walk(ch, ctx)));
        } else {
            element = element.child(walk(ch, ctx));
        }
    }
    element.into_any_element()
}

/// Whether a view holds a typed field somewhere in it.
fn holds_field(view: &BView) -> bool {
    match view {
        BView::SearchField { .. } => true,
        BView::Ctrl(ctrl) => matches!(
            ctrl.kind,
            crate::v2::view::prim::BCtrlKind::Text
                | crate::v2::view::prim::BCtrlKind::Search
                | crate::v2::view::prim::BCtrlKind::Path
        ),
        BView::Part { body, .. } => holds_field(body),
        BView::Row { list } | BView::Column { list } => list.iter().any(holds_field),
        _ => false,
    }
}
