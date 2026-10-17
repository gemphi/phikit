use super::*;
use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeLink};

fn node(name: &str) -> BlendNode {
    BlendNode { name: name.to_string(), width: 140.0, ..Default::default() }
}

fn link(from: &str, to: &str) -> BlendNodeLink {
    BlendNodeLink {
        from: from.to_string(),
        from_socket: "Out".to_string(),
        to: to.to_string(),
        to_socket: "In".to_string(),
    }
}

#[test]
fn apply_drops_the_removed_nodes_and_keeps_the_rest() {
    let mut nodes = vec![node("A"), node("B"), node("C")];
    BNodeRemovals::of(&["B"]).apply(&mut nodes);
    assert_eq!(nodes.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(), ["A", "C"]);
}

#[test]
fn prune_drops_links_with_either_end_on_a_removed_node() {
    let mut links = vec![link("A", "B"), link("B", "C"), link("A", "C")];
    // B is gone: its two links go, the A→C link stays.
    BNodeRemovals::of(&["B"]).prune(&mut links);
    assert_eq!(links.len(), 1);
    assert_eq!((links[0].from.as_str(), links[0].to.as_str()), ("A", "C"));
}

#[test]
fn encode_round_trips_and_add_does_not_duplicate() {
    let mut r = BNodeRemovals::default();
    r.add("A");
    r.add("B");
    r.add("A"); // already there
    assert_eq!(r.names, ["A", "B"]);
    assert_eq!(decode(&encode(&r)), r);
}

#[test]
fn an_empty_value_is_an_empty_set() {
    assert!(decode("").is_empty());
}
