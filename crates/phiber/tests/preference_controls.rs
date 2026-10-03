//! Focused examples for descriptor-backed B-prefixed preference controls.
//! Validates: Requirements 9.1-9.5, 10.1-10.3.

use phiber::v2::{
    part::props::pref::{BPreferenceControls, BPreferenceEditor, BPreferenceRow, SECTIONS},
    state::{BState, action::apply_action},
    theme::{BPaletteName, BStyleParseError, BStyleParser, ui_style_defaults},
    view::{BAction, BView, prim::BCtrlKind},
};

fn row(target: &str) -> BPreferenceRow {
    SECTIONS
        .iter()
        .flat_map(|section| section.rows())
        .find(|row| row.target() == target)
        .unwrap_or_else(|| panic!("missing preference row {target}"))
}

fn set_value(action: BAction, expected_row: BPreferenceRow) -> String {
    let BAction::Set(key, value) = action else {
        panic!("preference edit must emit BAction::Set")
    };
    assert_eq!(&key, expected_row.key());
    value
}

#[test]
fn number_field_accepts_display_units_and_exposes_b_step_controls() {
    let defaults = ui_style_defaults(BPaletteName::Graphite);
    let row = row("radii.xs");
    let mut state = BState::new();
    state.set(row.key(), "23");
    let controls = BPreferenceControls::for_row(row, &state, defaults);
    let BPreferenceEditor::Field(field) = controls.editor else {
        panic!("numeric descriptor must use BPreferenceField")
    };

    assert_eq!(field.text(), "23 px");
    assert_eq!(set_value(field.commit("24 px").unwrap(), row), "24");
    assert_eq!(set_value(field.decrease_action(), row), "22");
    assert_eq!(set_value(field.increase_action(), row), "24");

    // The view is the Properties control family now: a `BCtrl` of the kind the
    // descriptor describes, and the reset beside it. It was a text field with a
    // hand-built "Decrease" and "Increase" either side — three widgets
    // reimplemented beside three that existed, and the copies were the ones
    // without the stepper. The model above is untouched and still drives the
    // round trip; only what gets painted changed.
    let BView::Row { list } = controls.view(&state) else {
        panic!("B controls must compose as a BView row")
    };
    assert_eq!(list.len(), 2, "the editor and its reset, not a field and two buttons");
    let BView::Ctrl(ctrl) = &list[0] else { panic!("the editor is a control") };
    assert_eq!(ctrl.key, *row.key(), "on this preference's key");
    assert!(
        matches!(ctrl.kind, BCtrlKind::Number { .. }),
        "a numeric descriptor draws the number control, which carries its own step either side: {:?}",
        ctrl.kind
    );
    // And its bounds are the descriptor's, so it cannot step past what the
    // descriptor would then reject.
    let (BCtrlKind::Number { min, max, .. }, BStyleParser::Number { min: lo, max: hi }) = (ctrl.kind, row.parser()) else {
        panic!("a number control on a number parser")
    };
    assert_eq!((min, max), (lo, hi), "the control is bounded by the descriptor");
    assert!(
        matches!(&list[1], BView::Button(ctl)
            if matches!(&ctl.action, Some(BAction::Reset(key)) if key == row.key()))
    );
}

#[test]
fn fractional_and_color_fields_step_within_descriptor_bounds() {
    let defaults = ui_style_defaults(BPaletteName::Graphite);
    let mut state = BState::new();

    let fractional = row("effects.gradient_lift");
    state.set(fractional.key(), "0");
    let BPreferenceEditor::Field(field) =
        BPreferenceControls::for_row(fractional, &state, defaults).editor
    else {
        panic!()
    };
    assert_eq!(set_value(field.decrease_action(), fractional), "-0.02");
    assert_eq!(set_value(field.increase_action(), fractional), "0.02");

    let color = row("colors.bg_app");
    state.set(color.key(), "10, 20, 30, 1");
    let BPreferenceEditor::Field(field) =
        BPreferenceControls::for_row(color, &state, defaults).editor
    else {
        panic!()
    };
    assert_eq!(set_value(field.decrease_action(), color), "10, 20, 29, 1");
    assert_eq!(set_value(field.increase_action(), color), "10, 20, 31, 1");
    assert_eq!(field.commit("invalid"), Err(BStyleParseError::Malformed));
}

#[test]
fn selector_toggle_and_reset_use_the_canonical_action_reducer() {
    let defaults = ui_style_defaults(BPaletteName::Graphite);
    let mut state = BState::new();

    let shadow = row("effects.surface_shadow");
    let BPreferenceEditor::Selector(selector) =
        BPreferenceControls::for_row(shadow, &state, defaults).editor
    else {
        panic!()
    };
    let select = selector.select("extra_large").unwrap();
    assert_eq!(set_value(select.clone(), shadow), "extra_large");
    assert_eq!(
        selector.select("foreign-kit-choice"),
        Err(BStyleParseError::Malformed)
    );
    apply_action(&mut state, &select);
    assert_eq!(state.get(shadow.key()), Some("extra_large"));

    let gradients = row("effects.gradients");
    state.set(gradients.key(), "false");
    let controls = BPreferenceControls::for_row(gradients, &state, defaults);
    let BPreferenceEditor::Toggle(toggle) = controls.editor else {
        panic!()
    };
    let toggle_action = toggle.action();
    assert_eq!(set_value(toggle_action.clone(), gradients), "true");
    apply_action(&mut state, &toggle_action);
    assert_eq!(state.get(gradients.key()), Some("true"));

    assert!(controls.reset.enabled());
    let reset = controls.reset.action();
    assert!(matches!(&reset, BAction::Reset(key) if key == gradients.key()));
    apply_action(&mut state, &reset);
    assert_eq!(state.get(gradients.key()), None);
}

#[test]
fn malformed_override_displays_effective_default_but_remains_resettable() {
    let defaults = ui_style_defaults(BPaletteName::Sphere);
    let row = row("radii.xs");
    let mut state = BState::new();
    state.set(row.key(), "not-a-number");

    let controls = BPreferenceControls::for_row(row, &state, defaults);
    let BPreferenceEditor::Field(field) = controls.editor else {
        panic!()
    };
    assert_eq!(field.text(), format!("{} px", defaults.radii.xs));
    assert!(controls.reset.enabled());
}
