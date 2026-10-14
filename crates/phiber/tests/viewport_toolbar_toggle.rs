//! View ▸ Toolbar (T): the viewport's tool rail hides and shows, Blender's
//! `SpaceView3D.show_region_toolbar`. Shown by default — the same show/hide
//! family as the sidebar (N) — and each editor flips its own.

use phiber::v2::host::BHostType;
use phiber::v2::part::toolbar::BToolbarShown;
use phiber::v2::state::action::apply_action;
use phiber::v2::state::{BKey as _, BState, OpKey};
use phiber::v2::view::prim::BAction;

#[test]
fn t_toggles_the_tool_rail_on_by_default() {
    let mut s = BState::new();
    // Shown until turned off, as Blender's startup file has the tool column.
    assert!(BToolbarShown(BHostType::View3D).on(&s), "the toolbar is shown by default");

    // The menu entry flips the showing editor's key, on-by-default (the `!`
    // sense in BFlipRule): View ▸ Toolbar fires `OpKey::Flip` with `!<key>`.
    let flip = BAction::Set(OpKey::Flip.path(), format!("!{}", BToolbarShown::view3d_key().encode()));
    apply_action(&mut s, &flip);
    assert!(!BToolbarShown(BHostType::View3D).on(&s), "T hides it");
    apply_action(&mut s, &flip);
    assert!(BToolbarShown(BHostType::View3D).on(&s), "T again shows it");

    // Per editor: hiding the viewport's toolbar leaves the Image editor's shown.
    apply_action(&mut s, &flip);
    assert!(!BToolbarShown(BHostType::View3D).on(&s));
    assert!(BToolbarShown(BHostType::Image).on(&s), "the Image editor keeps its own toolbar");
}

#[test]
fn the_hidden_toolbar_part_is_empty() {
    use phiber::v2::view::{BView, BViewItem};
    let mut s = BState::new();
    let toolbar = phiber::v2::part::toolbar::BToolbar::new(BHostType::View3D);
    assert!(!matches!(toolbar.build(&s), BView::Empty), "shown: a rail");
    // Turn it off.
    s.set(&BToolbarShown::view3d_key(), "false");
    assert!(matches!(toolbar.build(&s), BView::Empty), "hidden: nothing to draw");
}

#[test]
fn the_menus_bind_t_to_the_toolbar() {
    // The keymap derives its bindings from the menu shortcuts, so a tog_host_on
    // entry with shortcut "T" binds T to the toolbar toggle.
    let src = include_str!("../src/v2/part/menubar/editor/view3d/mod.rs");
    assert!(src.contains(r#"tog_host_on("Toolbar", "T", crate::v2::part::toolbar::BToolbarShown::key_for)"#));
    let img = include_str!("../src/v2/part/menubar/editor/image/set.rs");
    assert!(img.contains(r#"tog_host_on("Toolbar", "T", crate::v2::part::toolbar::BToolbarShown::key_for)"#));
}
