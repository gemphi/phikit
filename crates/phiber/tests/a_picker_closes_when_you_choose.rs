//! Choosing a value closes the picker it was chosen from.
//!
//! Every enum control opens a list and its rows write the value
//! (`draw::ctrl`, `BCtrl::set`). Nothing closed the list: you picked GPU
//! Compute and the menu stayed open over the panel, with a tick beside the
//! item you had just chosen.
//!
//! The mechanism was already there and the picker was the one overlay it did
//! not name — `state::action` calls `menu::close_overlays` after every
//! committed `Set`, and that swept the menus and the area switchers but not
//! `ctx.buts.dropdown.*`. So this is about one prefix, and these are here so
//! it stays swept.
//!
//! Opening is a `Toggle`, which does not run the sweep — otherwise a picker
//! would close itself the instant it opened.

use phiber::v2::state::val::{V_FALSE, V_TRUE};
use phiber::v2::state::action::apply_action;
use phiber::v2::state::{BKey as _, BPath, BState, ButsKey, MenuKey};
use phiber::v2::view::prim::BAction;

/// The Device picker from the Render tab, as `prop_view` keys it.
fn device_open() -> BPath {
    ButsKey::Dropdown("device").path()
}

fn device_value() -> BPath {
    ButsKey::Value("device").path()
}

#[test]
fn a_toggle_opens_the_picker_and_leaves_it_open() {
    let mut state = BState::new();
    apply_action(&mut state, &BAction::Toggle(device_open()));
    assert_eq!(state.get(&device_open()), Some(V_TRUE), "the picker opens");
}

#[test]
fn choosing_a_value_closes_it() {
    let mut state = BState::new();
    apply_action(&mut state, &BAction::Toggle(device_open()));
    apply_action(&mut state, &BAction::Set(device_value(), "GPU".into()));
    assert_eq!(state.get(&device_value()), Some("GPU"), "the value is written");
    assert_ne!(state.get(&device_open()), Some(V_TRUE), "and the list is gone");
}

#[test]
fn choosing_in_one_picker_closes_every_other_one() {
    // Two open pickers is a state you can reach by opening one, clicking
    // another's anchor, and it is never what the user wants to look at.
    let mut state = BState::new();
    let other = ButsKey::Dropdown("engine").path();
    state.set(&device_open(), V_TRUE);
    state.set(&other, V_TRUE);
    apply_action(&mut state, &BAction::Set(device_value(), "CPU".into()));
    for key in [device_open(), other] {
        assert_ne!(state.get(&key), Some(V_TRUE), "{} stayed open", key.encode());
    }
}

#[test]
fn a_value_written_anywhere_closes_a_picker() {
    // The picker's rows are not the only thing that writes while one is
    // open: a keystroke, an operator, a click on the panel behind it. Any
    // committed write means the moment for choosing has passed.
    let mut state = BState::new();
    state.set(&device_open(), V_TRUE);
    apply_action(&mut state, &BAction::Set(ButsKey::Value("samples").path(), "64".into()));
    assert_ne!(state.get(&device_open()), Some(V_TRUE));
}

#[test]
fn the_menus_and_the_switchers_are_still_swept() {
    // The sweep gained a prefix; it did not lose one.
    let mut state = BState::new();
    let menu = MenuKey::Top("view3d", "View").path();
    state.set(&menu, V_TRUE);
    apply_action(&mut state, &BAction::Set(device_value(), "CPU".into()));
    assert_eq!(state.get(&menu), Some(V_FALSE), "a menu still goes down with a write");
}

#[test]
fn the_prefix_is_the_key_enums_own() {
    // Spelling it here would let the sweep and the key drift apart.
    assert!(
        device_open().encode().starts_with(&ButsKey::dropdown_prefix()),
        "{} is not under {}",
        device_open().encode(),
        ButsKey::dropdown_prefix()
    );
    // And a panel value is NOT under it, or every write would clear values.
    assert!(!device_value().encode().starts_with(&ButsKey::dropdown_prefix()));
}
