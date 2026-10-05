//! A vertical strip is the colour of the content beside it, and the tabs carry
//! the distinction.
//!
//! Blender's, read rather than reasoned. `region_background_color_id`
//! (`editors/screen/area.cc:3012`) returns `TH_HEADER` for a header,
//! `TH_PREVIEW_BACK` for a preview, and **`TH_BACK` for everything else** —
//! `RGN_TYPE_NAV_BAR` among them. So the Properties navigation bar is painted in
//! exactly the same colour as the panel beside it; Blender has no darker rail
//! shade anywhere.
//!
//! What separates the tabs is `wcol_tab` in `userdef_default_theme.c`:
//! `inner = 0x1d1d1d` for an unselected tab and `inner_sel = 0x303030` for the
//! selected one, against a `space_properties.back` of `0x303030`. An unselected
//! tab is a darker CHIP; the selected one is the background's own shade, so it
//! reads as a hole cut onto the panel.
//!
//! phiber had the same two shades with the roles swapped — the whole strip
//! darkened (`step(body, -1)`) and the active tab at the body's shade — which
//! made the navigation bar read as a separate panel bolted to the side of the
//! editor. Measured off a screenshot: the strip was `#1B1823` against a panel of
//! `#2C2739`.

use phiber::v2::host::BHostType;
use phiber::v2::state::BState;
use phiber::v2::theme::{resolve_ui_style, ui_style_defaults_for, BPaletteName, BThemeMode};

fn style_for(palette: BPaletteName, mode: BThemeMode) -> phiber::v2::theme::BResolvedUiStyle {
    let mut state = BState::new();
    state.set(phiber::v2::state::AppKey::Theme, palette.slug());
    state.set(phiber::v2::state::AppKey::ThemeMode, mode.slug());
    resolve_ui_style(ui_style_defaults_for(&state), &state)
}

#[test]
fn the_rail_is_level_with_the_content_beside_it() {
    // `TH_BACK` for the nav bar as for the region it sits against — one colour,
    // in every palette and both modes, so no theme reintroduces the seam.
    for palette in BPaletteName::ALL {
        for mode in BThemeMode::ALL {
            let style = style_for(*palette, mode);
            for host in [BHostType::Buts, BHostType::View3D, BHostType::Activity] {
                let rail = style.rail_surface(host);
                let body = style.host_background(host);
                assert_eq!(
                    rail.bottom, body,
                    "{palette:?}/{mode:?}: the {host:?} rail is not the colour of the content beside it"
                );
                assert!(!rail.enabled, "a rail is flat — a sweep on one side of the join makes two surfaces");
            }
        }
    }
}

#[test]
fn a_resting_tab_is_the_strip_and_the_selected_one_is_the_panel() {
    // `wcol_tab.inner` sits UNDER `TH_BACK` and `inner_sel` sits ON it — one
    // step apart, and the direction is the whole point. Which of the two the
    // step lands on is what changed: it used to be the resting tab's chip, so
    // every unselected tab drew a square and the selected tab drew nothing,
    // and the selection was the one tab with no mark on it. It is the BODY's
    // now, so a resting tab is strip and the selected tab carries the shade.
    //
    // Stated as a step rather than as a lightness comparison, because "darker"
    // is only Blender's answer in Blender's one dark theme, and this has
    // sixteen palettes in two modes.
    for palette in BPaletteName::ALL {
        for mode in BThemeMode::ALL {
            let style = style_for(*palette, mode);
            let strip = style.rail_surface(BHostType::Buts).bottom;
            let panel = style.host_body_surface(BHostType::Buts).bottom;
            let active = style.tab_active_gradient(BHostType::Buts).bottom;

            assert_eq!(
                panel,
                style.step(strip, -1),
                "{palette:?}/{mode:?}: the panel is one step off the strip (`wcol_tab.inner` under `TH_BACK`)"
            );
            assert_eq!(
                active, panel,
                "{palette:?}/{mode:?}: the selected tab is the panel's own shade, so it reads as a hole cut onto it"
            );
            assert_ne!(
                active, strip,
                "{palette:?}/{mode:?}: and it must not be the strip, or nothing marks the selection"
            );
        }
    }
}

/// Every secondary activity column is a bordered card — Profile included.
///
/// Profile's rows are built differently from the rest (an identity row, then
/// four folding sections) and it returned a bare column because of it, so it was
/// the one column in the strip with no container border and no heading. The rows
/// differ; the container does not.
#[test]
fn every_secondary_activity_column_has_the_same_container() {
    use phiber::v2::part::activity::BActivityList;
    use phiber::v2::state::ActivityKey;
    use phiber::v2::view::BView;

    let mut checked = 0;
    for entry in phiber::v2::part::activity::set::ENTRIES.iter().filter(|e| e.is_grouping()) {
        let mut state = BState::new();
        state.set(ActivityKey::Group, entry.label);
        let strip = BActivityList::new().group_strip(&state);
        match strip {
            BView::Card(card) => {
                assert_eq!(
                    card.title(),
                    Some(entry.label),
                    "{}'s column is headed by its own name",
                    entry.label
                );
                checked += 1;
            }
            // A group with nothing to list draws nothing, which is not a
            // missing container.
            BView::Empty => {}
            other => panic!("{}'s column is a {other:?}, not the card every other column is", entry.label),
        }
    }
    assert!(checked >= 2, "the strip has several groups to compare, got {checked}");
}
