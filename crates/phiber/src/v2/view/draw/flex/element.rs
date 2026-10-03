//! `BFlex` itself: the record of one container's layout decisions, and the
//! builder the levels describe themselves with.

use super::fill::BFill;
use super::place::BOverflow;
use super::token::{BAlign, BBasis, BEdge, BInk, BJustify, BLen, BPlace, BRadius, BSelf, BSpace, BType};
// (`BEdge`, `BFill`, `BRadius`, `BType` are the record's field types; their
// builder methods live in `surface.rs`.)
use crate::v2::theme::BElevation;
use crate::v2::view::layout::BAxis;

/// The complete description of one container. Fields left `None` are not
/// decided here and fall to the defaults.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFlex {
    pub axis: Option<BAxis>,
    /// Children that do not fit on the axis start a new line (a header
    /// too narrow for its menus puts them under its switcher).
    pub wrap: bool,
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub basis: Option<BBasis>,
    pub width: Option<BLen>,
    pub height: Option<BLen>,
    pub min_width: Option<BLen>,
    pub min_height: Option<BLen>,
    pub max_width: Option<BLen>,
    pub max_height: Option<BLen>,
    pub align: Option<BAlign>,
    pub justify: Option<BJustify>,
    pub gap: Option<BSpace>,
    pub pad_top: Option<BSpace>,
    pub pad_right: Option<BSpace>,
    pub pad_bottom: Option<BSpace>,
    pub pad_left: Option<BSpace>,
    pub margin_left: Option<BSpace>,
    pub margin_right: Option<BSpace>,
    pub margin_top: Option<BSpace>,
    pub margin_bottom: Option<BSpace>,
    /// Raised above its parent's alignment by this much of its own margin.
    pub raised_by: Option<BSpace>,
    /// How this child sits across its parent's axis, when it does not take
    /// the parent's own alignment.
    pub self_align: Option<BSelf>,
    pub fill: Option<BFill>,
    /// What it paints under the pointer. Written here so a hover is a token
    /// like any other fill, rather than a `.hover(|s| s.bg(…))` written out
    /// wherever a hoverable thing happens to be built.
    pub hover_fill: Option<BFill>,
    pub border: Option<BEdge>,
    pub radius: Option<BRadius>,
    /// How far off the workspace this sits, and so what it casts. `Base` is
    /// in the plane and casts nothing (`theme::BElevation`).
    pub elevation: BElevation,
    pub ink: Option<BInk>,
    pub type_size: Option<BType>,
    pub place: BPlace,
    pub overflow: BOverflow,
    /// Swallows pointer events so what lies beneath does not see them.
    pub occludes: bool,
    /// Drawn at half strength — a hint, not a control (Blender's panel grip).
    pub dimmed: bool,
    /// The hover group this container names, for parts shown on hover.
    pub group: Option<&'static str>,
    /// Shown only while the pointer is over the named group.
    pub revealed_by: Option<&'static str>,
}

impl BFlex {
    /// A flex container laid out along `axis`, deciding nothing else yet.
    pub const fn along(axis: BAxis) -> Self {
        Self {
            axis: Some(axis),
            ..Self::NOTHING
        }
    }

    /// A row.
    pub const fn row() -> Self {
        Self::along(BAxis::Row)
    }

    /// A column.
    pub const fn column() -> Self {
        Self::along(BAxis::Column)
    }

    /// A plain box: no flex container of its own, only its own size and place.
    pub const fn cell() -> Self {
        Self::NOTHING
    }

    const NOTHING: Self = Self {
        axis: None,
        wrap: false,
        grow: None,
        shrink: None,
        basis: None,
        width: None,
        height: None,
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        align: None,
        justify: None,
        gap: None,
        pad_top: None,
        pad_right: None,
        pad_bottom: None,
        pad_left: None,
        margin_left: None,
        margin_right: None,
        margin_top: None,
        margin_bottom: None,
        raised_by: None,
        self_align: None,
        fill: None,
        hover_fill: None,
        border: None,
        radius: None,
        elevation: BElevation::Base,
        ink: None,
        type_size: None,
        place: BPlace::Flow,
        overflow: BOverflow::Visible,
        occludes: false,
        group: None,
        revealed_by: None,
        dimmed: false,
    };

    pub const fn width(mut self, len: BLen) -> Self {
        self.width = Some(len);
        self
    }

    pub const fn height(mut self, len: BLen) -> Self {
        self.height = Some(len);
        self
    }

    /// The whole of the parent on both axes.
    pub const fn full(self) -> Self {
        self.width(BLen::Full).height(BLen::Full)
    }

    pub const fn min_width(mut self, len: BLen) -> Self {
        self.min_width = Some(len);
        self
    }

    pub const fn min_height(mut self, len: BLen) -> Self {
        self.min_height = Some(len);
        self
    }

    /// A ceiling on its height: a popover bounded so a long list scrolls
    /// inside its panel rather than running past the bottom of the area.
    pub const fn max_height(mut self, len: BLen) -> Self {
        self.max_height = Some(len);
        self
    }

    pub const fn max_width(mut self, len: BLen) -> Self {
        self.max_width = Some(len);
        self
    }

    /// May shrink below its content on both axes.
    pub const fn unclamped(self) -> Self {
        self.min_width(BLen::Zero).min_height(BLen::Zero)
    }

    // ── How it arranges its children ────────────────────────────────────

    /// How this one sits across its parent's axis.
    pub const fn self_align(mut self, align: BSelf) -> Self {
        self.self_align = Some(align);
        self
    }

    pub const fn align(mut self, align: BAlign) -> Self {
        self.align = Some(align);
        self
    }

    pub const fn justify(mut self, justify: BJustify) -> Self {
        self.justify = Some(justify);
        self
    }

    pub const fn gap(mut self, space: BSpace) -> Self {
        self.gap = Some(space);
        self
    }

}
