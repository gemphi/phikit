//! The tokens a `BFlex` is written in. Every number the element uses is one of
//! these, resolved against the render pass's style — so no layout decision
//! carries a literal.

pub use super::ink::BInk;
use super::super::ctx::BRenderCtx;
use crate::v2::theme::BStyleRole;

/// A spacing token: a gap, a padding or a margin, resolved from the theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BSpace {
    None,
    GapXs,
    GapSm,
    GapMd,
    GapLg,
    PadXs,
    PadSm,
    PadMd,
    PadLg,
    /// The gutter between resizable zones — the width of a resize seam.
    Seam,
    /// One border's width, as a space — a row that draws its own stroke
    /// insets its content by exactly what the stroke takes, so the content
    /// does not shift when the stroke appears on hover.
    Stroke,
    /// The lead above a vertical rail's first icon, level with the first
    /// accordion header beside it.
    RailLead,
    /// A content region's vertical pad above its first card (`content_pad`).
    ContentPad,
    /// The gap a menu cap leaves above itself in its strip.
    MenuTopGap,
    /// A card's edge to the rows it holds. With a row's own `PadSm` it lands
    /// the row's text one surface gutter (`PadMd`) in — the column the
    /// heading's fold triangle starts in.
    CardInset,
}

impl BSpace {
    pub(super) fn px(self, ctx: &BRenderCtx) -> f32 {
        let s = &ctx.style.spacing;
        match self {
            Self::None => s.gap_none,
            Self::GapXs => s.gap_xs,
            Self::GapSm => s.gap_sm,
            Self::GapMd => s.gap_md,
            Self::GapLg => s.gap_lg,
            Self::PadXs => s.pad_xs,
            Self::PadSm => s.pad_sm,
            Self::PadMd => s.pad_md,
            Self::PadLg => s.pad_lg,
            Self::Seam => ctx.style.resize.gap,
            Self::Stroke => ctx.style.strokes.border,
            Self::RailLead => ctx.style.rail_lead(ctx.host),
            Self::ContentPad => ctx.style.content_pad(ctx.host),
            Self::MenuTopGap => ctx.style.dimensions.menu_top_gap,
            Self::CardInset => (s.pad_md - s.pad_sm).max(0.0),
        }
    }
}

/// A named dimension from the theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BDim {
    HeaderHeight,
    /// A card's heading strip: the button height, so a panel heading and
    /// the rows and buttons under it are one module.
    CardHead,
    FooterHeight,
    ButtonHeight,
    RowHeight,
    ScrubbingHeight,
    AssetShelfHeight,
    BottomStripHeight,
    StatusLogHeight,
    ToolbarWidth,
    NavbarWidth,
    SidebarMinWidth,
    PanelWidth,
    ChannelsWidth,
    ToolPropsWidth,
    SideZoneDefaultWidth,
}

impl BDim {
    pub(super) fn px(self, ctx: &BRenderCtx) -> f32 {
        let d = &ctx.style.dimensions;
        match self {
            Self::HeaderHeight => d.header_height,
            Self::CardHead => d.card_head_height(),
            Self::FooterHeight => d.footer_height,
            Self::ButtonHeight => d.button_height,
            Self::RowHeight => d.row_height,
            Self::ScrubbingHeight => d.scrubbing_height,
            Self::AssetShelfHeight => d.asset_shelf_height,
            Self::BottomStripHeight => d.bottom_strip_height,
            Self::StatusLogHeight => d.status_log_height,
            Self::ToolbarWidth => d.toolbar_width,
            Self::NavbarWidth => d.navbar_width,
            Self::SidebarMinWidth => d.sidebar_min_width,
            Self::PanelWidth => d.panel_width,
            Self::ChannelsWidth => d.channels_width,
            Self::ToolPropsWidth => d.tool_props_width,
            Self::SideZoneDefaultWidth => d.side_zone_default_width,
        }
    }
}

/// A length on one axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BLen {
    /// The whole of the parent.
    Full,
    /// Zero — as a minimum, what lets a flex child shrink below its content.
    Zero,
    /// A theme dimension.
    Dim(BDim),
    /// A share of the parent — a reading sheet two thirds of the window wide.
    Share(f32),
    /// A length that comes from elsewhere — a dragged zone size, a sum of
    /// theme dimensions, a size a modal's spec declares. Never a literal in a
    /// walker: the number is computed from the theme or state, or declared in
    /// the one spec that owns it.
    Measured(f32),
}

/// The flex basis a growing child starts from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BBasis {
    /// 0% of the parent — `flex_1`'s basis, so free space splits by grow alone.
    NoShare,
    /// 0 px — the pinned header cells, which divide the row by grow alone.
    NoPixels,
    /// Its content's width — what a wrapping row measures a child by, so a
    /// child that no longer fits moves to the next line instead of shrinking.
    Content,
}

/// How children line up across the axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BAlign {
    Start,
    Center,
    End,
    Stretch,
}

/// How children spread along the axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BJustify {
    Start,
    Center,
    End,
    Between,
}

/// How one child sits across its parent's axis, overriding the parent's
/// `BAlign` for itself — a cap pinned to the strip's bottom edge, a button
/// centred in its rail, a rail entry stretched to what is left of the rail.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BSelf {
    Center,
    End,
    /// Fills what is left of the cross axis — not the same as `BLen::Full`:
    /// a cap carrying a gap on its far edge must fill the remainder of the
    /// rail, not 100% of it plus the margin.
    Stretch,
}

/// Which edges a border runs along.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BEdge {
    All,
    Bottom,
    /// A tree guide down the left edge: the separator hairline, as the
    /// Outliner's hierarchy lines (`tree::guide`) — a sub-panel's branch.
    Guide,
    /// A hairline on every side in an ink — the stroke of a drawn glyph.
    Ink(BInk),
}

/// Which corners are rounded, and by how much.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BRadius {
    /// Every corner, at a role's radius.
    Role(BStyleRole),
    /// Every corner, at a radius computed from roles by the caller (an inner
    /// arc compatible with the surface around it).
    All(f32),
    /// The top two corners.
    Top(f32),
    /// A chosen set of corners — the ones this element owns on its host's
    /// surface.
    Corners { tl: bool, tr: bool, bl: bool, br: bool, radius: f32 },
}

/// Where a container sits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BPlace {
    /// In the flow, positioning nothing.
    Flow,
    /// In the flow, and the anchor its absolute children position against.
    Anchor,
    /// Over its parent, covering it edge to edge.
    Overlay,
    /// Floating over its parent against one side, inset from that side and
    /// the top by `inset` and from the bottom by `foot` — the viewport's
    /// toolbar and sidebar, whose pinned cluster sits a cap gap off the
    /// bottom edge.
    Float { side: BSide, inset: BSpace, foot: BSpace },
    /// Floating directly above its anchor, flush with the anchor's left and
    /// right edges — the action log's drawer over the status bar.
    Drawer,
}

/// A side of the parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BSide {
    Left,
    Right,
}


/// Text defaults for everything inside.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BType {
    /// Body text — a panel title (Blender's `paneltitle` is the widget size).
    Body,
    Header,
    Caption,
}
