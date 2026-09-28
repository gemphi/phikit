//! Eleven themes in Preferences, each a seed ladder with its own radius
//! scale, sweeps, shadow and seam gap, in a dark and a light mode. Choosing
//! one in the Theme section is what the window draws with, the Dark / Light
//! switch in its heading turns every theme's ladder the other way up, every
//! rung of a radius scale reaches the roles that take it, and the values
//! Preferences shows are that theme's.

use phiber::v2::{
    part::pref::BPrefsItem,
    part::props::pref::{BPreferenceControls, BPreferenceEditor, BPreferenceRow, SECTIONS},
    host::BHostType,
    state::{AppKey, BPath, BState, action::apply_action, key::ui_style as key},
    theme::{
        BPaletteName, BRadiusScale, BRadiusToken, BResolvedUiStyle, BShadowStyle, BStyleRole, BThemeMode,
        MEDIUM_RADII, PILL_RADII, ROUND_RADII, SOFT_RADII, SQUARE_RADII, TIGHT_RADII, resolve_ui_style,
        ui_style_defaults, ui_style_defaults_for, ui_style_defaults_in,
    },
    view::{BAction, BView, BViewItem},
};

/// What each theme was approved with: scale, shadow, sweeps, seam gap.
const THEMES: [(BPaletteName, BRadiusScale, BShadowStyle, bool, f32); 11] = [
    (BPaletteName::Sphere, SOFT_RADII, BShadowStyle::ExtraLarge, true, 4.0),
    (BPaletteName::Graphite, TIGHT_RADII, BShadowStyle::None, false, 4.0),
    (BPaletteName::Tokyo, MEDIUM_RADII, BShadowStyle::Medium, true, 6.0),
    (BPaletteName::Onyx, MEDIUM_RADII, BShadowStyle::Large, true, 4.0),
    (BPaletteName::Grey, ROUND_RADII, BShadowStyle::Medium, true, 6.0),
    (BPaletteName::Apple, MEDIUM_RADII, BShadowStyle::Medium, false, 4.0),
    (BPaletteName::Swift, SOFT_RADII, BShadowStyle::Large, true, 4.0),
    (BPaletteName::Barbie, PILL_RADII, BShadowStyle::ExtraLarge, true, 8.0),
    (BPaletteName::Fenty, ROUND_RADII, BShadowStyle::Large, true, 6.0),
    (BPaletteName::Cyberpunk, SQUARE_RADII, BShadowStyle::None, false, 2.0),
    (BPaletteName::Chanel, TIGHT_RADII, BShadowStyle::Medium, true, 4.0),
];

/// The window's style for the theme and mode the state names — what
/// `backend.rs` does.
fn drawn_with(state: &BState) -> BResolvedUiStyle {
    resolve_ui_style(ui_style_defaults_for(state), state)
}

fn showing_themes() -> BState {
    let mut state = BState::new();
    state.set(&BPath::owned(vec![BHostType::Prefs.slug(), "navbar", "tab"]), "Appearance");
    state
}

/// The Theme section: whether it is open, its rows as (label, active, the
/// Use button's action), and the Dark / Light switch in its heading as
/// (label, active, action).
struct BPicker {
    open: bool,
    rows: Vec<(String, bool, BAction)>,
    switch: Vec<(String, bool, BAction)>,
}

fn picker(state: &BState) -> BPicker {
    let view = BPrefsItem::new().build(state);
    let BView::Card(column) = &view else { panic!("preferences are a column") };
    let BView::Card(first) = &column.body[0] else { panic!("the picker leads") };
    assert_eq!(first.title(), Some("Theme"));
    let switch = first
        .head
        .as_ref()
        .expect("a heading")
        .preset
        .iter()
        .map(|view| {
            let BView::Button(control) = view else { panic!("the switch is buttons") };
            (control.label.clone().unwrap_or_default(), control.active, control.action.clone().expect("a mode button acts"))
        })
        .collect();
    let rows = first
        .body
        .iter()
        .map(|row| {
            let BView::Split { list } = row else { panic!("a theme row is a split") };
            let BView::Label { text } = &list[0] else { panic!("label first") };
            let BView::Button(control) = &list[1] else { panic!("button second") };
            (text.clone(), control.active, control.action.clone().expect("a theme button acts"))
        })
        .collect();
    BPicker { open: !first.body.is_empty(), rows, switch }
}

fn row(target: &str) -> BPreferenceRow {
    SECTIONS.iter().flat_map(|section| section.rows()).find(|row| row.target() == target).unwrap_or_else(|| panic!("{target}"))
}

/// Clicking a colour or number row's field: one step up.
fn nudge(row: BPreferenceRow, state: &BState) -> BAction {
    let BPreferenceEditor::Field(field) = BPreferenceControls::for_row(row, state, ui_style_defaults_for(state)).editor else {
        panic!("{}: a field", row.target())
    };
    field.increase_action()
}

#[test]
fn preferences_offer_eleven_themes_in_an_accordion_and_choosing_one_is_what_the_window_draws_with() {
    let mut state = showing_themes();
    let first = picker(&state);
    assert!(first.open, "the section is open before anyone touches it");
    let labels: Vec<String> = first.rows.iter().map(|(label, _, _)| label.clone()).collect();
    assert_eq!(labels, ["Sphere", "Graphite", "Tokyo", "Onyx", "Grey", "Apple", "Swift", "Barbie", "Fenty", "Cyberpunk", "Chanel"]);
    for (name, radii, shadow, gradients, gap) in THEMES {
        let (_, _, choose) = picker(&state).rows.into_iter().find(|(label, _, _)| label == name.label()).unwrap();
        apply_action(&mut state, &choose);
        let after = picker(&state);
        let lit: Vec<String> = after.rows.into_iter().filter(|(_, active, _)| *active).map(|(label, _, _)| label).collect();
        assert_eq!(lit, [name.label()], "the chosen theme alone reads as active");
        let style = drawn_with(&state);
        assert_eq!(style.colors.ladder, ui_style_defaults(name).colors.ladder, "{name:?}: the window takes the theme's ladder");
        assert_eq!(style.radii, radii, "{name:?}");
        assert_eq!(style.surface_shadow(), shadow, "{name:?}");
        assert_eq!(style.effects.gradients, gradients, "{name:?}");
        assert_eq!(style.resize.gap, gap, "{name:?}: every seam in the window is the theme's gap");
    }
    // It folds like every other section, and unfolds again.
    let fold = phiber::v2::part::props::BPanelItem::expand_key(BHostType::Prefs, "Appearance", "Theme");
    state.set(&fold, phiber::v2::state::V_FALSE);
    assert!(!picker(&state).open, "folded: the heading alone");
    state.set(&fold, phiber::v2::state::V_TRUE);
    assert!(picker(&state).open);
}

#[test]
fn the_dark_light_switch_in_the_theme_heading_turns_every_theme_the_other_way_up() {
    let mut state = showing_themes();
    let switch = picker(&state).switch;
    let labels: Vec<&str> = switch.iter().map(|(label, _, _)| label.as_str()).collect();
    assert_eq!(labels, ["Dark", "Light"]);
    assert!(switch[0].1 && !switch[1].1, "dark is the mode in force until chosen otherwise");
    for (name, ..) in THEMES {
        state.set(AppKey::Theme, name.slug());
        let dark = drawn_with(&state);
        let to_light = picker(&state).switch[1].2.clone();
        apply_action(&mut state, &to_light);
        let light = drawn_with(&state);
        assert!(!dark.colors.ladder.light && light.colors.ladder.light, "{name:?}");
        assert_eq!(light.colors.ladder, ui_style_defaults_in(name, BThemeMode::Light).colors.ladder, "{name:?}");
        // Light reads from white: the window shade is near white, the text dark.
        assert!(light.colors.bg_app.2 > 85.0 && light.colors.text.2 < 35.0, "{name:?}: {:?} {:?}", light.colors.bg_app, light.colors.text);
        assert!(dark.colors.bg_app.2 < 20.0 && dark.colors.text.2 > 65.0, "{name:?}");
        // Colours change, nothing else: the same radii, effects and gap.
        assert_eq!((light.radii, light.effects, light.resize.gap), (dark.radii, dark.effects, dark.resize.gap), "{name:?}");
        assert!(picker(&state).switch[1].1 && !picker(&state).switch[0].1, "light reads as the mode in force");
        let to_dark = picker(&state).switch[0].2.clone();
        apply_action(&mut state, &to_dark);
        assert_eq!(drawn_with(&state).colors.ladder, dark.colors.ladder, "{name:?}: and back");
    }
    // Grey's light mode is the silver Tesla's dashboard reads in, with its
    // blue; Apple's is macOS's light window grey with the system blue.
    state.set(AppKey::Theme, BPaletteName::Grey.slug());
    state.set(AppKey::ThemeMode, "light");
    let grey = drawn_with(&state);
    assert_eq!(grey.colors.ladder.seed, [0xC4, 0xC6, 0xC9]);
    assert!((grey.colors.accent.0 - 223.8).abs() < 0.1, "Tesla's blue");
    state.set(AppKey::Theme, BPaletteName::Apple.slug());
    let apple = drawn_with(&state);
    assert_eq!(apple.colors.ladder.seed, [0xCB, 0xCB, 0xCD]);
    assert!((apple.colors.accent.0 - 211.3).abs() < 0.1, "#007AFF");
    state.set(AppKey::ThemeMode, "dark");
    assert_eq!(drawn_with(&state).colors.ladder.seed, [0x38, 0x38, 0x3A], "Apple's dark window grey");
}

#[test]
fn every_rung_of_a_themes_scale_reaches_the_roles_that_take_it() {
    for (name, radii, _, _, _) in THEMES {
        let mut state = BState::new();
        state.set(AppKey::Theme, name.slug());
        let style = drawn_with(&state);
        // One rung, one shape: a button, a field, a menu cap and a tab share
        // `sm`, so a header, a dropdown and a menu bar round alike.
        for role in [BStyleRole::Button, BStyleRole::Field, BStyleRole::MenuItem, BStyleRole::Tab] {
            assert_eq!(style.radius_for(role), radii.sm, "{name:?} {role:?}");
        }
        assert_eq!(style.radius_for(BStyleRole::Popover), radii.md, "{name:?}");
        assert_eq!(style.radius_for(BStyleRole::Region), radii.md, "{name:?}");
        assert_eq!(style.radius_for(BStyleRole::Card), radii.lg, "{name:?}");
        assert_eq!(style.radius_for(BStyleRole::Area), radii.lg, "{name:?}");
        assert_eq!(style.radius_for(BStyleRole::Modal), radii.xl, "{name:?}");
        assert_eq!(style.radius_for(BStyleRole::Workspace), 0.0, "{name:?}");
        for (token, rung) in [(BRadiusToken::Xs, radii.xs), (BRadiusToken::Sm, radii.sm), (BRadiusToken::Md, radii.md), (BRadiusToken::Lg, radii.lg), (BRadiusToken::Xl, radii.xl)] {
            assert_eq!(style.radii.resolve(token), rung, "{name:?} {token:?}");
        }
        assert!(radii.xs <= radii.sm && radii.sm <= radii.md && radii.md <= radii.lg && radii.lg <= radii.xl, "{name:?}: rungs climb");
        // Preferences shows the theme's own rungs, and a rung the user edits
        // moves every role on it together.
        let defaults = ui_style_defaults(name);
        for (target, rung) in [("radii.xs", radii.xs), ("radii.sm", radii.sm), ("radii.md", radii.md), ("radii.lg", radii.lg), ("radii.xl", radii.xl)] {
            let BPreferenceEditor::Field(field) = BPreferenceControls::for_row(row(target), &state, defaults).editor else { panic!() };
            assert_eq!(field.text(), format!("{rung} px"), "{name:?} {target}");
        }
        state.set(&key::RADIUS_SM, "3");
        let edited = drawn_with(&state);
        for role in [BStyleRole::Button, BStyleRole::Field, BStyleRole::MenuItem, BStyleRole::Tab] {
            assert_eq!(edited.radius_for(role), 3.0, "{name:?} {role:?}");
        }
        assert_eq!(edited.radius_for(BStyleRole::Modal), radii.xl, "{name:?}: the other rungs stay the theme's");
    }
}

#[test]
fn shadow_gradient_and_gap_preferences_show_each_themes_own_settings() {
    for (name, _, shadow, gradients, gap) in THEMES {
        let mut state = BState::new();
        state.set(AppKey::Theme, name.slug());
        let defaults = ui_style_defaults(name);
        let BPreferenceEditor::Selector(selector) = BPreferenceControls::for_row(row("effects.surface_shadow"), &state, defaults).editor else { panic!() };
        assert_eq!(selector.selected(), shadow.slug(), "{name:?}");
        let choices: Vec<(&str, &str)> = selector.choices().iter().map(|choice| (choice.value, choice.label)).collect();
        assert_eq!(choices, BShadowStyle::ALL.map(|style| (style.slug(), style.label())), "every shadow is offered, in order");
        let BPreferenceEditor::Toggle(toggle) = BPreferenceControls::for_row(row("effects.gradients"), &state, defaults).editor else { panic!() };
        assert_eq!(toggle.checked(), gradients, "{name:?}");
        let BPreferenceEditor::Field(field) = BPreferenceControls::for_row(row("resize.gap"), &state, defaults).editor else { panic!() };
        assert_eq!(field.text(), format!("{gap} px"), "{name:?}");
        // A chosen shadow round-trips through the selector and the resolver.
        for style in BShadowStyle::ALL {
            let action = selector.select(style.slug()).unwrap();
            apply_action(&mut state, &action);
            assert_eq!(drawn_with(&state).surface_shadow(), style, "{name:?} {style:?}");
        }
    }
}

/// Blender's `reset_default_theme` / preset load: a theme is every value of
/// it. A colour row nudged under one theme is an override, and picking
/// another theme — or its other mode — clears the overrides, so a header and
/// a viewport nudged under Onyx never stay Onyx under Barbie.
#[test]
fn picking_a_theme_or_its_mode_puts_every_value_of_that_theme_in_force() {
    let mut state = showing_themes();
    state.set(AppKey::Theme, BPaletteName::Onyx.slug());
    // Nudge two colour rows, as clicking them in the Colors section does.
    let header = row("colors.bg_header");
    let viewport = row("colors.bg_viewport");
    for nudged in [header, viewport] {
        let action = nudge(nudged, &state);
        apply_action(&mut state, &action);
    }
    let onyx = drawn_with(&state);
    assert_ne!(onyx.colors.bg_header, ui_style_defaults(BPaletteName::Onyx).colors.bg_header, "the nudge took");
    assert!(state.get(&key::COLOR_BG_HEADER).is_some() && state.get(&key::COLOR_BG_VIEWPORT).is_some());

    // Use Barbie: its own header and canvas, nothing of Onyx.
    let use_barbie = picker(&state).rows.iter().find(|(label, ..)| label == "Barbie").unwrap().2.clone();
    apply_action(&mut state, &use_barbie);
    let barbie = drawn_with(&state);
    let own = ui_style_defaults(BPaletteName::Barbie);
    assert_eq!((barbie.colors.bg_header, barbie.colors.bg_viewport), (own.colors.bg_header, own.colors.bg_viewport));
    assert!(state.iter_keys().all(|k| !k.starts_with("ctx.ui_style.")), "no override survives a theme pick");

    // The same for the mode switch.
    let action = nudge(viewport, &state);
    apply_action(&mut state, &action);
    assert!(state.get(&key::COLOR_BG_VIEWPORT).is_some());
    let to_light = picker(&state).switch[1].2.clone();
    apply_action(&mut state, &to_light);
    assert!(state.get(&key::COLOR_BG_VIEWPORT).is_none(), "the mode switch clears the nudge too");
    assert_eq!(drawn_with(&state).colors.bg_viewport, ui_style_defaults_in(BPaletteName::Barbie, BThemeMode::Light).colors.bg_viewport);
}
