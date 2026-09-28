//! Controls: every widget that edits a value goes through one vocabulary
//! (`BCtrl`) over one value category (`BValue`), so a property panel, a
//! filter menu and a script set the render engine the same way.

use phiber::v2::state::{BKey as _, CtxKey};
use phiber::v2::state::ButsKey;
use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::props::prop::BPropKind;
use phiber::v2::part::props::set::{TABS, sections_for};
use phiber::v2::state::log;
use phiber::v2::state::val::{BValue, BValueKind};
use phiber::v2::state::{BState};
use phiber::v2::view::BView;
use phiber::v2::view::prim::{BAction, BCtrlKind};

fn ctrls(view: &BView, out: &mut Vec<phiber::v2::view::prim::BCtrl>) {
    match view {
        BView::Ctrl(c) => out.push((**c).clone()),
        BView::Row { list } | BView::Column { list } | BView::Split { list } => {
            list.iter().for_each(|v| ctrls(v, out))
        }
        BView::Card(card) if matches!(card.kind, phiber::v2::view::prim::BCardKind::Column(_)) => card.body.iter().for_each(|v| ctrls(v, out)),
        _ => {}
    }
}

#[test]
fn every_property_kind_is_a_control_over_its_value_category() {
    let s = BState::new();
    let mut seen = std::collections::BTreeSet::new();
    for tab in TABS {
        for section in sections_for(BHostType::Buts, tab) {
            for prop in section.all_props() {
                let row = phiber::v2::part::props::prop_view::prop_row(prop, &s);
                let mut found = Vec::new();
                ctrls(&row, &mut found);
                assert!(!found.is_empty(), "{tab} ▸ {} has no control", prop.label);
                let expected: BValueKind = match prop.kind {
                    BPropKind::Bool => BValueKind::Bool,
                    BPropKind::Int { .. } => BValueKind::Int,
                    BPropKind::Float { .. } | BPropKind::Vec3 { .. } | BPropKind::Vec4 { .. } => BValueKind::Float,
                    BPropKind::Enum(_) => BValueKind::Enum,
                    BPropKind::Str | BPropKind::Pointer { .. } => BValueKind::Text,
                    BPropKind::Color { .. } => BValueKind::Color,
                };
                for c in &found {
                    assert_eq!(c.kind.value_kind(), expected, "{tab} ▸ {}", prop.label);
                    assert_eq!(c.value.kind(), expected, "{tab} ▸ {} holds the wrong category", prop.label);
                    seen.insert(format!("{:?}", c.kind.value_kind()));
                }
            }
        }
    }
    // The panels between them exercise the whole vocabulary.
    for kind in ["Bool", "Int", "Float", "Enum", "Text", "Color"] {
        assert!(seen.contains(kind), "no property uses a {kind} control");
    }
}

#[test]
fn choosing_phiray_is_one_set_of_one_typed_key() {
    let s = BState::new();
    let engine = sections_for(BHostType::Buts, "Render")
        .iter()
        .flat_map(|s| s.all_props())
        .find(|p| p.key == "engine")
        .expect("the Render tab declares the engine");
    let row = phiber::v2::part::props::prop_view::prop_row(engine, &s);
    let mut found = Vec::new();
    ctrls(&row, &mut found);
    let select = &found[0];
    assert_eq!(select.kind, BCtrlKind::Select);
    let phiray = select.options.iter().find(|o| o.key == "PHIRAY").expect("Phiray is an option");
    assert_eq!(phiray.label, "Phiray");

    let action = select.set(BValue::Enum(phiray.key.clone()));
    assert!(matches!(&action, BAction::Set(p, v) if p.encode() == "ctx.buts.engine" && v == "PHIRAY"));

    // Applied, the state holds it as an enum, and the log names it by label.
    let mut after = s.clone();
    phiber::v2::state::action::apply_action(&mut after, &action);
    assert_eq!(after.value(&ButsKey::Value("engine").path(), BValueKind::Enum), Some(BValue::Enum("PHIRAY".into())));
    assert_eq!(log::describe(&action, &s, &after).as_deref(), Some("Render Engine: Phiray"));

    // Read back, the control shows the choice.
    let row = phiber::v2::part::props::prop_view::prop_row(engine, &after);
    let mut found = Vec::new();
    ctrls(&row, &mut found);
    assert_eq!(found[0].chosen().map(|o| o.label.as_str()), Some("Phiray"));
}

#[test]
fn numbers_step_within_their_range_and_bools_flip() {
    use phiber::v2::view::prim::BCtrl;
    let s = BState::new();
    let key = ButsKey::Value("cycles.samples").path();
    let samples = BCtrl::read(BCtrlKind::Int { min: 0, max: 10 }, key.clone(), BValue::Int(9), &s);
    let up = samples.nudge(1.0).unwrap();
    assert!(matches!(&up, BAction::Set(_, v) if v == "10"));
    let mut s2 = s.clone();
    phiber::v2::state::action::apply_action(&mut s2, &up);
    let again = BCtrl::read(BCtrlKind::Int { min: 0, max: 10 }, key, BValue::Int(0), &s2);
    assert!(matches!(&again.nudge(1.0).unwrap(), BAction::Set(_, v) if v == "10"), "clamped at the top");

    let flag = BCtrl::read(BCtrlKind::Check, ButsKey::Value("use_x").path(), BValue::Bool(false), &s);
    assert!(matches!(&flag.flip(), BAction::Set(_, v) if v == "true"));
}

#[gpui::test]
async fn the_engine_dropdown_opens_a_menu_with_phiray_in_it(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, "layout");
        app.state.set(&ButsKey::NavbarTab.path(), "Render");
        app.state.set(&ButsKey::Dropdown("engine").path(), "true");
        cx.notify();
    });
    cx.run_until_parked();
    let ids: Vec<String> = cx.all_debug_bounds().into_iter().map(|(id, _)| id).collect();
    assert!(ids.iter().any(|id| id == "menu-Phiray"), "the engine menu lists Phiray: {:?}", ids.iter().filter(|i| i.starts_with("menu-")).collect::<Vec<_>>());
}
