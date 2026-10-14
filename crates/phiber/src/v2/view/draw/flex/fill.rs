//! `BFill` — what a container paints behind its children.
//!
//! Its own file because the fill vocabulary is the one token that carries
//! real resolution logic: every other token in `token.rs` is a plain enum
//! the layout reads, while a fill has to ask the theme for a surface, a
//! gradient, a row's stripe or a control's state colour. Keeping it here
//! leaves `token.rs` the flat vocabulary it is meant to be.

use super::super::color::{resolved_gradient, to_hsla};
use super::super::ctx::BRenderCtx;
use super::token::BInk;
use crate::v2::theme::{BColor, BSurfaceTone};
use crate::v2::view::prim::BRowShade;
use gpui::Background;

/// What a container paints behind its children.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BFill {
    /// Explicitly nothing.
    Clear,
    /// The gutter showing between zones.
    Gutter,
    /// The current host's body — the panel shade for an editor and the
    /// activity bar, the chrome shade for the top and status bars
    /// (`host_background`).
    HostBody,
    /// A semantic surface tone (modal, header, …).
    Tone(BSurfaceTone),
    /// A heading over a tone's surface, a step over it (`heading_surface`).
    Heading(BSurfaceTone),
    /// The current host's header surface.
    HostHeader,
    /// The current host's strip gradient.
    HostStrip,
    /// The current host's vertical strip: a step under its body, so the
    /// active tab — painted in the body's shade — is a hole cut in it.
    HostRail,
    /// A rail tab at rest — `wcol_tab.inner`, one step under the strip it sits
    /// on, since the strip is level with the panel beside it.
    RailTabResting,
    /// Chrome floating over a canvas: a step over the panel colour, near
    /// solid (`pane::floating_bg`).
    Floating,
    /// The dim laid over the workspace behind a modal.
    ModalDim,
    /// A value's own colour — a colour control's swatch. RGBA, 0..1.
    Swatch([f32; 4]),
    /// The same, for a value held as HSL: hue in degrees, saturation and
    /// lightness in percent, alpha 0..1 — the spelling a `BStyleDescriptor`
    /// formats and parses back, which is what a Preferences colour carries.
    ///
    /// Without this, `ctrl/mod.rs` matched `BValue::Color` alone and fell back to
    /// opaque black for anything else, so all 29 Preferences swatches were black
    /// squares beside correct numbers.
    SwatchHsl(BColor),
    /// An ink, as a fill — the filled part of a drawn glyph.
    Ink(BInk),
    /// A rule between entries — a header's divider. `colors.sep` sits near
    /// the dark end of the ladder, so this is for a rule against a dark
    /// surface; a rule inside a panel wants [`Self::MenuDivider`].
    Separator,
    /// A divider inside a menu or a panel: `colors.border`, the shade every
    /// other border uses and a step lighter than the surface it sits on.
    /// Borders here are always lighter than their ground, never darker.
    MenuDivider,
    /// A tone's surface with its gradient flattened to the bottom stop: a
    /// progress track, where a sweep would read as motion that is not there.
    Flat(BSurfaceTone),
    /// A list row at rest, striped or not as its shade says.
    Row(BRowShade),
    /// A list row under the pointer.
    RowHover(BRowShade),
    /// An item under the pointer — a menu entry, an outliner row.
    ItemHover,
    /// A control that is on: a pressed toggle, the active tab.
    ControlActive,
}

impl BFill {
    pub(in crate::v2::view::draw) fn background(self, ctx: &BRenderCtx) -> Background {
        match self {
            Self::Clear => gpui::transparent_black().into(),
            Self::Gutter => to_hsla(ctx.style.resize_gutter_background()).into(),
            Self::HostBody => resolved_gradient(ctx.style.host_body_surface(ctx.host)),
            Self::Tone(tone) => resolved_gradient(ctx.style.surface(tone)),
            Self::Heading(tone) => resolved_gradient(ctx.style.heading_surface(tone)),
            Self::HostHeader => resolved_gradient(ctx.style.host_header_surface(ctx.host)),
            Self::HostStrip => resolved_gradient(ctx.style.strip_gradient(ctx.host)),
            Self::HostRail => resolved_gradient(ctx.style.rail_surface(ctx.host)),
            Self::RailTabResting => to_hsla(ctx.style.rail_tab_resting(ctx.host)).into(),
            Self::ModalDim => to_hsla(ctx.style.colors.bg_modal_dim).into(),
            Self::Swatch([r, g, b, a]) => gpui::Rgba { r, g, b, a }.into(),
            Self::SwatchHsl(color) => to_hsla(color).into(),
            Self::Ink(ink) => ink.color(ctx).into(),
            Self::Separator => to_hsla(ctx.style.colors.sep).into(),
            Self::MenuDivider => to_hsla(ctx.style.colors.border).into(),
            Self::Flat(tone) => to_hsla(ctx.style.surface(tone).bottom).into(),
            Self::Row(shade) => match shade.fill(ctx.style) {
                Some(color) => to_hsla(color).into(),
                None => gpui::transparent_black().into(),
            },
            Self::RowHover(shade) => to_hsla(shade.hover(ctx.style, ctx.host)).into(),
            Self::ItemHover => to_hsla(ctx.style.host_item_hover(ctx.host)).into(),
            Self::ControlActive => to_hsla(ctx.style.colors.bg_active).into(),
            Self::Floating => {
                let base = to_hsla(ctx.style.step(ctx.style.colors.bg_panel, 1));
                gpui::hsla(base.h, base.s, base.l, ctx.style.effects.floating_alpha).into()
            }
        }
    }
}
