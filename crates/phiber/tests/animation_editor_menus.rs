//! The animation editors' menus are Blender 5.0's (`DOPESHEET_MT_*`,
//! `GRAPH_MT_*`, `NLA_MT_*`, labels and shortcuts from the operators and the
//! default keymap): the menu row of each, entries in order, and the keys
//! that run them over the editor — N opens the Dope Sheet's sidebar, Alt P
//! clears the preview range, M adds a marker; an entry phiber cannot do yet
//! says so rather than doing nothing.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::menubar::item::BMenuSpec;
use phiber::v2::part::sidebar::show::BSidebarShown;
use phiber::v2::scene::marker::BMarkerList;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{CtxKey, ModalKey};

fn titles(host: BHostType) -> Vec<&'static str> {
    BMenuSpec::list_for(host).iter().map(|m| m.label).collect()
}

fn entries(host: BHostType, menu: &str) -> Vec<&'static str> {
    let spec = BMenuSpec::list_for(host).iter().find(|m| m.label == menu).unwrap_or_else(|| panic!("{menu}"));
    spec.items.iter().map(|i| i.label).filter(|l| !l.is_empty()).collect()
}

#[test]
fn each_editor_has_blenders_menus_in_blenders_order() {
    // The Action menu (and the Channel menu in the Action Editor) is drawn
    // only over an action (`action_menu_push_down_and_stash.rs`).
    assert_eq!(titles(BHostType::Action), ["View", "Select", "Marker", "Channel", "Key", "Action"]);
    assert_eq!(entries(BHostType::Action, "Action"), ["Merge Animation", "Separate Slots", "Move Slots to new Action", "Push Down Action", "Stash Action"]);
    assert_eq!(titles(BHostType::Ipo), ["View", "Select", "Marker", "Channel", "Key"]);
    assert_eq!(titles(BHostType::Drivers), ["View", "Select", "Channel", "Key"], "the Drivers editor has no markers");
    assert_eq!(titles(BHostType::Nla), ["View", "Select", "Marker", "Add", "Tracks", "Strips"]);
    assert_eq!(
        entries(BHostType::Action, "Key"),
        ["Transform", "Snap", "Mirror", "Insert Keyframes", "Jump to Keyframes", "Copy Keyframes", "Paste Keyframes", "Paste Flipped", "Duplicate", "Delete Keyframes", "Keyframe Type", "Handle Type", "Interpolation Mode", "Easing Mode", "Clean Keyframes", "Bake Keyframes", "Discontinuity (Euler) Filter"]
    );
    assert_eq!(entries(BHostType::Nla, "Add"), ["Action", "Transition", "Sound", "Selected Objects"]);
    assert!(entries(BHostType::Ipo, "Channel").contains(&"Reveal Curves") && !entries(BHostType::Ipo, "Channel").contains(&"Delete Invalid Drivers"));
    assert!(entries(BHostType::Drivers, "Channel").contains(&"Delete Invalid Drivers"));
    for host in [BHostType::Action, BHostType::Ipo, BHostType::Drivers, BHostType::Nla] {
        assert!(BMenuSpec::list_for(host).iter().all(|m| !m.items.is_empty()), "{host:?}: no empty menu");
    }
}

#[gpui::test]
async fn keys_over_the_dope_sheet_run_its_menus(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(CtxKey::Space, BSpaceType::Animation.slug());
        app.state.set(CtxKey::PreviewRange, "true");
        cx.notify();
    });
    cx.run_until_parked();
    let window = cx.debug_bounds("time-window").expect("the Dope Sheet's window");
    cx.simulate_mouse_move(window.center(), None, Modifiers::none());
    cx.run_until_parked();
    let sidebar = |app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext| app.read_with(cx, |a, _| BSidebarShown(BHostType::Action).on(&a.state));

    // View ▸ Sidebar from the menu, then N puts it away.
    let view = cx.debug_bounds("btn-label-toggle-ctx.menu.action.View-text").expect("the View menu").center();
    cx.simulate_click(view, Modifiers::none());
    cx.run_until_parked();
    let item = cx.debug_bounds("menu-Sidebar").expect("View ▸ Sidebar").center();
    cx.simulate_click(item, Modifiers::none());
    cx.run_until_parked();
    assert!(sidebar(&app, cx), "the sidebar opens");
    cx.simulate_mouse_move(window.center(), None, Modifiers::none());
    cx.simulate_keystrokes("n");
    cx.run_until_parked();
    assert!(!sidebar(&app, cx), "N puts it away");

    // Alt P: Clear Preview Range.
    cx.simulate_keystrokes("alt-p");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::PreviewRange).map(str::to_string)), Some("false".into()));

    // M: Marker ▸ Add Marker, at the current frame.
    let before = app.read_with(cx, |a, _| BMarkerList::now(&a.state).list.len());
    cx.simulate_keystrokes("m");
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| BMarkerList::now(&a.state).list.len()), before + 1, "a marker added");

    // Ctrl C: Key ▸ Copy Keyframes, which says it is not here yet.
    cx.simulate_keystrokes("ctrl-c");
    cx.run_until_parked();
    let said = phiber::v2::state::log::latest().map(|e| e.text.clone()).unwrap_or_default();
    assert!(said.contains("Copy Keyframes") && said.contains("not available yet"), "{said}");
}
