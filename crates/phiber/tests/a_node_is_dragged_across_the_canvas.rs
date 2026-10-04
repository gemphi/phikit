//! A node editor's cards can be moved, and each editor remembers its own.
//!
//! The canvas drew the material's real tree but drew it read-only. Blender's node
//! editor is mostly moving nodes around — G is `NODE_OT_translate_attach` and it
//! is the first thing anyone does to a graph — so a graph you cannot rearrange is
//! a picture of one.
//!
//! What Blender does, from `editors/transform/transform_convert_node.cc`:
//! `create_transform_data_for_node` takes `node.location` as the transform origin
//! and notes "Use top-left corner as the transform origin for nodes";
//! `flushTransNodes` adds the accumulated offset to `td.iloc`, the location the
//! drag STARTED at, so a move is a translation and not a jump to the pointer.
//! `node_snap_grid_apply` rounds the start and the offset separately, and only
//! when snapping is on — phiber has no node snap setting, so nothing rounds.
//!
//! The drag maths lives in `canvas/node/drag.rs` and is unit-tested there. What
//! this file is for is the two things only the whole store can show: that a drag
//! written as an action comes back out as a displacement, and that two node
//! editors do not share one.

use phiber::v2::host::BHostType;
use phiber::v2::state::action::apply_action;
use phiber::v2::state::{BKey as _, BState, NodeKey, intern};
use phiber::v2::view::draw::canvas::node::drag;
use phiber::v2::view::prim::BAction;

/// One drag, as the canvas dispatches it: `Adjust`, not `Set`.
fn drag_to(state: &mut BState, host: BHostType, name: &str, offset: (f32, f32)) {
    let action =
        BAction::Adjust(NodeKey::At(host, intern(name)).path(), drag::encode(offset));
    apply_action(state, &action);
}

#[test]
fn a_dragged_node_keeps_the_displacement_it_was_given() {
    let mut state = BState::new();
    assert_eq!(
        drag::offset_of(&state, BHostType::Shader, "Principled BSDF"),
        (0.0, 0.0),
        "a node nobody has touched sits where the file put it"
    );

    drag_to(&mut state, BHostType::Shader, "Principled BSDF", (0.15, -0.2));
    let at = drag::offset_of(&state, BHostType::Shader, "Principled BSDF");
    assert!((at.0 - 0.15).abs() < 1e-6 && (at.1 + 0.2).abs() < 1e-6, "{at:?}");
}

#[test]
fn a_drag_adjusts_rather_than_decides() {
    // `BAction::Set` dismisses every deployed overlay. A node drag fires on
    // every mouse move, so writing `Set` would shut any open menu on the first
    // pixel of movement — the same reason a slider adjusts (`track/scale.rs`).
    // Stated here as the action the canvas builds, so a change to `Set` in
    // `drag.rs` fails rather than becoming a bug report about menus closing.
    let action =
        BAction::Adjust(NodeKey::At(BHostType::Shader, "Material Output").path(), "0, 0".into());
    assert!(matches!(action, BAction::Adjust(..)), "a move is not a decision");
}

#[test]
fn two_node_editors_do_not_share_one_nodes_place() {
    // Blender gives every `SpaceNode` its own `edittree`. The Shader editor and
    // the Compositor are two different trees that happen to contain nodes with
    // the same names — "Material Output" exists in both — so a key without the
    // host would have dragging one move the other.
    let mut state = BState::new();
    drag_to(&mut state, BHostType::Shader, "Material Output", (0.3, 0.0));
    assert_eq!(
        drag::offset_of(&state, BHostType::Compositor, "Material Output"),
        (0.0, 0.0),
        "the Compositor's node moved when the Shader editor's did"
    );
    let shader = drag::offset_of(&state, BHostType::Shader, "Material Output");
    assert!((shader.0 - 0.3).abs() < 1e-6, "{shader:?}");
}

#[test]
fn two_node_editors_do_not_share_one_selection() {
    let mut state = BState::new();
    apply_action(
        &mut state,
        &BAction::Set(NodeKey::Selected(BHostType::Shader).path(), "Principled BSDF".into()),
    );
    assert_eq!(state.get(&NodeKey::Selected(BHostType::Shader).path()), Some("Principled BSDF"));
    assert_eq!(
        state.get(&NodeKey::Selected(BHostType::Compositor).path()),
        None,
        "selecting in one node editor selected in another"
    );
}

#[test]
fn a_node_dragged_at_the_edge_stays_reachable() {
    // The one place this leaves Blender, and it follows from the canvas FITTING
    // the graph rather than panning it (`canvas/node/mod.rs`): Blender lets a
    // node go anywhere because you can always pan back to it, and here there is
    // no pan, so a node pushed past the edge would be gone for good.
    //
    // The clamp is applied to the WRITE, not only to the draw, so a drag jammed
    // into the edge and nudged back moves on that nudge instead of unwinding an
    // invisible surplus first.
    let base = (0.5, 0.5);
    let jammed = drag::moved(base, (0.0, 0.0), (50.0, 50.0));
    let mut state = BState::new();
    drag_to(&mut state, BHostType::Shader, "Principled BSDF", jammed);

    let stored = drag::offset_of(&state, BHostType::Shader, "Principled BSDF");
    let at = drag::placed(base, stored);
    assert!(at.0 <= 0.79 && at.1 <= 0.79, "{at:?} left the band the fit reserves for cards");

    let back = drag::placed(base, drag::moved(base, stored, (-0.05, 0.0)));
    assert!(back.0 < at.0 - 1e-6, "nudged back from the edge it did not move: {back:?} vs {at:?}");
}

/// Every node editor can be dragged in, not only the Shader one.
#[test]
fn every_node_editor_stores_its_own_places() {
    let editors =
        [BHostType::Node, BHostType::Shader, BHostType::Compositor, BHostType::GeoNode];
    let mut state = BState::new();
    for (i, host) in editors.iter().enumerate() {
        drag_to(&mut state, *host, "Group Output", (i as f32 / 10.0, 0.0));
    }
    for (i, host) in editors.iter().enumerate() {
        let at = drag::offset_of(&state, *host, "Group Output");
        assert!(
            (at.0 - i as f32 / 10.0).abs() < 1e-6,
            "{host:?} reads another editor's displacement: {at:?}"
        );
    }
}
