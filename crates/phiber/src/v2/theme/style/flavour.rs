//! The dimensional coat: flat, or spatial.
//!
//! A theme's palette says what colour the window is; the coat says how deep it
//! reads. The two are orthogonal — any palette can be worn flat, the way
//! Blender's chrome is flat everywhere, or in the spatial look the app's
//! virtual world is for, where surfaces sweep and lifted things cast so a menu
//! reads as glass over the scene and a zone as a panel standing in space.
//!
//! Blender has no such axis, so the spatial coat is a deliberate, recorded
//! divergence (phiber drawing on Apple's depth rather than inventing one). The
//! coat is not a per-element decision: it is set once, for the whole window —
//! every zone, menu and panel, not just the canvas — which is why it lives at
//! the one place the style is resolved (`view/draw/backend`).
//!
//! ## Scope
//!
//! Whose choice the coat is:
//!
//! * `Auto` leaves the palette's own depth cues alone — today's look, per
//!   theme — and is the default, so nothing changes until a coat is asked for.
//! * `Flat` and `Spatial` force one coat across the whole window.
//!
//! Each space also has a natural coat ([`BFlavour::of_space`]): the
//! viewport-centric spaces are the virtual world, the editor spaces (nodes,
//! text) are flat. `Auto` does not wear it yet — turning the per-space default
//! on is a separate, visible step (it reshapes every surface in those spaces),
//! so it is kept behind the switch until the spatial coat's look is settled.

use super::{BEffects, BShadowStyle};
use crate::v2::space::BSpaceType;

/// The coat a surface wears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BFlavour {
    /// No depth: gradients off, nothing casts. Blender's look.
    Flat,
    /// The virtual world: surfaces sweep and lifted things cast.
    Spatial,
}

/// Whose choice the coat is (`ctx.flavour`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BFlavourScope {
    /// Follow the palette's own depth cues — today's look, per theme.
    #[default]
    Auto,
    /// Force the flat coat across the whole window.
    Flat,
    /// Force the spatial coat across the whole window.
    Spatial,
}

impl BFlavourScope {
    pub const ALL: [Self; 3] = [Self::Auto, Self::Flat, Self::Spatial];

    /// The value the state stores (`ctx.flavour`).
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Flat => "flat",
            Self::Spatial => "spatial",
        }
    }

    /// The label a control shows.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Flat => "Flat",
            Self::Spatial => "Spatial",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scope| scope.slug() == slug)
    }

    /// The coat to force over `space`, or `None` to leave the palette's
    /// effects as they are. `Auto` forces nothing today (the per-space default
    /// is read with [`BFlavour::of_space`] but not yet worn), so the window
    /// looks exactly as it did until a coat is chosen.
    pub fn coat(self, _space: BSpaceType) -> Option<BFlavour> {
        match self {
            Self::Auto => None,
            Self::Flat => Some(BFlavour::Flat),
            Self::Spatial => Some(BFlavour::Spatial),
        }
    }
}

impl BFlavour {
    /// This space's natural coat — what `Auto` will wear once the per-space
    /// default is turned on. Viewport-centric spaces are the virtual world;
    /// the editor spaces stay flat.
    pub const fn of_space(space: BSpaceType) -> Self {
        match space {
            BSpaceType::Layout
            | BSpaceType::Modeling
            | BSpaceType::Sculpting
            | BSpaceType::TexturePaint
            | BSpaceType::Rendering
            | BSpaceType::Animation => Self::Spatial,
            BSpaceType::UvEditing
            | BSpaceType::Shading
            | BSpaceType::Compositing
            | BSpaceType::GeometryNodes
            | BSpaceType::Scripting => Self::Flat,
        }
    }

    /// `base` worn in this coat: the coat sets the depth cues — whether
    /// surfaces sweep and how far lifted ones cast — and leaves the palette's
    /// colour-bound lifts untouched, so a flat palette worn spatial gains
    /// depth without changing hue. A flat coat is the floor (nothing casts);
    /// a spatial one is at least Large, so a palette that already casts
    /// further keeps its ceiling.
    pub fn wear(self, base: BEffects) -> BEffects {
        match self {
            Self::Flat => BEffects { gradients: false, surface_shadow: BShadowStyle::None, ..base },
            Self::Spatial => BEffects {
                gradients: true,
                gradient_lift: if base.gradient_lift > 0.0 { base.gradient_lift } else { 0.035 },
                surface_shadow: base.surface_shadow.at_least(BShadowStyle::Large),
                ..base
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::theme::style::effect::SPHERE_EFFECTS;

    #[test]
    fn auto_forces_no_coat_in_any_space() {
        // The default scope leaves the palette's effects alone everywhere, so
        // the window looks exactly as it did.
        for space in BSpaceType::EVERY.iter().copied() {
            assert_eq!(BFlavourScope::Auto.coat(space), None, "{space:?}");
        }
    }

    #[test]
    fn a_forced_scope_wears_its_coat_regardless_of_space() {
        for space in BSpaceType::EVERY.iter().copied() {
            assert_eq!(BFlavourScope::Flat.coat(space), Some(BFlavour::Flat), "{space:?}");
            assert_eq!(BFlavourScope::Spatial.coat(space), Some(BFlavour::Spatial), "{space:?}");
        }
    }

    #[test]
    fn the_flat_coat_turns_every_depth_cue_off() {
        let worn = BFlavour::Flat.wear(SPHERE_EFFECTS);
        assert!(!worn.gradients, "gradients off");
        assert_eq!(worn.surface_shadow, BShadowStyle::None, "nothing casts");
    }

    #[test]
    fn the_spatial_coat_turns_depth_on_without_touching_colour_lifts() {
        // A flat palette worn spatial gains sweep and a cast, and keeps every
        // colour-bound lift the palette set — the coat is depth, not hue.
        let flat = crate::v2::theme::style::effect::GRAPHITE_EFFECTS;
        assert!(!flat.gradients, "the base is flat to begin with");
        let worn = BFlavour::Spatial.wear(flat);
        assert!(worn.gradients, "gradients on");
        assert!(worn.surface_shadow.rung() >= BShadowStyle::Large.rung(), "it casts at least Large");
        assert_eq!(worn.sidebar_lift, flat.sidebar_lift, "colour-bound lifts are the palette's");
        assert_eq!(worn.card_lift, flat.card_lift);
    }

    #[test]
    fn a_palette_that_casts_further_keeps_its_ceiling_when_worn_spatial() {
        // Sphere's ceiling is ExtraLarge; the spatial floor of Large must not
        // pull it down.
        let worn = BFlavour::Spatial.wear(SPHERE_EFFECTS);
        assert_eq!(worn.surface_shadow, SPHERE_EFFECTS.surface_shadow);
    }
}
