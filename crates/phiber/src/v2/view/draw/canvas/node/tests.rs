//! Node canvas tests — split from `mod.rs` for the 240-line read budget
//! (`level_structure`), the way `socket/`, `view/` and `wire/` are.

use super::*;

fn node(name: &str, x: f32, y: f32) -> BlendNode {
    BlendNode { name: name.into(), x, y, ..Default::default() }
}

#[test]
fn the_opening_framing_puts_blenders_top_node_at_the_top() {
    // Blender's y runs UP; the canvas's runs down. A node Blender puts at
    // the top must come out at the top, or every graph opens upside down.
    // The fit used to do this flip itself; it happens once now, in
    // `BNodeView::to_fraction`, and this is the fact it has to keep.
    let nodes = vec![node("low", 0.0, 0.0), node("high", 100.0, 200.0)];
    let view = view::store::opening(&nodes, |_| 80.0, 240.0);
    let (low, high) = (view.to_fraction((0.0, 0.0)), view.to_fraction((100.0, 200.0)));
    assert!(low.0 < high.0, "the node further right in Blender is further right here");
    assert!(high.1 < low.1, "the node higher in Blender is higher here, not lower");
    for p in [low, high] {
        assert!((0.0..=1.0).contains(&p.0) && (0.0..=1.0).contains(&p.1), "{p:?}");
    }
}

#[test]
fn the_selected_node_is_drawn_last_so_it_is_on_top() {
    // `compare_node_depth`: selection raises a node. Without it a dragged
    // card slides behind whatever it is dragged over.
    let nodes = vec![node("a", 0.0, 0.0), node("b", 1.0, 1.0), node("c", 2.0, 2.0)];
    let one = |name: &str| select::BNodeSelection::of(&[name]);
    let order: Vec<&str> =
        draw_order(&nodes, &one("a")).iter().map(|n| n.name.as_str()).collect();
    assert_eq!(order, vec!["b", "c", "a"], "the selected node is painted over the others");
    // And the ones that are not selected keep the tree's own order.
    let order: Vec<&str> =
        draw_order(&nodes, &one("b")).iter().map(|n| n.name.as_str()).collect();
    assert_eq!(order, vec!["a", "c", "b"]);
}

#[test]
fn the_active_node_is_drawn_over_the_rest_of_its_group() {
    // Three levels, not two: the node the next operator acts on is the one
    // in front, which is only expressible now that a selection is a set.
    let nodes = vec![node("a", 0.0, 0.0), node("b", 1.0, 1.0), node("c", 2.0, 2.0)];
    let group = select::BNodeSelection::of(&["b", "a"]);
    let order: Vec<&str> =
        draw_order(&nodes, &group).iter().map(|n| n.name.as_str()).collect();
    assert_eq!(order, vec!["c", "a", "b"], "c behind, a selected, b active and in front");
}

#[test]
fn with_nothing_selected_the_file_decides_what_is_raised() {
    // `NODE_SELECT` out of the file is what the editor opens on, so it is
    // what raises a node until the user has clicked anything.
    let mut nodes = vec![node("a", 0.0, 0.0), node("b", 1.0, 1.0)];
    nodes[0].flag = crate::phiphx::blend::shade::graph::NODE_SELECT;
    let order: Vec<&str> = draw_order(&nodes, &select::BNodeSelection::default())
        .iter()
        .map(|n| n.name.as_str())
        .collect();
    assert_eq!(order, vec!["b", "a"], "the file's selected node is raised");
}

#[test]
fn a_muted_nodes_header_dims_toward_the_background_keeping_a_little_hue() {
    // `get_color_blend_4f(TH_BACK, color_id, 0.4)`: mostly background, a
    // little of the class hue left so the class is still legible under the
    // fade. `dim_toward(hue, bg, 0.4)` is 40% hue + 60% bg.
    let hue = (0.8_f32, 0.2, 0.2, 1.0);
    let bg = (0.1_f32, 0.1, 0.1, 1.0);
    let dim = dim_toward(hue, bg, MUTED_HEADER);
    // Each channel is 0.4*hue + 0.6*bg.
    assert!((dim.0 - (0.4 * 0.8 + 0.6 * 0.1)).abs() < 1e-6, "{dim:?}");
    assert!((dim.1 - (0.4 * 0.2 + 0.6 * 0.1)).abs() < 1e-6);
    // It moved toward the background on the loud channel, not past it.
    assert!(dim.0 < hue.0 && dim.0 > bg.0, "between hue and bg: {}", dim.0);
    // Alpha is the hue's own — dimming is a colour move, not a fade here.
    assert_eq!(dim.3, hue.3);
    // t = 1 is the hue untouched; t = 0 is the full background.
    assert_eq!(dim_toward(hue, bg, 1.0), hue, "all hue");
    let full = dim_toward(hue, bg, 0.0);
    assert!((full.0 - bg.0).abs() < 1e-5 && full.3 == hue.3, "all background: {full:?}");
}

#[test]
fn the_mute_flag_is_read_off_the_node() {
    use crate::phiphx::blend::shade::graph::NODE_MUTED;
    let mut n = node("n", 0.0, 0.0);
    assert!(!n.muted());
    n.flag = NODE_MUTED;
    assert!(n.muted(), "a node the file marks muted reads as muted");
}

#[test]
fn a_node_is_titled_by_its_label_and_identified_by_its_type() {
    let mut n = node("Principled BSDF", 0.0, 0.0);
    n.idname = "ShaderNodeBsdfPrincipled".into();
    assert_eq!(n.title(), "Principled BSDF");
    n.label = "Skin".into();
    assert_eq!(n.title(), "Skin", "a labelled node is called by its label");
}
