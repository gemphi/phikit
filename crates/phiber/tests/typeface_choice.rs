//! The typeface is a Preferences choice — *Typography ▸ Family*, nine
//! faces: Phi-Soft (the platform's own, bundled, and the default), the
//! system's, two Mac faces, Inter, and VS Code's four code faces — resolved
//! like every other style value and applied once at the root.

use phiber::v2::{
    state::{BState, key::ui_style as key},
    theme::{BFontFamily, BPaletteName, BPreferenceControl, UI_STYLE_DESCRIPTORS, resolve_ui_style, ui_style_defaults},
};

#[test]
fn nine_faces_each_with_a_name_a_stack_behind_it_and_a_slug_that_round_trips() {
    assert_eq!(BFontFamily::ALL.len(), 9);
    let code: Vec<_> = BFontFamily::ALL.iter().filter(|family| family.monospace()).collect();
    assert_eq!(code.len(), 4, "five text faces, four code faces");
    for family in BFontFamily::ALL {
        assert!(!family.family().is_empty() && !family.label().is_empty());
        assert!(!family.fallbacks().is_empty(), "{family:?} has a stack to fall back down");
        assert!(!family.fallbacks().contains(&family.family()), "{family:?} does not fall back to itself");
        assert_eq!(BFontFamily::from_slug(family.slug()), Some(family));
        // A code face falls back to code faces, a text face to text faces.
        let generic = if family.monospace() { "monospace" } else { "sans-serif" };
        assert!(
            family == BFontFamily::System || family.fallbacks().contains(&generic),
            "{family:?} ends on the generic {generic}"
        );
    }
    assert_eq!(BFontFamily::DEFAULT, BFontFamily::PhiSoft, "the platform's own face, which ships in the app");
    assert_eq!(BFontFamily::from_slug("comic_sans"), None);
}

#[test]
fn the_family_is_a_typography_choice_that_resolves_from_state() {
    let descriptor = UI_STYLE_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.target == "typography.family")
        .expect("the family is a style descriptor");
    assert_eq!((descriptor.preference.section, descriptor.preference.label), ("Typography", "Family"));
    assert_eq!(descriptor.preference.control, BPreferenceControl::Choice);
    let labels: Vec<&str> = descriptor.choices().iter().map(|choice| choice.label).collect();
    assert_eq!(labels, BFontFamily::ALL.iter().map(|family| family.label()).collect::<Vec<_>>());

    let defaults = ui_style_defaults(BPaletteName::Sphere);
    let mut state = BState::new();
    assert_eq!(resolve_ui_style(defaults, &state).typography.family, BFontFamily::PhiSoft, "Phi-Soft until another is chosen");
    state.set(&key::TYPOGRAPHY_FAMILY, "menlo");
    assert_eq!(resolve_ui_style(defaults, &state).typography.family, BFontFamily::Menlo);
    state.set(&key::TYPOGRAPHY_FAMILY, "papyrus");
    assert_eq!(resolve_ui_style(defaults, &state).typography.family, BFontFamily::PhiSoft, "an unknown face is ignored");
    for palette in BPaletteName::ALL.iter().copied() {
        assert_eq!(ui_style_defaults(palette).typography.family, BFontFamily::PhiSoft, "{palette:?}: every theme starts on Phi-Soft");
    }
}

#[test]
fn the_root_sets_the_family_once_for_every_letter_under_it() {
    let backend = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/v2/view/draw/backend.rs")).unwrap();
    assert!(backend.contains(".font_family(family.family())"), "the root takes the chosen family");
    assert!(backend.contains("font_fallbacks"), "and its stack");
    let others = ["view/draw/button.rs", "view/draw/field.rs", "view/draw/card.rs", "view/draw/tab/mod.rs"];
    for file in others {
        let source = std::fs::read_to_string(format!("{}/src/v2/{file}", env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert!(!source.contains("font_family("), "{file} inherits the root's face rather than naming one");
    }
}
