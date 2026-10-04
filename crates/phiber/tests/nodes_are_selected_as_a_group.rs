//! A node editor's selection is a set with one active member, and a drag
//! carries all of it.
//!
//! phiber's selection was ONE node name, which was recorded twice as a known
//! limitation. Blender's is a set, and the difference is not cosmetic: every
//! operator from G to Delete acts on the set (`get_transformed_nodes`: *"Keep
//! only nodes that are selected or inside a frame that is selected"*), and
//! `NODE_ACTIVE` picks out the one member the next operator treats as its
//! subject.
//!
//! The rules are unit-tested beside the code in `canvas/node/select/`. What this
//! file is for is the three things only the whole store shows: that a click
//! written as an action comes back out as a selection, that the active mark
//! survives the round trip, and that two node editors do not share either.

use phiber::phiphx::blend::shade::graph::{BlendNode, NODE_ACTIVE, NODE_SELECT};
use phiber::v2::host::BHostType;
use phiber::v2::state::action::apply_action;
use phiber::v2::state::{BKey as _, BState, NodeKey};
use phiber::v2::view::draw::canvas::node::select::{
    self, BBoxMode, BClick, BNodeSelection, clicked, rank, selection_of,
};
use phiber::v2::view::prim::BAction;

fn node(name: &str, flag: i32) -> BlendNode {
    BlendNode { name: name.into(), flag, ..Default::default() }
}

/// A click, as the canvas dispatches it: `Adjust`, never `Set` — a press may be
/// the start of a drag, and a committed `Set` would dismiss every open overlay
/// on the first pixel of it.
fn click(state: &mut BState, host: BHostType, hit: Option<&str>, shift: bool, when: BClick) {
    let before = selection_of(state, host);
    if let Some(next) = clicked(&before, hit, shift, when) {
        apply_action(
            state,
            &BAction::Adjust(NodeKey::Selected(host).path(), select::encode(&next)),
        );
    }
}

#[test]
fn clicking_a_node_selects_it_and_clicking_another_replaces_it() {
    let mut state = BState::new();
    let shader = BHostType::Shader;
    assert!(selection_of(&state, shader).is_empty(), "nothing is selected to begin with");

    click(&mut state, shader, Some("Principled BSDF"), false, BClick::Press);
    assert_eq!(selection_of(&state, shader), BNodeSelection::of(&["Principled BSDF"]));

    click(&mut state, shader, Some("Material Output"), false, BClick::Press);
    assert_eq!(
        selection_of(&state, shader),
        BNodeSelection::of(&["Material Output"]),
        "a plain click replaces rather than adds",
    );
}

#[test]
fn shift_clicking_builds_a_group_and_moves_the_active_mark() {
    let mut state = BState::new();
    let shader = BHostType::Shader;
    click(&mut state, shader, Some("Image Texture"), false, BClick::Press);
    click(&mut state, shader, Some("Principled BSDF"), true, BClick::Press);
    click(&mut state, shader, Some("Material Output"), true, BClick::Press);

    let selection = selection_of(&state, shader);
    assert_eq!(selection.names.len(), 3, "{selection:?}");
    assert_eq!(selection.active(), Some("Material Output"), "the last one clicked");
    assert!(selection.holds("Image Texture") && selection.holds("Principled BSDF"));
}

#[test]
fn pressing_a_node_already_in_the_group_leaves_the_group_alone() {
    // `select_passthrough`. This is what makes a press-and-drag move the whole
    // selection instead of collapsing it to the node under the cursor first,
    // and it is only observable through the store: the press writes nothing.
    let mut state = BState::new();
    let shader = BHostType::Shader;
    click(&mut state, shader, Some("a"), false, BClick::Press);
    click(&mut state, shader, Some("b"), true, BClick::Press);
    let before = selection_of(&state, shader);

    click(&mut state, shader, Some("a"), false, BClick::Press);
    assert_eq!(selection_of(&state, shader), before, "the press changed nothing");

    // The release, with no drag in between, is what narrows it.
    click(&mut state, shader, Some("a"), false, BClick::Release);
    assert_eq!(selection_of(&state, shader), BNodeSelection::of(&["a"]));
}

#[test]
fn clicking_empty_canvas_clears_the_group() {
    let mut state = BState::new();
    let shader = BHostType::Shader;
    click(&mut state, shader, Some("a"), false, BClick::Press);
    click(&mut state, shader, Some("b"), true, BClick::Press);
    click(&mut state, shader, None, false, BClick::Release);
    assert!(selection_of(&state, shader).is_empty());
}

#[test]
fn a_box_select_replaces_the_group_and_shift_extends_it() {
    let mut state = BState::new();
    let shader = BHostType::Shader;
    let sweep = |state: &mut BState, inside: &[&str], mode| {
        let next = select::boxed(&selection_of(state, shader), inside, mode);
        apply_action(
            state,
            &BAction::Adjust(NodeKey::Selected(shader).path(), select::encode(&next)),
        );
    };
    click(&mut state, shader, Some("a"), false, BClick::Press);
    sweep(&mut state, &["b", "c"], BBoxMode::Set);
    assert_eq!(selection_of(&state, shader), BNodeSelection::of(&["b", "c"]), "SET pre-deselects");

    sweep(&mut state, &["d"], BBoxMode::Add);
    let selection = selection_of(&state, shader);
    assert_eq!(selection.names.len(), 3, "{selection:?}");

    sweep(&mut state, &["b"], BBoxMode::Sub);
    assert!(!selection_of(&state, shader).holds("b"), "SUB takes it out");
}

#[test]
fn the_active_node_is_drawn_in_front_of_the_rest_of_the_group() {
    // `compare_node_depth`: active > selected > nothing. Three levels, which
    // phiber could not express while a selection was one name.
    let mut state = BState::new();
    let shader = BHostType::Shader;
    click(&mut state, shader, Some("a"), false, BClick::Press);
    click(&mut state, shader, Some("b"), true, BClick::Press);
    let selection = selection_of(&state, shader);
    assert_eq!(rank(&node("b", 0), &selection), 2, "active");
    assert_eq!(rank(&node("a", 0), &selection), 1, "selected");
    assert_eq!(rank(&node("c", 0), &selection), 0, "neither");
}

#[test]
fn with_no_selection_in_the_editor_the_files_own_flags_decide() {
    // What the editor opens on, before anything is clicked — and the reason
    // `rank` takes the node and not just its name.
    let state = BState::new();
    let none = selection_of(&state, BHostType::Shader);
    assert!(none.is_empty());
    assert_eq!(rank(&node("a", NODE_ACTIVE), &none), 2);
    assert_eq!(rank(&node("a", NODE_SELECT), &none), 1);
    assert_eq!(rank(&node("a", 0), &none), 0);
}

#[test]
fn two_node_editors_do_not_share_a_selection() {
    // Blender gives every `SpaceNode` its own `edittree`.
    let mut state = BState::new();
    click(&mut state, BHostType::Shader, Some("Principled BSDF"), false, BClick::Press);
    click(&mut state, BHostType::Compositor, Some("Render Layers"), false, BClick::Press);
    assert_eq!(
        selection_of(&state, BHostType::Shader),
        BNodeSelection::of(&["Principled BSDF"]),
    );
    assert_eq!(
        selection_of(&state, BHostType::Compositor),
        BNodeSelection::of(&["Render Layers"]),
    );
    assert!(selection_of(&state, BHostType::GeoNode).is_empty());
}

#[test]
fn a_group_survives_the_round_trip_with_its_active_member_first() {
    // The store holds one string, and a node name carries whatever the file
    // gives it — spaces included.
    let mut state = BState::new();
    let shader = BHostType::Shader;
    click(&mut state, shader, Some("Image Texture"), false, BClick::Press);
    click(&mut state, shader, Some("Principled BSDF"), true, BClick::Press);
    let stored = state.get(&NodeKey::Selected(shader).path()).expect("a selection was written");
    assert!(stored.contains("Image Texture") && stored.contains("Principled BSDF"), "{stored}");
    assert_eq!(select::decode(stored).active(), Some("Principled BSDF"));
}
