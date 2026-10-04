//! Drag-and-commit tests — split out so `mod.rs` stays inside the 240-line
//! read budget (`level_structure`), the way `canvas/time/row/` is.

use super::*;
use crate::phiphx::blend::shade::graph::BlendNode;
use crate::phiphx::blend::shade::socket::BlendSocket;

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

fn link(from: &str, from_socket: &str, to: &str, to_socket: &str) -> BlendNodeLink {
    BlendNodeLink {
        from: from.into(),
        to: to.into(),
        from_socket: from_socket.into(),
        to_socket: to_socket.into(),
    }
}

fn graph() -> BlendNodeGraph {
    BlendNodeGraph {
        nodes: vec![
            node("Image Texture", &["Vector"], &["Color", "Alpha"]),
            node("Principled BSDF", &["Base Color", "Metallic"], &["BSDF"]),
            node("Material Output", &["Surface"], &[]),
        ],
        links: vec![link("Principled BSDF", "BSDF", "Material Output", "Surface")],
        ..Default::default()
    }
}

/// A drag off the Image Texture's Color output, cursor wherever.
fn from_color() -> BLinking {
    BLinking {
        anchor: BLinkEnd::new("Image Texture", "Color"),
        from_output: true,
        at: (0.5, 0.5),
        moving: None,
    }
}

#[test]
fn a_drag_from_an_output_looks_for_an_input() {
    assert_eq!(from_color().wants(), BSocketSide::Input);
    let from_input = BLinking { from_output: false, ..from_color() };
    assert_eq!(from_input.wants(), BSocketSide::Output);
}

#[test]
fn the_preview_runs_output_to_input_whichever_end_was_grabbed() {
    // A link always runs output → input, so which end the cursor carries
    // decides which end of the curve it is — not which end is anchored.
    let (anchor, cursor) = ((0.1, 0.1), (0.9, 0.9));
    assert_eq!(from_color().preview(cursor, anchor), (anchor, cursor), "anchor is the output");
    let from_input = BLinking { from_output: false, ..from_color() };
    assert_eq!(from_input.preview(cursor, anchor), (cursor, anchor), "cursor is the output");
}

#[test]
fn a_drop_on_a_free_input_adds_the_link() {
    let g = graph();
    let edits = commit(
        &g,
        &g.links,
        &from_color(),
        Some(&BLinkEnd::new("Principled BSDF", "Base Color")),
        &BLinkEdits::default(),
    )
    .expect("an ordinary new link");
    assert_eq!(edits.added.len(), 1);
    assert!(edits.removed.is_empty(), "nothing was in the way: {:?}", edits.removed);
}

#[test]
fn a_drop_on_a_taken_input_replaces_what_was_there() {
    // The limit of 1 doing its work through the whole commit, not just the rule.
    let g = graph();
    let edits = commit(
        &g,
        &g.links,
        &from_color(),
        Some(&BLinkEnd::new("Material Output", "Surface")),
        &BLinkEdits::default(),
    )
    .expect("allowed, and the existing link makes way");
    assert_eq!(edits.added.len(), 1);
    assert_eq!(edits.removed.len(), 1);
    assert_eq!(edits.removed[0].0.node, "Principled BSDF", "the BSDF's link came out");
}

#[test]
fn a_release_over_nothing_changes_nothing() {
    // The recorded divergence: Blender would open the link-drag search here.
    // Until that exists, a slipped drag must not cost a link.
    let g = graph();
    assert!(commit(&g, &g.links, &from_color(), None, &BLinkEdits::default()).is_none());
}

#[test]
fn a_drop_that_is_not_allowed_changes_nothing() {
    let g = graph();
    // Onto its own node.
    let own = BLinking { anchor: BLinkEnd::new("Principled BSDF", "BSDF"), ..from_color() };
    assert!(
        commit(&g, &g.links, &own, Some(&BLinkEnd::new("Principled BSDF", "Base Color")), &BLinkEdits::default())
            .is_none()
    );
    // Onto the socket it is already linked to.
    assert!(
        commit(&g, &g.links, &own, Some(&BLinkEnd::new("Material Output", "Surface")), &BLinkEdits::default())
            .is_none()
    );
}

#[test]
fn a_moved_link_is_only_removed_once_the_drop_succeeds() {
    // Picked up off Material Output's Surface, carried to Base Color.
    let g = graph();
    let moving = (
        BLinkEnd::new("Principled BSDF", "BSDF"),
        BLinkEnd::new("Material Output", "Surface"),
    );
    let drag = BLinking {
        anchor: BLinkEnd::new("Principled BSDF", "BSDF"),
        from_output: true,
        at: (0.5, 0.5),
        moving: Some(moving.clone()),
    };
    // Dropped on nothing: the link stays exactly where it was.
    assert!(commit(&g, &g.links, &drag, None, &BLinkEdits::default()).is_none());
    // Dropped somewhere real: now it moves.
    let edits = commit(
        &g,
        &g.links,
        &drag,
        Some(&BLinkEnd::new("Principled BSDF", "Metallic")),
        &BLinkEdits::default(),
    );
    // Its own node, so not allowed — pick a target on another node instead.
    assert!(edits.is_none(), "BSDF to its own Metallic is a self-link");
    let edits = commit(
        &g,
        &g.links,
        &drag,
        Some(&BLinkEnd::new("Image Texture", "Vector")),
        &BLinkEdits::default(),
    )
    .expect("a different node's input");
    assert!(edits.removed.contains(&moving), "the old link came out: {:?}", edits.removed);
    assert_eq!(edits.added.len(), 1);
}

#[test]
fn commits_build_on_the_edits_already_made() {
    // Two links made in a row: the second does not drop the first.
    let g = graph();
    let first = commit(
        &g,
        &g.links,
        &from_color(),
        Some(&BLinkEnd::new("Principled BSDF", "Base Color")),
        &BLinkEdits::default(),
    )
    .unwrap();
    let alpha = BLinking { anchor: BLinkEnd::new("Image Texture", "Alpha"), ..from_color() };
    let second =
        commit(&g, &g.links, &alpha, Some(&BLinkEnd::new("Principled BSDF", "Metallic")), &first)
            .unwrap();
    assert_eq!(second.added.len(), 2, "{:?}", second.added);
}

#[test]
fn replacing_a_link_made_this_session_takes_it_out_of_the_additions() {
    // A link that only exists as an edit cannot be "removed" from the file —
    // it has to come out of the added list, or applying the edits would draw
    // both.
    let g = graph();
    let made = commit(
        &g,
        &g.links,
        &from_color(),
        Some(&BLinkEnd::new("Principled BSDF", "Base Color")),
        &BLinkEdits::default(),
    )
    .unwrap();
    let wired = made.apply(&g.links);
    let alpha = BLinking { anchor: BLinkEnd::new("Image Texture", "Alpha"), ..from_color() };
    let next =
        commit(&g, &wired, &alpha, Some(&BLinkEnd::new("Principled BSDF", "Base Color")), &made)
            .unwrap();
    let applied = next.apply(&g.links);
    let into_base: Vec<&BlendNodeLink> =
        applied.iter().filter(|l| l.to_socket == "Base Color").collect();
    assert_eq!(into_base.len(), 1, "Base Color takes one link: {into_base:?}");
    assert_eq!(into_base[0].from_socket, "Alpha", "the newer one won");
}

#[test]
fn a_drag_survives_a_round_trip_through_the_store() {
    for drag in [
        from_color(),
        BLinking { from_output: false, at: (-0.25, 0.75), ..from_color() },
        BLinking {
            moving: Some((
                BLinkEnd::new("Principled BSDF", "BSDF"),
                BLinkEnd::new("Material Output", "Surface"),
            )),
            ..from_color()
        },
    ] {
        assert_eq!(decode(&encode(&drag)), Some(drag.clone()), "{drag:?}");
    }
}

#[test]
fn a_half_written_drag_is_no_drag() {
    for bad in ["", "a/Out", "a/Out|o", "a/Out|x|0.5,0.5", "a/Out|o|0.5", "aOut|o|0.5,0.5", "a/Out|o|x,y"] {
        assert!(decode(bad).is_none(), "{bad:?} should not have parsed");
    }
}
