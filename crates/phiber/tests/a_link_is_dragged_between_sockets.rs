//! Links can be made, moved and replaced by dragging between sockets.
//!
//! Blender's `NODE_OT_link` is a modal operator in three steps
//! (`editors/space_node/node_relationships.cc`): `node_link_init` on the press,
//! `node_link_find_socket` on each move, `add_dragged_links_to_tree` on the
//! release. The rules each step applies are unit-tested beside the code they
//! live in — `canvas/node/link/` for what is allowed and what a drop costs,
//! `canvas/node/pick/` for which socket the cursor is on.
//!
//! What this file is for is the three things only the whole store shows: that a
//! drag written as an action comes back out as a drag, that the edits it commits
//! survive a round trip and apply over the file's own links, and that two node
//! editors do not share either.
//!
//! The store is the subject here because phiber keeps link edits as an OVERLAY
//! rather than editing the tree: the `.blend` is re-read every frame and nothing
//! writes a node tree back yet, so a made link is `+from/sock>to/sock` under
//! `ctx.<host>.node.wired` and is applied over the file's list at draw time. The
//! file is never touched, which is why a graph edit cannot lose anything — and
//! why these edits do not outlive the session.

use phiber::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph, BlendNodeLink};
use phiber::phiphx::blend::shade::socket::BlendSocket;
use phiber::v2::host::BHostType;
use phiber::v2::state::action::apply_action;
use phiber::v2::state::{BKey as _, BState, NodeKey};
use phiber::v2::view::draw::canvas::node::link::{
    self, BLinkEdits, BLinkEnd, BLinking, commit, edits_of, linking_of,
};
use phiber::v2::view::prim::BAction;

fn input(identifier: &str) -> BlendSocket {
    BlendSocket { identifier: identifier.into(), name: identifier.into(), limit: 1, ..Default::default() }
}

fn output(identifier: &str) -> BlendSocket {
    BlendSocket { identifier: identifier.into(), name: identifier.into(), limit: 0xFFF, ..Default::default() }
}

fn node(name: &str, inputs: &[&str], outputs: &[&str]) -> BlendNode {
    BlendNode {
        name: name.into(),
        inputs: inputs.iter().map(|i| input(i)).collect(),
        outputs: outputs.iter().map(|o| output(o)).collect(),
        ..Default::default()
    }
}

/// Image Texture, Principled BSDF and Material Output, wired BSDF → Surface:
/// what a stock material's tree looks like.
fn graph() -> BlendNodeGraph {
    BlendNodeGraph {
        nodes: vec![
            node("Image Texture", &["Vector"], &["Color", "Alpha"]),
            node("Principled BSDF", &["Base Color", "Metallic"], &["BSDF"]),
            node("Material Output", &["Surface"], &[]),
        ],
        links: vec![BlendNodeLink {
            from: "Principled BSDF".into(),
            to: "Material Output".into(),
            from_socket: "BSDF".into(),
            to_socket: "Surface".into(),
        }],
        ..Default::default()
    }
}

/// A press on a socket, as the canvas dispatches it: `Adjust`, never `Set` — a
/// gesture writes continuously and a committed `Set` would dismiss every open
/// overlay on the first pixel of movement.
fn press(state: &mut BState, host: BHostType, drag: &BLinking) {
    apply_action(
        state,
        &BAction::Adjust(NodeKey::Linking(host).path(), link::live::encode(drag)),
    );
}

/// A release, with whatever socket was under it.
fn release(state: &mut BState, host: BHostType, graph: &BlendNodeGraph, target: Option<&BLinkEnd>) {
    let edits = edits_of(state, host);
    let wired = edits.apply(&graph.links);
    let Some(drag) = linking_of(state, host) else { panic!("nothing was being dragged") };
    if let Some(next) = commit(graph, &wired, &drag, target, &edits) {
        apply_action(state, &BAction::Adjust(NodeKey::Wired(host).path(), link::encode(&next)));
    }
    apply_action(state, &BAction::Reset(NodeKey::Linking(host).path()));
}

fn from_color() -> BLinking {
    BLinking {
        anchor: BLinkEnd::new("Image Texture", "Color"),
        from_output: true,
        at: (0.5, 0.5),
        moving: None,
    }
}

#[test]
fn a_drag_between_two_sockets_leaves_a_link_behind() {
    let mut state = BState::new();
    let g = graph();
    assert!(edits_of(&state, BHostType::Shader).apply(&g.links).len() == 1, "the file's one link");

    press(&mut state, BHostType::Shader, &from_color());
    assert!(linking_of(&state, BHostType::Shader).is_some(), "the press started a drag");

    release(
        &mut state,
        BHostType::Shader,
        &g,
        Some(&BLinkEnd::new("Principled BSDF", "Base Color")),
    );
    assert!(linking_of(&state, BHostType::Shader).is_none(), "the release ended it");

    let wired = edits_of(&state, BHostType::Shader).apply(&g.links);
    assert_eq!(wired.len(), 2, "the file's link and the new one: {wired:?}");
    assert!(
        wired.iter().any(|l| l.from_socket == "Color" && l.to_socket == "Base Color"),
        "{wired:?}",
    );
}

#[test]
fn a_release_over_nothing_leaves_the_graph_alone() {
    // Blender would open the link-drag search here (`link_drag_search.cc`);
    // phiber has no Add menu, so a slipped drag must cost nothing.
    let mut state = BState::new();
    let g = graph();
    press(&mut state, BHostType::Shader, &from_color());
    release(&mut state, BHostType::Shader, &g, None);
    assert_eq!(edits_of(&state, BHostType::Shader), BLinkEdits::default(), "no edits were made");
    assert_eq!(edits_of(&state, BHostType::Shader).apply(&g.links), g.links);
    assert!(linking_of(&state, BHostType::Shader).is_none(), "and the drag still ended");
}

#[test]
fn a_link_dropped_on_a_taken_input_replaces_the_one_there() {
    // `node_remove_existing_links_if_needed` with an input's limit of 1.
    let mut state = BState::new();
    let g = graph();
    press(&mut state, BHostType::Shader, &from_color());
    release(&mut state, BHostType::Shader, &g, Some(&BLinkEnd::new("Material Output", "Surface")));

    let wired = edits_of(&state, BHostType::Shader).apply(&g.links);
    let into_surface: Vec<&BlendNodeLink> =
        wired.iter().filter(|l| l.to_socket == "Surface").collect();
    assert_eq!(into_surface.len(), 1, "Surface takes one link: {into_surface:?}");
    assert_eq!(into_surface[0].from, "Image Texture", "the new one won");
}

#[test]
fn several_drags_in_a_row_all_stick() {
    let mut state = BState::new();
    let g = graph();
    for (socket, target) in
        [("Color", "Base Color"), ("Alpha", "Metallic")]
    {
        let drag = BLinking { anchor: BLinkEnd::new("Image Texture", socket), ..from_color() };
        press(&mut state, BHostType::Shader, &drag);
        release(&mut state, BHostType::Shader, &g, Some(&BLinkEnd::new("Principled BSDF", target)));
    }
    let wired = edits_of(&state, BHostType::Shader).apply(&g.links);
    assert_eq!(wired.len(), 3, "one from the file and two made: {wired:?}");
}

#[test]
fn two_node_editors_do_not_share_their_wiring() {
    // Blender gives every `SpaceNode` its own `edittree`. Linking in the
    // Shader editor must not wire the Compositor up.
    let mut state = BState::new();
    let g = graph();
    press(&mut state, BHostType::Shader, &from_color());
    release(
        &mut state,
        BHostType::Shader,
        &g,
        Some(&BLinkEnd::new("Principled BSDF", "Base Color")),
    );
    assert_eq!(edits_of(&state, BHostType::Shader).added.len(), 1);
    assert_eq!(
        edits_of(&state, BHostType::Compositor),
        BLinkEdits::default(),
        "the compositor was not touched",
    );
    assert!(linking_of(&state, BHostType::Compositor).is_none());
}

#[test]
fn a_drag_in_one_editor_does_not_appear_in_another() {
    let mut state = BState::new();
    press(&mut state, BHostType::GeoNode, &from_color());
    assert!(linking_of(&state, BHostType::GeoNode).is_some());
    for other in [BHostType::Shader, BHostType::Compositor, BHostType::Node] {
        assert!(linking_of(&state, other).is_none(), "{other:?} saw the geometry editor's drag");
    }
}

#[test]
fn the_edits_round_trip_through_the_store_untouched() {
    // The store holds strings, so the one thing that can quietly break is the
    // spelling — and a socket named "Base Color" has a space in it.
    let mut state = BState::new();
    let g = graph();
    press(&mut state, BHostType::Shader, &from_color());
    release(
        &mut state,
        BHostType::Shader,
        &g,
        Some(&BLinkEnd::new("Principled BSDF", "Base Color")),
    );
    let stored = state.get(&NodeKey::Wired(BHostType::Shader).path()).expect("something was written");
    assert!(stored.contains("Base Color"), "{stored}");
    assert_eq!(
        link::decode(stored).added[0],
        (BLinkEnd::new("Image Texture", "Color"), BLinkEnd::new("Principled BSDF", "Base Color")),
    );
}

#[test]
fn a_press_writes_a_drag_that_does_not_dismiss_overlays() {
    // `Adjust`, not `Set`. A `Set` runs `menu::close_overlays`, so the first
    // pixel of a drag would shut whatever the user had open.
    let mut state = BState::new();
    apply_action(&mut state, &BAction::Set(NodeKey::Selected(BHostType::Shader).path(), "x".into()));
    let before = state.get(&NodeKey::Selected(BHostType::Shader).path()).map(str::to_string);
    press(&mut state, BHostType::Shader, &from_color());
    assert_eq!(
        state.get(&NodeKey::Selected(BHostType::Shader).path()).map(str::to_string),
        before,
        "an adjustment writes its own key and disturbs nothing else",
    );
}
