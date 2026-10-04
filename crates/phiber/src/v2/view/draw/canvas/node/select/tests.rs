//! Selection tests — split out so `mod.rs` stays inside the 240-line read
//! budget (`level_structure`), the way `canvas/time/row/` is.

use super::*;
use crate::phiphx::blend::shade::graph::{BlendNode, NODE_ACTIVE, NODE_SELECT};

fn node(name: &str, flag: i32) -> BlendNode {
    BlendNode { name: name.into(), flag, ..Default::default() }
}

#[test]
fn a_plain_press_on_an_unselected_node_selects_only_it() {
    // `node_select_single`: everything else off.
    let before = BNodeSelection::of(&["a", "b"]);
    let after = clicked(&before, Some("c"), false, BClick::Press).expect("a change");
    assert_eq!(after, BNodeSelection::of(&["c"]));
    assert_eq!(after.active(), Some("c"));
}

#[test]
fn a_plain_press_on_an_already_selected_node_changes_nothing() {
    // `select_passthrough`: "if the node is already selected, found = false".
    // This is what lets a press-and-drag carry a whole selection instead of
    // collapsing it to the node under the cursor first.
    let before = BNodeSelection::of(&["a", "b", "c"]);
    assert_eq!(clicked(&before, Some("b"), false, BClick::Press), None);
    assert_eq!(clicked(&before, Some("a"), false, BClick::Press), None);
}

#[test]
fn a_press_and_release_with_no_drag_narrows_a_group_to_one() {
    // The keymap's separate CLICK binding with `deselect_all=True`, which is
    // what makes passthrough survivable: the group still collapses, just on
    // the release.
    let before = BNodeSelection::of(&["a", "b", "c"]);
    assert_eq!(
        clicked(&before, Some("b"), false, BClick::Release),
        Some(BNodeSelection::of(&["b"])),
    );
    // One already selected: nothing to narrow.
    assert_eq!(clicked(&BNodeSelection::of(&["b"]), Some("b"), false, BClick::Release), None);
}

#[test]
fn clicking_empty_canvas_clears_the_selection() {
    assert_eq!(
        clicked(&BNodeSelection::of(&["a", "b"]), None, false, BClick::Release),
        Some(BNodeSelection::default()),
    );
    // Nothing selected already: no write.
    assert_eq!(clicked(&BNodeSelection::default(), None, false, BClick::Release), None);
    // And the press alone does not clear — only the release does, or a drag
    // from empty canvas could never box-select.
    assert_eq!(clicked(&BNodeSelection::of(&["a"]), None, false, BClick::Press), None);
}

#[test]
fn shift_adds_to_the_selection_and_takes_the_active_mark() {
    let before = BNodeSelection::of(&["a"]);
    let after = clicked(&before, Some("b"), true, BClick::Press).expect("a change");
    assert_eq!(after, BNodeSelection::of(&["b", "a"]), "b is first, so b is active");
    assert!(after.holds("a"), "and a is still selected");
}

#[test]
fn shift_clicking_a_selected_but_inactive_node_activates_it() {
    // The detail with Blender's reason written in: "Check active so clicking
    // on an inactive node activates it." `is_selected` is NODE_SELECT AND
    // NODE_ACTIVE, so an inactive member of the group is not deselected.
    let before = BNodeSelection::of(&["a", "b"]);
    let after = clicked(&before, Some("b"), true, BClick::Press).expect("a change");
    assert_eq!(after.active(), Some("b"), "activated, not removed");
    assert!(after.holds("a"));
    // The ACTIVE one, though, does come out.
    let after = clicked(&before, Some("a"), true, BClick::Press).expect("a change");
    assert!(!after.holds("a"), "the active node toggles off: {after:?}");
    assert!(after.holds("b"));
}

#[test]
fn a_box_replaces_the_selection_by_default_and_extends_with_shift() {
    let before = BNodeSelection::of(&["a"]);
    assert_eq!(boxed(&before, &["b", "c"], BBoxMode::Set), BNodeSelection::of(&["b", "c"]));
    let added = boxed(&before, &["b"], BBoxMode::Add);
    assert!(added.holds("a") && added.holds("b"), "{added:?}");
    let subbed = boxed(&BNodeSelection::of(&["a", "b"]), &["a"], BBoxMode::Sub);
    assert_eq!(subbed, BNodeSelection::of(&["b"]));
}

#[test]
fn the_box_modes_come_from_the_modifiers_the_keymap_binds() {
    assert_eq!(BBoxMode::of(false, false), BBoxMode::Set);
    assert_eq!(BBoxMode::of(true, false), BBoxMode::Add);
    assert_eq!(BBoxMode::of(false, true), BBoxMode::Sub);
}

#[test]
fn a_node_half_inside_the_box_is_selected() {
    // `BLI_rctf_isect`, not `BLI_rctf_inside_rctf`. Requiring containment is
    // the commoner choice elsewhere and would feel broken to anyone used to
    // Blender.
    let box_ = (100.0, 100.0, 200.0, 200.0);
    assert!(touches(box_, (150.0, 150.0, 300.0, 300.0)), "overlapping corner");
    assert!(touches(box_, (0.0, 0.0, 500.0, 500.0)), "box entirely inside the node");
    assert!(touches(box_, (120.0, 120.0, 180.0, 180.0)), "node entirely inside the box");
    assert!(touches(box_, (200.0, 200.0, 300.0, 300.0)), "touching at one corner counts");
    assert!(!touches(box_, (201.0, 100.0, 300.0, 200.0)), "clear to the right");
    assert!(!touches(box_, (100.0, 201.0, 200.0, 300.0)), "clear below");
}

#[test]
fn the_active_node_is_drawn_in_front_of_the_rest_of_the_selection() {
    // `compare_node_depth`: "One has a higher selection state (active >
    // selected > nothing)." Two levels was all phiber could express while the
    // selection was one name.
    let selection = BNodeSelection::of(&["b", "a"]);
    assert_eq!(rank(&node("b", 0), &selection), 2, "active");
    assert_eq!(rank(&node("a", 0), &selection), 1, "selected");
    assert_eq!(rank(&node("c", 0), &selection), 0, "neither");
}

#[test]
fn with_nothing_selected_in_the_editor_the_files_flags_decide() {
    // What the editor opens on, before the user has clicked anything.
    let none = BNodeSelection::default();
    assert_eq!(rank(&node("a", NODE_ACTIVE), &none), 2);
    assert_eq!(rank(&node("a", NODE_SELECT), &none), 1);
    assert_eq!(rank(&node("a", 0), &none), 0);
}

#[test]
fn a_selection_survives_a_round_trip_through_the_store() {
    for selection in [
        BNodeSelection::default(),
        BNodeSelection::of(&["Principled BSDF"]),
        BNodeSelection::of(&["Material Output", "Principled BSDF", "Image Texture"]),
    ] {
        assert_eq!(decode(&encode(&selection)), selection, "{selection:?}");
    }
    // The active one stays first, which is the whole point of the order.
    assert_eq!(decode("b;a").active(), Some("b"));
}
