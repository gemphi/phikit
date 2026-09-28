//! The elevation scale — how far off the workspace a surface sits, and the
//! shadow that says so. Split out of `style/effect.rs` so each stays under the
//! file cap.
//!
//! Every lifted thing used to cast the theme's one `surface_shadow`, so a
//! modal, a dropdown and the action log's drawer read at the same depth while
//! the viewport's floating toolbar cast nothing at all. A theme still declares
//! a single shadow — its ceiling, the one a modal casts — and each level below
//! is one rung softer, the way a shade moves along its ladder in whole steps
//! rather than by a free value. A theme whose ceiling is `None` is flat at
//! every level: no level can talk a flat theme into casting.

use super::BShadowStyle;

/// What a surface sits on, in five steps from the plane to over everything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BElevation {
    /// In the plane of the workspace: a zone, an area, a docked region. The
    /// seams and the shade changes say where these are; a shadow would put a
    /// rim on something that was never lifted.
    #[default]
    Base,
    /// Lifted off the surface it sits on but still docked — a panel's card,
    /// the action log's drawer over the status bar.
    Raised,
    /// Floating over a canvas: the viewport's toolbar and sidebar, its
    /// gizmos. These sit on the scene, so they cast on it.
    Floating,
    /// Over the workspace, dismissed by a click away: a dropdown, a menu, a
    /// tooltip.
    Overlay,
    /// Over everything, behind a dim. The theme's full shadow.
    Modal,
}

impl BElevation {
    /// Every level, in order from the plane upward.
    pub const ALL: [Self; 5] = [Self::Base, Self::Raised, Self::Floating, Self::Overlay, Self::Modal];

    /// How many rungs below the theme's ceiling this level casts.
    pub const fn rungs_under_ceiling(self) -> u8 {
        match self {
            Self::Modal => 0,
            Self::Overlay => 1,
            Self::Floating => 2,
            Self::Raised => 3,
            // Not lifted at all, whatever the theme's ceiling is.
            Self::Base => u8::MAX,
        }
    }

    /// This level's shadow under a theme whose ceiling is `ceiling`.
    pub const fn shadow(self, ceiling: BShadowStyle) -> BShadowStyle {
        ceiling.softer(self.rungs_under_ceiling())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flat_theme_is_flat_at_every_level() {
        for level in BElevation::ALL {
            assert_eq!(level.shadow(BShadowStyle::None), BShadowStyle::None, "{level:?}");
        }
    }

    #[test]
    fn the_plane_never_casts() {
        for ceiling in BShadowStyle::ALL {
            assert_eq!(BElevation::Base.shadow(ceiling), BShadowStyle::None, "{ceiling:?}");
        }
    }

    #[test]
    fn a_modal_casts_the_theme_ceiling_and_each_level_below_is_softer() {
        let ceiling = BShadowStyle::ExtraLarge;
        assert_eq!(BElevation::Modal.shadow(ceiling), BShadowStyle::ExtraLarge);
        assert_eq!(BElevation::Overlay.shadow(ceiling), BShadowStyle::Large);
        assert_eq!(BElevation::Floating.shadow(ceiling), BShadowStyle::Medium);
        assert_eq!(BElevation::Raised.shadow(ceiling), BShadowStyle::Small);
        assert_eq!(BElevation::Base.shadow(ceiling), BShadowStyle::None);
    }

    #[test]
    fn levels_never_cast_harder_than_the_level_above() {
        for ceiling in BShadowStyle::ALL {
            let mut previous = BShadowStyle::None;
            for level in BElevation::ALL {
                let shadow = level.shadow(ceiling);
                assert!(
                    shadow.rung() >= previous.rung(),
                    "{ceiling:?}: {level:?} casts {shadow:?}, under the level below it"
                );
                previous = shadow;
            }
        }
    }

    #[test]
    fn a_shallow_ceiling_runs_out_rather_than_wrapping() {
        // Apple's ceiling is Medium: a modal and a dropdown carry it and its
        // rung below, and nothing under those casts at all.
        let ceiling = BShadowStyle::Medium;
        assert_eq!(BElevation::Modal.shadow(ceiling), BShadowStyle::Medium);
        assert_eq!(BElevation::Overlay.shadow(ceiling), BShadowStyle::Small);
        assert_eq!(BElevation::Floating.shadow(ceiling), BShadowStyle::None);
        assert_eq!(BElevation::Raised.shadow(ceiling), BShadowStyle::None);
    }
}
