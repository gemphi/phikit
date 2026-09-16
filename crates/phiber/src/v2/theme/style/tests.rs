//! Style-defaults tests. Split out of `style.rs` so the parent stays under
//! the 200-line cap.

use super::*;
use crate::v2::theme::palette::{BPaletteName, palette_for};

#[test]
fn preserved_radius_scales_are_owned_by_ui_defaults() {
    assert_eq!(
        APPLE_UI_STYLE_DEFAULTS.radii,
        BRadiusScale {
            xs: 4.0,
            sm: 7.0,
            md: 10.0,
            lg: 14.0,
            xl: 18.0,
        }
    );
    assert_eq!(
        BLENDER_UI_STYLE_DEFAULTS.radii,
        BRadiusScale {
            xs: 1.0,
            sm: 2.0,
            md: 4.0,
            lg: 6.0,
            xl: 8.0,
        }
    );
}

#[test]
fn resize_defaults_preserve_usability() {
    let resize = APPLE_UI_STYLE_DEFAULTS.resize;
    // visible_seam is 2 px — one step stronger than the hairline border so
    // the resize boundary reads as a deliberate affordance; the 14 px hit
    // band stays much wider so the seam is easy to grab.
    assert_eq!((resize.visible_seam, resize.hit_width), (2.0, 14.0));
    // The hit band must be wider than the visible line.
    assert!(resize.hit_width >= resize.visible_seam);
}

#[test]
fn resolved_style_copies_every_default_domain() {
    for name in BPaletteName::ALL.iter().copied() {
        let defaults = ui_style_defaults(name);
        let resolved = BResolvedUiStyle::from(defaults);
        assert_eq!(resolved.colors, defaults.colors);
        assert_eq!(resolved.radii, defaults.radii);
        assert_eq!(resolved.typography, defaults.typography);
        assert_eq!(resolved.spacing, defaults.spacing);
        assert_eq!(resolved.dimensions, defaults.dimensions);
        assert_eq!(resolved.strokes, defaults.strokes);
        assert_eq!(resolved.resize, defaults.resize);
        assert_eq!(resolved.effects, defaults.effects);
        assert_eq!(resolved.roles, defaults.roles);
    }
}

#[test]
fn semantic_role_defaults_reference_radius_tokens() {
    assert_eq!(DEFAULT_ROLES.workspace, BRadiusToken::None);
    assert_eq!(DEFAULT_ROLES.button, BRadiusToken::Sm);
    assert_eq!(DEFAULT_ROLES.field, BRadiusToken::Sm);
    assert_eq!(DEFAULT_ROLES.menu_item, BRadiusToken::Sm);
    assert_eq!(DEFAULT_ROLES.tab, BRadiusToken::Md);
    assert_eq!(DEFAULT_ROLES.popover, BRadiusToken::Md);
    assert_eq!(DEFAULT_ROLES.region, BRadiusToken::Md);
    assert_eq!(DEFAULT_ROLES.card, BRadiusToken::Lg);
    assert_eq!(DEFAULT_ROLES.modal, BRadiusToken::Xl);
}

#[test]
fn resolved_colors_consume_existing_palette_defaults() {
    for name in BPaletteName::ALL.iter().copied() {
        let defaults = ui_style_defaults(name);
        let palette = palette_for(name);
        assert_eq!(defaults.colors, BResolvedColors::from_palette(palette));
    }
}
