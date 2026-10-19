//! Pane walker — tri-cell horizontal slot inside a region. Left hugs left,
//! Center flexes, Right hugs right. Uses `items_stretch` (default) so a
//! Canvas child in the Center pane can fill the full pane height; button
//! rows keep their own `h(h_button)` and still align cleanly.
//!
//! In every **canvas host** (View3D + node graphs + image/UV/seq/clip) the
//! Toolbar (Left) and Sidebar (Right) panes are rendered as absolute
//! overlays on top of the content so the canvas fills the full host area
//! (Blender's floating panel behaviour).

use super::flex::{BAlign, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BPlace, BRadius, BSide, BSpace, BDim};
use crate::v2::theme::BStyleRole;
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
/// and the navigation gizmos: one step over the panel shade, so the strip
/// is a light bar on the body it floats over, at `floating_alpha` — near
/// solid, a hint of the canvas through it. (`BFill::Floating` is the same
/// colour for containers built through `BFlex`.) Blender's floating panels
/// are the region's own colour at `TH_BACK`'s alpha; a lighter bar is
/// Phiber's — deliberate, recorded.
pub fn floating_bg(ctx: &BRenderCtx) -> Hsla {
    let base = to_hsla(ctx.style.step(ctx.style.colors.bg_panel, 1));
    hsla(base.h, base.s, base.l, ctx.style.effects.floating_alpha)
}

/// How much room a floating side strip takes, including its inset from the
/// canvas edge. Overlays that sit in the same corner read this so they keep
/// their distance from the strip instead of colliding with it.
pub fn floating_strip_room(is_right: bool, ctx: &BRenderCtx) -> f32 {
    if !ctx.host.floats_side_panels() {
        return 0.0;
    }
    strip_width(is_right, ctx) + panels_width(is_right, ctx) + 2.0 * ctx.style.spacing.pad_sm + 2.0 * ctx.style.spacing.pad_xs
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
pub fn strip_width(is_right: bool, ctx: &BRenderCtx) -> f32 {
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
            // A floating strip is a bar: the floating shade behind its
            // buttons (`floating_bg`), rounded as a panel is, its buttons a
            // pad in from its edges — so the strip reads as one sidebar over
            // the canvas rather than loose buttons on it, and the foot's
            // controls sit off its edges as every other button does.
            Self::Float { side, strip } => BFlex::column()
                // The strip's foot is its pinned cluster, a cap gap (2px) off
                // the canvas's bottom edge, as a docked rail's is off its
                // region's (`region`).
                .place(BPlace::Float { side, inset: BSpace::PadSm, foot: BSpace::MenuTopGap })
                .gap(BSpace::GapXs)
                .pad(BSpace::PadXs)
                .width(BLen::Measured(strip + 2.0 * ctx.style.spacing.pad_xs))
                .radius(BRadius::Role(BStyleRole::Panel))
                .fill(BFill::Floating),
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
            let base = match in_header {
                true => base.align(BAlign::Center).height(BLen::Full).pad_top(BSpace::MenuTopGap),
                false => base.align(BAlign::Center),
            };
            // Three cells in every strip — what the editor IS on the left,
            // what you are LOOKING FOR in the middle, how it is FILTERED on
            // the right — so which cell this is, and how the strip shares its
            // width, is one match rather than a ladder of conditions.
            //
            // A pinned header (the top bar) divides its width into equal
            // thirds, so its centre group sits on the window's centre line
            // however wide the menus beside it are. Every other header lets
            // the centre take the slack instead, which is what pushes Left and
            // Right to the ends. Either way the right cell gives nothing up:
            // the zone toggles and the window controls must be reachable at
            // any width, and the centre clips before they move.
            let third = base.equal_cell().height(BLen::Full).min_height(BLen::Zero);
            // Wrapping (`filter::topbar_wraps`): each cell one line tall, and
            // the left one the whole first line — which is what puts the
            // workspace tabs and the right-hand controls on the second.
            let line = base.height(BLen::Dim(BDim::HeaderHeight)).min_height(BLen::Zero);
            let clipped = |cell: BFlex, at| cell.yields().min_width(BLen::Zero).overflow(BOverflow::Clip).justify(at);
            let held = |cell: BFlex| cell.keeps_size().min_width(BLen::Measured(floor)).justify(BJustify::End);
            match (pinned, wraps, kind.is_center(), kind.is_right()) {
                (true, true, true, _) => clipped(line.equal_cell(), BJustify::Center),
                (true, true, _, true) => held(line),
                (true, true, ..) => line.width(BLen::Full).keeps_size().justify(BJustify::Start),
                (true, _, true, _) => clipped(third, BJustify::Center),
                (true, _, _, true) => held(third),
                // Once the centre is down to nothing the menus clip, rather
                // than pushing the right-hand controls off the screen.
                (true, ..) => clipped(third, BJustify::Start),
                // The centre takes the slot, so Left and Right sit at the
                // ends. In a header it centres its hugging content instead of
                // stretching it, and is never narrower than its controls: a
                // header too narrow for them scrolls (`header_scrolls`) rather
                // than laying them over the panes beside it — the Timeline's
                // jump buttons sat on its frame field. A field in it takes the
                // slack instead (the file browser's directory and search). In
                // a strip of columns the centre IS the content region, padded
                // as that region is, so its field starts where the file list
                // does.
                (_, _, true, _) => {
                    let cell = base.fills().height(BLen::Full).min_height(BLen::Zero);
                    let cell = match columns {
                        true => cell.pad_left(BSpace::Seam),
                        false => cell,
                    };
                    match in_header && !fielded {
                        true => cell.justify(BJustify::Center),
                        false => cell.min_width(BLen::Zero),
                    }
                }
                (_, _, _, true) => {
                    let cell = base.keeps_size().justify(BJustify::End);
                    match columns {
                        true => cell.pad_x(BSpace::PadMd),
                        false => cell,
                    }
                }
                // The left pane of a strip laid out as the columns under it
                // (the file browser's path strip over its source list) is the
                // tools region: as wide as it, padded as it is, its parts at
                // either end — so the folder moves start where the list does,
                // the refresh ends where it does, and the fields beside it
                // start where the file list does. With a field of its own it
                // is the strip's body instead: it grows, and the field grows
                // with it. Otherwise a left pane shrinks to fit and never
                // expands.
                _ if columns => base
                    .keeps_size()
                    .width(BLen::Measured(super::region::tools_column_width(ctx)))
                    .pad_left(BSpace::PadMd)
                    .justify(BJustify::Between),
                _ if fielded => base.fills().min_width(BLen::Zero),
                _ => base.keeps_size(),
            }
            }
            Self::Stretch => BFlex::row().fills().unclamped().height(BLen::Full),
        }
    }
}

/// How much of the canvas a floating pane covers on one side — the strip
/// plus whatever panels are open beside it.
///
/// A canvas host floats its Toolbar and Sidebar *over* the canvas rather
/// than taking a slot beside it, so the canvas element runs the full width
/// of the editor and anything painted onto it — the render window's info
/// line, say — is drawn underneath them unless it holds off by this much.
/// Callers used to approximate it with `dimensions.sidebar_width`, which
/// is the *minimum*, not the width in force: with the shared
/// `ctx.sidebar.width` dragged to 425 and a 240-wide strip beside it, the
/// real figure was over 650 and the info line's right-hand group was
/// painted under the panel. One function, so the inset cannot drift from
/// the pane it is clearing.
pub fn floating_width(is_right: bool, ctx: &BRenderCtx) -> f32 {
    strip_width(is_right, ctx) + panels_width(is_right, ctx)
}

pub fn view(kind: BPaneType, list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    // Canvas-host Toolbar / Sidebar: an overlay on the canvas, not a flex slot.
    if is_floating_pane(kind, ctx) {
        // Hidden (Toolbar off, T): the parts are all empty, so the floating
        // bar draws nothing rather than an empty rounded strip on the canvas.
        if list.iter().all(view_is_empty) {
            return gpui::div().into_any_element();
        }
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

/// Whether a view contributes nothing to paint — an empty part, or a part
/// whose body is empty (a hidden Toolbar).
fn view_is_empty(view: &BView) -> bool {
    match view {
        BView::Empty => true,
        BView::Part { body, .. } => view_is_empty(body),
        _ => false,
    }
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
