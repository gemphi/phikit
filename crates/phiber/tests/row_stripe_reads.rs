//! An alternating row has to be visible on every surface a striped list sits
//! on, in every theme.
//!
//! `row_alt` is a fixed rung of the shade ladder, so it is one step over some
//! surfaces and exactly equal to others. Today every striped list — the Info
//! log, the Spreadsheet, the Outliner, the welcome window's recent files and
//! the command palette — sits on a body or a modal, where the stripe reads.
//! A card is the shade the stripe is, so a striped list put inside one would
//! have invisible stripes. This test is the guard: it fails the moment that
//! becomes possible, rather than the moment somebody notices.

use phiber::v2::state::BState;
use phiber::v2::theme::{
    BPaletteName, BSurfaceTone, BThemeMode, resolve_ui_style, ui_style_defaults_in,
};

/// The surfaces a striped list is drawn on, and what puts it there.
const STRIPED_ON: &[(BSurfaceTone, &str)] = &[
    (BSurfaceTone::Body, "the Info log, the Spreadsheet and the Outliner"),
    (BSurfaceTone::Modal, "the welcome window's recent files and the command palette"),
];

/// Surfaces no striped list may be drawn on, because the stripe is this
/// shade: put one here and the banding vanishes.
const NOT_STRIPED_ON: &[(BSurfaceTone, &str)] = &[
    (BSurfaceTone::Card, "a panel card"),
    (BSurfaceTone::Section, "a property section"),
];

#[test]
fn a_stripe_reads_against_every_surface_it_is_drawn_on() {
    for name in BPaletteName::ALL.iter().copied() {
        for mode in [BThemeMode::Dark, BThemeMode::Light] {
            let style = resolve_ui_style(ui_style_defaults_in(name, mode), &BState::new());
            let stripe = style.colors.row_alt;
            for (tone, who) in STRIPED_ON {
                assert_ne!(
                    stripe,
                    style.surface(*tone).bottom,
                    "{name:?}/{mode:?}: the stripe vanishes on {tone:?} — {who}"
                );
            }
        }
    }
}

#[test]
fn the_surfaces_a_stripe_would_vanish_on_are_the_ones_we_keep_it_off() {
    // Not a wish — a statement of why `STRIPED_ON` is the list it is. If this
    // stops holding, `row_alt` became relative to its surface and this whole
    // file can go.
    let style = resolve_ui_style(
        ui_style_defaults_in(BPaletteName::Sphere, BThemeMode::Dark),
        &BState::new(),
    );
    for (tone, who) in NOT_STRIPED_ON {
        assert_eq!(
            style.colors.row_alt,
            style.surface(*tone).bottom,
            "{tone:?} ({who}) was the stripe's own shade; if it no longer is, revisit STRIPED_ON"
        );
    }
}

#[test]
fn a_stripe_is_one_step_off_the_body_it_bands() {
    for name in BPaletteName::ALL.iter().copied() {
        for mode in [BThemeMode::Dark, BThemeMode::Light] {
            let style = resolve_ui_style(ui_style_defaults_in(name, mode), &BState::new());
            let body = style.surface(BSurfaceTone::Body).bottom;
            assert_eq!(
                style.colors.row_alt,
                style.step(body, 1),
                "{name:?}/{mode:?}: a stripe is one ladder step over the body, no more"
            );
        }
    }
}
