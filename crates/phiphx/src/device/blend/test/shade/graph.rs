use super::*;

#[test]
fn a_node_shows_its_label_when_it_has_one_and_its_name_otherwise() {
    // `nodeLabel`: the user's label wins; without one the node's own name is
    // what Blender has already made from the type (`Principled BSDF.001`).
    let mut node = BlendNode { name: "Principled BSDF".into(), ..Default::default() };
    assert_eq!(node.title(), "Principled BSDF");
    node.label = "Skin".into();
    assert_eq!(node.title(), "Skin", "a labelled node is called by its label");
}

#[test]
fn selection_reads_the_flag_bit_blender_sets() {
    let mut node = BlendNode::default();
    assert!(!node.selected());
    node.flag = NODE_SELECT;
    assert!(node.selected());
    // Other bits in `flag` are not selection.
    node.flag = 1 << 4;
    assert!(!node.selected(), "only NODE_SELECT means selected");
}

#[test]
fn a_reroute_is_known_by_its_type_name() {
    // `node.is_reroute()` is a TYPE test in Blender, not a flag — the type's
    // idname is `"NodeReroute"`. Every other node is drawn as a card; a
    // reroute is drawn as a bare dot, so getting this wrong draws the dot as
    // a card.
    let mut node = BlendNode::default();
    assert!(!node.reroute(), "an untyped node is not a reroute");
    node.idname = "NodeReroute".into();
    assert!(node.reroute());
    node.idname = "ShaderNodeBsdfPrincipled".into();
    assert!(!node.reroute(), "a real shader node is not a reroute");
}

#[test]
fn a_frame_is_known_by_its_type_name_and_carries_its_parent() {
    // Like `reroute`, `is_frame()` is a TYPE test — the idname is
    // `"NodeFrame"`. A node not inside any frame carries no parent.
    let mut node = BlendNode::default();
    assert!(!node.frame(), "an untyped node is not a frame");
    assert_eq!(node.parent, None, "and it is parented into nothing");
    node.idname = "NodeFrame".into();
    assert!(node.frame());
    node.idname = "ShaderNodeMath".into();
    assert!(!node.frame());
    // A child names its frame by the frame's own address.
    node.parent = Some(0xBEEF);
    assert_eq!(node.parent, Some(0xBEEF));
}

#[test]
fn an_empty_graph_says_so() {
    assert!(BlendNodeGraph::default().is_empty());
    let one = BlendNodeGraph {
        nodes: vec![BlendNode { name: "A".into(), ..Default::default() }],
        links: Vec::new(),
    };
    assert!(!one.is_empty());
}
