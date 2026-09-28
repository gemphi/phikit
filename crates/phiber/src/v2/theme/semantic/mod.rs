//! Kit-neutral semantic color, gradient, stroke, and effect derivations.
//!
//! GPUI renderers adapt these values to backend types. The resolved style owns
//! every design decision so renderers do not invent local shades or geometry.
//!
//! Every colour here is a step of the theme's shade ladder
//! (`palette/shade`): a surface is the token for its slot, a hover is one
//! step up, a gradient runs one step up to the slot's own shade, and a
//! "lift" moves whole steps — never a free lightness. A colour off the
//! ladder (the accent, an axis) keeps the old lightness nudge, since it has
//! no ladder to move along.

pub mod set;
pub mod stroke;

use super::{BColor, BResolvedUiStyle};

/// Which way a gradient sweeps.
///
/// Most chrome sweeps down, the way light falls. A Properties rail tab does
/// not: it has to hand its shade to the panel beside it so the selected tab
/// and the content read as one continuous surface, which means sweeping along
/// the rail-to-panel axis instead.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BGradientAxis {
    /// Top to bottom.
    #[default]
    Down,
    /// Left to right — a left-hand rail handing off to the panel on its right.
    Trailing,
    /// Right to left — a right-hand rail handing off to the panel on its left.
    Leading,
}

/// Where along an element a sweep has faded back to its base: half way,
/// so a lifted edge reads as a lit edge and the rest of the surface is the
/// surface — the theme's colour does not disappear under the sweep.
pub const SWEEP_STOP: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BGradientColors {
    /// The colour the sweep starts at, along [`BGradientColors::axis`].
    pub top: BColor,
    /// The colour it ends at.
    pub bottom: BColor,
    pub enabled: bool,
    pub axis: BGradientAxis,
    /// The fraction of the element at which `bottom` is reached; flat from
    /// there on (`SWEEP_STOP` for a sweep, 1 when flat).
    pub stop: f32,
}

impl BGradientColors {
    /// The same colours, swept along a different axis.
    pub fn along(self, axis: BGradientAxis) -> Self {
        Self { axis, ..self }
    }

    /// The lift the sweep starts with: how much lighter (or, in a light
    /// theme, darker) the lit edge is than the base, in lightness points.
    pub fn lift(&self) -> f32 {
        if self.enabled { self.top.2 - self.bottom.2 } else { 0.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BStrokeStyle {
    pub color: BColor,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCheckerColors {
    pub light: BColor,
    pub dark: BColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BSurfaceTone {
    Header,
    Footer,
    Sidebar,
    Body,
    Viewport,
    Modal,
    Card,
    Section,
    /// A sub-panel's well, `depth` levels under its panel: a step back
    /// toward the region for each level — a child accordion is recessed
    /// into its parent the way a tree's child row is indented under its
    /// parent's — so the tree reads in shade as well as in indent.
    Nested(u8),
}

/// The pointer states a control's fill answers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BControlState {
    Rest,
    Hover,
    Active,
}

impl BResolvedUiStyle {
    /// Lightness points one ladder step is worth when a lift has to be
    /// turned into steps (the dark half of the ladder is 2.9 apart).
    const STEP_LIGHTNESS: f32 = 3.0;

    /// `base` moved `by` whole steps along the ladder; a colour off the
    /// ladder moves `by` steps' worth of lightness instead.
    pub fn step(&self, base: BColor, by: i32) -> BColor {
        self.colors.ladder.step(base, by).unwrap_or_else(|| (base.0, base.1, (base.2 + by as f32 * Self::STEP_LIGHTNESS).clamp(0.0, 100.0), base.3))
    }

    /// A lift as Preferences spells it (`−1 .. 1` of lightness), applied as
    /// the nearest number of ladder steps.
    pub fn shade(&self, base: BColor, delta: f32) -> BColor {
        let steps = (delta * 100.0 / Self::STEP_LIGHTNESS).round() as i32;
        if steps == 0 { base } else { self.step(base, steps) }
    }

    pub(super) fn flat(color: BColor) -> BGradientColors {
        BGradientColors { top: color, bottom: color, enabled: false, axis: BGradientAxis::Down, stop: 1.0 }
    }

    /// A sweep while gradients are on — the only gradient the ladder allows
    /// — else flat: from one step over `base` at the left edge, back to
    /// `base` by `SWEEP_STOP` of the width and flat from there, so the lit
    /// edge reads against a border and the surface keeps its colour.
    pub fn gradient(&self, base: BColor, top_delta: f32, bottom_delta: f32) -> BGradientColors {
        let bottom = self.shade(base, bottom_delta);
        if !self.effects.gradients || top_delta <= bottom_delta {
            return Self::flat(bottom);
        }
        BGradientColors { top: self.step(bottom, 1), bottom, enabled: true, axis: BGradientAxis::Trailing, stop: SWEEP_STOP }
    }

    pub fn sheen(&self, base: BColor) -> BGradientColors {
        self.gradient(base, self.effects.gradient_lift, 0.0)
    }

    pub fn subtle_sheen(&self, base: BColor) -> BGradientColors {
        self.gradient(base, self.effects.sheen_lift, 0.0)
    }

    /// The surface for a tone: header and footer are the header shade;
    /// sidebar, body and modal the panel, a step lighter; a card, and a
    /// section inside it, a step over the panel (Blender's panel and
    /// sub-panel share `#3D3D3D` over the region's `#303030`); a nested
    /// sub-panel a step back toward the region per level; the viewport
    /// its own canvas shade, always flat.
    pub fn surface(&self, tone: BSurfaceTone) -> BGradientColors {
        let base = match tone {
            BSurfaceTone::Header | BSurfaceTone::Footer => self.colors.bg_header,
            BSurfaceTone::Sidebar | BSurfaceTone::Body | BSurfaceTone::Modal => self.colors.bg_panel,
            BSurfaceTone::Viewport => return Self::flat(self.viewport_surface()),
            // A sub-panel shares its panel's shade at any depth (Blender's
            // panel and sub-panel are both `#3D3D3D`): no darker strips.
            BSurfaceTone::Card | BSurfaceTone::Section | BSurfaceTone::Nested(_) => self.step(self.colors.bg_panel, 1),
        };
        self.sheen(base)
    }

    /// A heading over the surface of `tone`: one step over it, swept like
    /// every surface while gradients are on and flat where not, so a
    /// heading lifts off its rows by the same step in every theme — a
    /// panel's over its card, a sub-panel's over its well.
    pub fn heading_surface(&self, tone: BSurfaceTone) -> BGradientColors {
        self.sheen(self.step(self.surface(tone).bottom, 1))
    }

    /// The canvas: the palette's viewport token, opaque, flat.
    pub fn viewport_surface(&self) -> BColor {
        let mut color = self.colors.bg_viewport;
        color.3 = 1.0;
        color
    }
}
