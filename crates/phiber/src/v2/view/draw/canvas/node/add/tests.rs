//! Node-add overlay tests — that the added nodes merge over the file's own
//! with the sockets their type gives them, and leave the file untouched.

use super::*;
use crate::phiphx::blend::shade::graph::BlendNode;

fn file() -> Vec<BlendNode> {
    vec![
        BlendNode { name: "Principled BSDF".into(), idname: "ShaderNodeBsdfPrincipled".into(), ..Default::default() },
        BlendNode { name: "Material Output".into(), idname: "ShaderNodeOutputMaterial".into(), ..Default::default() },
    ]
}

#[test]
fn an_added_node_joins_the_file_with_the_sockets_of_its_type() {
    // The whole point of the catalog: the overlay stores only type/name/place,
    // and `apply` gives the node the sockets its type declares.
    let adds = BNodeAdditions {
        added: vec![BNodeAddition { idname: "ShaderNodeEmission".into(), name: "Emission".into(), at: (3.0, 5.0) }],
    };
    let merged = adds.apply(&file());
    assert_eq!(merged.len(), 3, "the two file nodes plus the one added");
    let added = merged.iter().find(|n| n.name == "Emission").unwrap();
    assert_eq!(added.idname, "ShaderNodeEmission");
    assert_eq!((added.x, added.y), (3.0, 5.0));
    let ins: Vec<&str> = added.inputs.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(ins, ["Color", "Strength"], "Emission's own inputs");
    assert_eq!(added.outputs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["Emission"]);
}

#[test]
fn the_file_is_never_touched_by_the_overlay() {
    let file = file();
    let adds = BNodeAdditions {
        added: vec![BNodeAddition { idname: "ShaderNodeValue".into(), name: "Value".into(), at: (0.0, 0.0) }],
    };
    let _ = adds.apply(&file);
    assert_eq!(file.len(), 2, "the file list is unchanged — the overlay clones it");
}

#[test]
fn an_addition_of_an_unknown_type_is_dropped_not_drawn_blank() {
    // A node phiber cannot give sockets to is one it should not pretend to have
    // added — a socketless typeless box is worse than nothing.
    let adds = BNodeAdditions {
        added: vec![
            BNodeAddition { idname: "ShaderNodeFromTheFuture".into(), name: "X".into(), at: (0.0, 0.0) },
            BNodeAddition { idname: "ShaderNodeValue".into(), name: "Value".into(), at: (0.0, 0.0) },
        ],
    };
    let merged = adds.apply(&file());
    assert_eq!(merged.len(), 3, "only the known type was merged");
    assert!(merged.iter().all(|n| n.name != "X"), "the unknown type did not join");
}

#[test]
fn an_empty_overlay_is_the_files_own_tree() {
    assert_eq!(BNodeAdditions::default().apply(&file()), file());
}

#[test]
fn a_free_name_is_used_as_is_and_a_taken_one_gets_a_suffix() {
    // `nodeUniqueName`: the base when free, else `.001`, `.002`, … to the first
    // gap. A duplicate name would make two nodes answer to one id.
    assert_eq!(unique_name("Emission", &[]), "Emission");
    assert_eq!(unique_name("Emission", &["Emission"]), "Emission.001");
    assert_eq!(unique_name("Emission", &["Emission", "Emission.001"]), "Emission.002");
    // A gap is filled rather than skipped past.
    assert_eq!(unique_name("Math", &["Math", "Math.002"]), "Math.001");
}

#[test]
fn place_appends_one_uniquely_named_node() {
    let ty = find("ShaderNodeValue").unwrap();
    let next = place(&BNodeAdditions::default(), &["Value"], ty, (4.0, 4.0));
    assert_eq!(next.added.len(), 1);
    assert_eq!(next.added[0].name, "Value.001", "Value was taken");
    assert_eq!(next.added[0].at, (4.0, 4.0));
    // And applying it over a file that has the original "Value" gives two nodes.
    let f = vec![BlendNode { name: "Value".into(), idname: "ShaderNodeValue".into(), ..Default::default() }];
    assert_eq!(next.apply(&f).len(), 2);
}

#[test]
fn an_instantiated_node_carries_no_file_identity() {
    // `nodeAddNode`: no flags, no parent, no saved address — an added node is
    // resolved by name like every node, and must not be mistaken for a framed
    // or selected file node.
    let ty = find("ShaderNodeValue").unwrap();
    let node = instantiate(ty, "Value", (2.0, 2.0));
    assert_eq!(node.flag, 0);
    assert_eq!(node.parent, None);
    assert_eq!(node.address, 0);
    assert!(!node.frame() && !node.reroute());
}
