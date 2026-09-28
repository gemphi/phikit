//! One elevation scale: a modal casts further than a dropdown, a dropdown
//! further than floating chrome, and nothing in the plane of the workspace
//! casts at all — in every theme.
//!
//! Before this, `BEffects` had a single `surface_shadow`, so a modal, a
//! popover and the action log's drawer cast the same shadow while the
//! viewport's floating panes cast none. Nothing in the window said which
//! surface was nearer the reader.

use phiber::v2::state::BState;
use phiber::v2::theme::{
    BElevation, BPaletteName, BShadowStyle, BThemeMode, resolve_ui_style, ui_style_defaults_in,
};

fn rung(shadow: BShadowStyle) -> usize {
    BShadowStyle::ALL.iter().position(|s| *s == shadow).expect("a known rung")
}

fn every_style() -> Vec<(BPaletteName, BThemeMode, phiber::v2::theme::BResolvedUiStyle)> {
    let mut all = Vec::new();
    for name in BPaletteName::ALL.iter().copied() {
        for mode in [BThemeMode::Dark, BThemeMode::Light] {
            all.push((name, mode, resolve_ui_style(ui_style_defaults_in(name, mode), &BState::new())));
        }
    }
    all
}

#[test]
fn the_scale_never_inverts_in_any_theme() {
    for (name, mode, style) in every_style() {
        let mut previous = 0usize;
        for level in BElevation::ALL {
            let here = rung(style.shadow_for(level));
            assert!(
                here >= previous,
                "{name:?}/{mode:?}: {level:?} casts under the level beneath it"
            );
            previous = here;
        }
    }
}

#[test]
fn the_plane_of_the_workspace_never_casts() {
    for (name, mode, style) in every_style() {
        assert_eq!(
            style.shadow_for(BElevation::Base),
            BShadowStyle::None,
            "{name:?}/{mode:?}: a zone, an area and a docked region are in the plane"
        );
    }
}

#[test]
fn a_modal_carries_the_themes_own_shadow() {
    for (name, mode, style) in every_style() {
        assert_eq!(
            style.shadow_for(BElevation::Modal),
            style.surface_shadow(),
            "{name:?}/{mode:?}: the theme's shadow is the ceiling, and a modal carries it"
        );
    }
}

#[test]
fn a_flat_theme_stays_flat_at_every_level() {
    for (name, mode, style) in every_style() {
        if style.surface_shadow() != BShadowStyle::None {
            continue;
        }
        for level in BElevation::ALL {
            assert_eq!(
                style.shadow_for(level),
                BShadowStyle::None,
                "{name:?}/{mode:?}: a flat theme is flat at {level:?}"
            );
        }
    }
}

#[test]
fn a_modal_outcasts_a_dropdown_wherever_the_theme_casts_at_all() {
    // The point of the scale: on a theme with room to show it, the three
    // lifted levels are three different depths, not one.
    let style = resolve_ui_style(
        ui_style_defaults_in(BPaletteName::Sphere, BThemeMode::Dark),
        &BState::new(),
    );
    assert_eq!(style.surface_shadow(), BShadowStyle::ExtraLarge, "Sphere is the deepest theme");
    let depths: Vec<_> = [BElevation::Raised, BElevation::Floating, BElevation::Overlay, BElevation::Modal]
        .into_iter()
        .map(|level| rung(style.shadow_for(level)))
        .collect();
    assert_eq!(depths, vec![1, 2, 3, 4], "four lifted levels, four depths");
}
