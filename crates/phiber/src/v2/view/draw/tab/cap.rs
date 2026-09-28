//! Cap geometry for a strip tab, and the two layers that hang off it.
//!
//! A tab is a cap cut into its strip: it is flush with the edge the pane
//! continues from, rounded on the opposite edge, and separated from the
//! strip's far edge by one `menu_top_gap`. The topbar does this vertically;
//! a rail does the same thing turned on its side, which is what [`BCapSide`]
//! names.

use super::super::color::to_hsla;
use crate::v2::theme::{BResolvedUiStyle, BStyleRole};
use crate::v2::view::prim::BRailSide;
use gpui::prelude::*;
use gpui::{Context, Hsla, Render, SharedString, Styled, Window, div, px};

/// Which edge of a tab runs flush into the pane it belongs to — or `Free`
/// when it belongs to no pane at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BCapSide {
    /// Floating chrome: the control connects to nothing, so it is a button,
    /// not a cap — rounded on every side, with no gap to any edge.
    Free,
    /// The topbar and editor tab strips: the pane is below.
    Below,
    /// A left-hand rail: the pane is to the right.
    Right,
    /// A right-hand rail: the pane is to the left.
    Left,
}

impl From<BRailSide> for BCapSide {
    fn from(side: BRailSide) -> Self {
        match side {
            BRailSide::Left => BCapSide::Right,
            BRailSide::Right => BCapSide::Left,
        }
    }
}

impl BCapSide {
    /// The side a control standing in `ctx` caps into: `Below` in a header
    /// strip (its menus and tabs), `Free` everywhere else (a dropdown in a
    /// panel, a row in a menu).
    pub fn of(ctx: &super::super::ctx::BRenderCtx) -> Self {
        use crate::v2::region::BRegionType;
        match ctx.region {
            Some(BRegionType::Header | BRegionType::ToolHeader | BRegionType::Footer | BRegionType::Execute) => Self::Below,
            _ => Self::Free,
        }
    }

    /// The axis a rail cap's fill sweeps along, so its shade lands on the
    /// panel's own tone at the edge the two meet — one continuous area rather
    /// than two boxes that happen to match.
    pub fn gradient_axis(self) -> crate::v2::theme::BGradientAxis {
        use crate::v2::theme::BGradientAxis;
        match self {
            // The pane is to the right, so the sweep runs left to right.
            BCapSide::Right => BGradientAxis::Trailing,
            // The pane is to the left, so it runs right to left.
            BCapSide::Left => BGradientAxis::Leading,
            BCapSide::Free | BCapSide::Below => BGradientAxis::Down,
        }
    }

    /// Whether this cap belongs to a vertical rail (a toolbar, a sidebar, a
    /// navbar) rather than a horizontal strip. A rail entry spans the rail's
    /// width and reads from its leading edge; a header cap hugs its label.
    pub fn in_rail(self) -> bool {
        !matches!(self, BCapSide::Below)
    }

    /// Round the edge opposite the pane, so the cap and the pane read as one
    /// continuous surface.
    pub fn round<E: Styled>(self, element: E, r: gpui::Pixels) -> E {
        match self {
            BCapSide::Free => element.rounded(r),
            BCapSide::Below => element.rounded_t(r),
            BCapSide::Right => element.rounded_l(r),
            BCapSide::Left => element.rounded_r(r),
        }
    }

    /// The gap that separates the cap from the strip's far edge. A cap in a
    /// horizontal strip fills the strip's cap band — the header pane pads the
    /// gap off the top (`pane::BPaneCell`) so every control shares its
    /// centre — and pins itself to the bottom edge.
    pub fn gap<E: Styled>(self, element: E, gap: f32) -> E {
        match self {
            BCapSide::Free => element,
            BCapSide::Below => element.self_end(),
            BCapSide::Right => element.ml(px(gap)),
            BCapSide::Left => element.mr(px(gap)),
        }
    }

    /// Place a Properties-style paint layer: inset on the far edge, bleeding
    /// past the pane-facing edge so the two surfaces meet.
    pub fn paint<E: Styled>(self, element: E, inset: f32, overlap: f32) -> E {
        match self {
            BCapSide::Free => element.left(px(inset)).right(px(inset)),
            BCapSide::Below => element.left(px(inset)).right(px(inset)).bottom(px(-overlap)),
            BCapSide::Right => element.left(px(inset)).right(px(-overlap)),
            BCapSide::Left => element.right(px(inset)).left(px(-overlap)),
        }
    }
}

/// Minimal tooltip view. Style values are captured from `BRenderCtx` at the
/// call site so no hardcoded design literals appear here.
pub struct BTabTooltip {
    pub text: SharedString,
    pub pad_x: f32,
    pub pad_y: f32,
    pub radius: f32,
    pub bg: Hsla,
    pub fg: Hsla,
    pub size: f32,
}

impl Render for BTabTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(self.pad_x))
            .py(px(self.pad_y))
            .rounded(px(self.radius))
            .bg(self.bg)
            .text_color(self.fg)
            .text_size(px(self.size))
            .child(self.text.clone())
    }
}

impl BTabTooltip {
    /// Build one from the resolved style, so callers pass only the text.
    pub fn of(text: &str, style: &BResolvedUiStyle) -> Self {
        Self {
            text: SharedString::from(text.to_string()),
            pad_x: style.spacing.pad_sm,
            pad_y: style.spacing.pad_xs,
            radius: style.radius_for(BStyleRole::Button),
            bg: to_hsla(style.colors.bg_app),
            fg: to_hsla(style.colors.text),
            size: style.typography.body,
        }
    }
}
