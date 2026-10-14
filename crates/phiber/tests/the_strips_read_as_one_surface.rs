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
fn a_resting_tab_is_the_strip_and_the_selected_one_is_the_accordion() {
    // The strip and the panel beside it are one surface, in every palette
    // and both modes; a resting tab paints nothing on it; and the selected
    // tab is the accordion heading's surface (`heading_surface(Card)`), so it
    // is marked by wearing the colour of what it opens. Stated as surfaces
    // rather than as a lightness comparison, because "lighter" is only one
    // theme's answer and this has sixteen palettes in two modes.
    for palette in BPaletteName::ALL {
        for mode in BThemeMode::ALL {
            let style = style_for(*palette, mode);
            let strip = style.rail_surface(BHostType::Buts).bottom;
            let panel = style.host_body_surface(BHostType::Buts).bottom;
            let active = style.tab_active_gradient(BHostType::Buts);

            assert_eq!(panel, strip, "{palette:?}/{mode:?}: the strip is the panel's own surface, no colour behind it");
            assert_eq!(
                active,
                style.heading_surface(phiber::v2::theme::BSurfaceTone::Card),
                "{palette:?}/{mode:?}: the selected tab is the accordion heading's surface"
            );
            assert_ne!(
                active.bottom, strip,
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
