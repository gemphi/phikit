//! Style-defaults tests. Split out of `style.rs` so the parent stays under
//! the 200-line cap.

use super::*;
use crate::v2::theme::palette::{BPaletteName, palette_for};

#[test]
fn every_theme_owns_its_radius_scale_effects_and_seam() {
    let themes = [
        (BPaletteName::Sphere, SOFT_RADII, SPHERE_EFFECTS, 4.0),
        (BPaletteName::Graphite, TIGHT_RADII, GRAPHITE_EFFECTS, 4.0),
        (BPaletteName::Tokyo, MEDIUM_RADII, TOKYO_EFFECTS, 6.0),
        (BPaletteName::Moss, ROUND_RADII, MOSS_EFFECTS, 8.0),
        (BPaletteName::Ember, SQUARE_RADII, EMBER_EFFECTS, 2.0),
        (BPaletteName::Plum, PILL_RADII, PLUM_EFFECTS, 10.0),
        (BPaletteName::Onyx, MEDIUM_RADII, ONYX_EFFECTS, 4.0),
        (BPaletteName::Grey, ROUND_RADII, GREY_EFFECTS, 6.0),
        (BPaletteName::Silver, MEDIUM_RADII, SILVER_EFFECTS, 6.0),
        (BPaletteName::Apple, MEDIUM_RADII, APPLE_EFFECTS, 4.0),
        (BPaletteName::AppleDark, MEDIUM_RADII, APPLE_DARK_EFFECTS, 4.0),
        (BPaletteName::Swift, SOFT_RADII, SWIFT_EFFECTS, 4.0),
        (BPaletteName::Barbie, PILL_RADII, BARBIE_EFFECTS, 8.0),
        (BPaletteName::Fenty, ROUND_RADII, FENTY_EFFECTS, 6.0),
        (BPaletteName::Cyberpunk, SQUARE_RADII, CYBERPUNK_EFFECTS, 2.0),
        (BPaletteName::Chanel, TIGHT_RADII, CHANEL_EFFECTS, 4.0),
    ];
    assert_eq!(themes.len(), BPaletteName::ALL.len());
    assert!(BPaletteName::ALL.len() <= BPaletteName::MAX, "at most sixteen themes");
    for (name, radii, effects, gap) in themes {
        let defaults = ui_style_defaults(name);
        assert_eq!(defaults.radii, radii, "{name:?}");
        assert_eq!(defaults.effects, effects, "{name:?}");
        assert_eq!(defaults.resize.gap, gap, "{name:?}");
        assert_eq!(defaults.resize.inset, -(defaults.resize.hit_width + gap) / 2.0, "{name:?}: the hit band stays centred on the seam");
        assert_eq!(defaults.colors.ladder, palette_for(name).ladder, "{name:?}");
        // The light mode changes colours and nothing else: the same scale,
        // effects and gap, on the ladder read the other way up.
        let light = ui_style_defaults_in(name, BThemeMode::Light);
        assert_eq!((light.radii, light.effects, light.resize.gap), (radii, effects, gap), "{name:?}");
        assert!(light.colors.ladder.light && !defaults.colors.ladder.light, "{name:?}");
        assert_eq!(light.colors.ladder, super::super::palette::palette_in(name, BThemeMode::Light).ladder, "{name:?}");
        // Sphere's scale is the one the interface was drawn on.
        if name == BPaletteName::DEFAULT {
            assert_eq!(radii, BRadiusScale::rungs(4.0, 7.0, 10.0, 14.0, 18.0));
        }
    }
}

#[test]
fn resize_defaults_preserve_usability() {
    let resize = SPHERE_UI_STYLE_DEFAULTS.resize;
    // The seam is 3 px thinner than the 4 px gap at rest and 2 px thicker
    // than it under the pointer; the 14 px hit band stays much wider so the
    // seam is easy to grab.
    assert_eq!((resize.rest_thickness(), resize.hover_thickness(), resize.hit_width), (1.0, 6.0, 14.0));
    // The hit band must be wider than the visible line.
    assert!(resize.hit_width >= resize.hover_thickness());
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
    // Every cap-shaped control on a strip carries one radius, so a header
    // reads as one row of parts rather than a mix of shapes.
    for (what, token) in [
        ("button", DEFAULT_ROLES.button),
        ("field", DEFAULT_ROLES.field),
        ("menu item", DEFAULT_ROLES.menu_item),
        ("tab", DEFAULT_ROLES.tab),
    ] {
        assert_eq!(token, BRadiusToken::Sm, "{what}");
    }
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
