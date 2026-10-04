//! Link-rule tests — split out so `mod.rs` stays inside the 240-line read
//! budget (`level_structure`), the way `canvas/time/row/` is.

use super::*;
use crate::phiphx::blend::shade::socket::SOCK_MULTI_INPUT;

/// An input socket as Blender creates one: `limit = 1`.
fn input(identifier: &str) -> BlendSocket {
    BlendSocket { identifier: identifier.into(), name: identifier.into(), limit: 1, ..Default::default() }
}

/// An output socket as Blender creates one: `limit = 0xFFF`.
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

/// Image Texture → Principled BSDF → Material Output, the usual chain.
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

#[test]
fn a_node_cannot_link_to_itself() {
    // "Skip if socket is on the same node as the fromsock."
    let g = graph();
    assert!(!allowed(
        &g,
        &g.links,
        &BLinkEnd::new("Principled BSDF", "BSDF"),
        &BLinkEnd::new("Principled BSDF", "Base Color"),
    ));
}

#[test]
fn the_same_pair_of_sockets_cannot_be_linked_twice() {
    // "Skip if tsock is already linked with this output."
    let g = graph();
    let (from, to) =
        (BLinkEnd::new("Principled BSDF", "BSDF"), BLinkEnd::new("Material Output", "Surface"));
    assert!(!allowed(&g, &g.links, &from, &to), "the link in the file already joins these two");
    // The same output into a DIFFERENT input is fine — that is fan-out, and an
    // output's limit of 4095 is what permits it.
    assert!(allowed(&g, &g.links, &from, &BLinkEnd::new("Image Texture", "Vector")));
}

#[test]
fn a_link_to_a_socket_that_is_not_drawn_is_not_allowed() {
    // A link nobody can see is a link nobody can break, so it is not offered.
    use crate::phiphx::blend::shade::socket::SOCK_UNAVAIL;
    let mut g = graph();
    g.nodes[1].inputs[0].flag = SOCK_UNAVAIL;
    assert!(!allowed(
        &g,
        &g.links,
        &BLinkEnd::new("Image Texture", "Color"),
        &BLinkEnd::new("Principled BSDF", "Base Color"),
    ));
    // And a socket that is not there at all.
    assert!(!allowed(
        &g,
        &g.links,
        &BLinkEnd::new("Image Texture", "Color"),
        &BLinkEnd::new("Principled BSDF", "Nonexistent"),
    ));
}

#[test]
fn an_ordinary_new_link_is_allowed() {
    let g = graph();
    assert!(allowed(
        &g,
        &g.links,
        &BLinkEnd::new("Image Texture", "Color"),
        &BLinkEnd::new("Principled BSDF", "Base Color"),
    ));
}

#[test]
fn a_float_into_a_colour_is_allowed_because_blender_allows_it() {
    // There is no type check in `node_link_find_socket`. A float into a colour
    // is legal and is converted on evaluation, so refusing it here would be a
    // rule phiber invented.
    let mut g = graph();
    g.nodes[0].outputs[1].kind = 0; // Alpha, SOCK_FLOAT
    g.nodes[1].inputs[0].kind = 2; // Base Color, SOCK_RGBA
    assert!(allowed(
        &g,
        &g.links,
        &BLinkEnd::new("Image Texture", "Alpha"),
        &BLinkEnd::new("Principled BSDF", "Base Color"),
    ));
}

#[test]
fn a_drop_on_a_connected_input_replaces_what_is_there() {
    // The limit of 1 doing its work: `!(count < limit)` with count 1 and limit
    // 1 is true, so the existing link comes out to make room.
    let links = vec![link("Principled BSDF", "BSDF", "Material Output", "Surface")];
    let going = links_to_remove(
        &links,
        &input("Surface"),
        &BLinkEnd::new("Image Texture", "Color"),
        &BLinkEnd::new("Material Output", "Surface"),
    );
    assert_eq!(going.len(), 1, "the link already on Surface has to go");
    assert_eq!(going[0].from, "Principled BSDF");
}

#[test]
fn a_drop_on_an_empty_input_removes_nothing() {
    let links = vec![link("Principled BSDF", "BSDF", "Material Output", "Surface")];
    let going = links_to_remove(
        &links,
        &input("Base Color"),
        &BLinkEnd::new("Image Texture", "Color"),
        &BLinkEnd::new("Principled BSDF", "Base Color"),
    );
    assert!(going.is_empty(), "nothing is in the way");
}

#[test]
fn a_duplicate_comes_out_in_the_first_pass_and_makes_its_own_room() {
    // Blender removes duplicates FIRST, each decrementing the count — so
    // re-dropping the same link replaces itself and takes nothing else with it.
    let links = vec![link("Principled BSDF", "BSDF", "Material Output", "Surface")];
    let going = links_to_remove(
        &links,
        &input("Surface"),
        &BLinkEnd::new("Principled BSDF", "BSDF"),
        &BLinkEnd::new("Material Output", "Surface"),
    );
    assert_eq!(going.len(), 1);
    // One removal, not two: the duplicate pass dropped the count to 0, so the
    // limit pass found room and took nothing.
    assert_eq!(going[0].from, "Principled BSDF");
}

#[test]
fn a_multi_input_socket_keeps_everything_already_on_it() {
    // Limit 4095, so a drop stacks rather than replaces — a Join Geometry
    // input with three things plugged in keeps all three.
    let joined = BlendSocket { identifier: "Geometry".into(), limit: 1, flag: SOCK_MULTI_INPUT, ..Default::default() };
    let links = vec![
        link("Cube", "Mesh", "Join", "Geometry"),
        link("Sphere", "Mesh", "Join", "Geometry"),
    ];
    let going = links_to_remove(
        &links,
        &joined,
        &BLinkEnd::new("Cone", "Mesh"),
        &BLinkEnd::new("Join", "Geometry"),
    );
    assert!(going.is_empty(), "a multi-input stacks: {going:?}");
}

#[test]
fn grabbing_an_output_starts_a_new_link() {
    let g = graph();
    let bsdf = &g.nodes[1];
    assert_eq!(
        grab(&g.links, bsdf, &bsdf.outputs[0], true),
        BLinkGrab::Fresh(BLinkEnd::new("Principled BSDF", "BSDF")),
    );
}

#[test]
fn grabbing_an_empty_input_starts_a_new_link() {
    let g = graph();
    let bsdf = &g.nodes[1];
    assert_eq!(
        grab(&g.links, bsdf, &bsdf.inputs[0], false),
        BLinkGrab::Fresh(BLinkEnd::new("Principled BSDF", "Base Color")),
    );
}

#[test]
fn grabbing_a_connected_input_picks_up_the_link_rather_than_making_a_second() {
    // `node_link_init`'s input half: the existing link is detached and dragged
    // by its OUTPUT end, so pulling off a connected input rewires instead of
    // stacking. An input's limit is 1, so a second could not exist anyway.
    let g = graph();
    let out_node = &g.nodes[2];
    let grabbed = grab(&g.links, out_node, &out_node.inputs[0], false);
    assert_eq!(
        grabbed,
        BLinkGrab::Moved {
            anchor: BLinkEnd::new("Principled BSDF", "BSDF"),
            was: (
                BLinkEnd::new("Principled BSDF", "BSDF"),
                BLinkEnd::new("Material Output", "Surface"),
            ),
        },
        "the anchor is the far end, which stays put while the cursor carries the near one",
    );
}

#[test]
fn a_connected_multi_input_is_never_picked_up() {
    // `link_to_pick != nullptr && !nldrag->start_socket->is_multi_input()` —
    // there would be no way to say WHICH of its links you meant.
    let joined = BlendSocket { identifier: "Geometry".into(), limit: 1, flag: SOCK_MULTI_INPUT, ..Default::default() };
    let join = BlendNode { name: "Join".into(), inputs: vec![joined.clone()], ..Default::default() };
    let links = vec![link("Cube", "Mesh", "Join", "Geometry")];
    assert_eq!(grab(&links, &join, &joined, false), BLinkGrab::Fresh(BLinkEnd::new("Join", "Geometry")));
}
