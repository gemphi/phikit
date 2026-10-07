use super::*;
use crate::phiphx::blend::shade::graph::BlendNode;
use crate::phiphx::blend::shade::socket::BlendSocket;

fn input(id: &str, default: BSocketDefault) -> BlendSocket {
    BlendSocket { identifier: id.into(), name: id.into(), default: Some(default), ..Default::default() }
}

fn node(name: &str, inputs: Vec<BlendSocket>) -> BlendNode {
    BlendNode { name: name.into(), inputs, ..Default::default() }
}

fn edits() -> BValueEdits {
    let mut e = BValueEdits::default();
    e.set("Principled BSDF", "Metallic", BSocketDefault::Float(0.5));
    e.set("Principled BSDF", "Base Color", BSocketDefault::Color([0.1, 0.2, 0.3, 1.0]));
    e.set("Noise Texture", "Scale", BSocketDefault::Vector([1.0, 2.0, 3.0]));
    e.set("Mix", "Clamp Result", BSocketDefault::Bool(true));
    e
}

#[test]
fn every_value_kind_survives_a_round_trip_through_the_store() {
    assert_eq!(decode(&encode(&edits())), edits());
}

#[test]
fn a_node_or_socket_name_with_a_space_survives() {
    // "Base Color" and "Principled BSDF" carry spaces; the separators are `/`
    // and `;`, which neither a name nor an identifier uses.
    let text = encode(&edits());
    assert_eq!(decode(&text).get("Principled BSDF", "Base Color"), Some(&BSocketDefault::Color([0.1, 0.2, 0.3, 1.0])));
}

#[test]
fn apply_overrides_only_the_named_input_socket() {
    let mut nodes =
        vec![node("Mix", vec![input("Fac", BSocketDefault::Float(0.5)), input("Clamp Result", BSocketDefault::Bool(false))])];
    let mut e = BValueEdits::default();
    e.set("Mix", "Clamp Result", BSocketDefault::Bool(true));
    e.apply(&mut nodes);
    let got = |id: &str| nodes[0].inputs.iter().find(|s| s.identifier == id).unwrap().default.clone();
    assert_eq!(got("Clamp Result"), Some(BSocketDefault::Bool(true)), "the edited socket takes the new value");
    assert_eq!(got("Fac"), Some(BSocketDefault::Float(0.5)), "its neighbour is untouched");
}

#[test]
fn setting_a_value_twice_replaces_rather_than_stacks() {
    let mut e = BValueEdits::default();
    e.set("Mix", "Fac", BSocketDefault::Float(0.2));
    e.set("Mix", "Fac", BSocketDefault::Float(0.8));
    assert_eq!(e.edits.len(), 1, "one entry per (node, socket)");
    assert_eq!(e.get("Mix", "Fac"), Some(&BSocketDefault::Float(0.8)), "the later value wins");
}

#[test]
fn an_edit_for_a_socket_that_is_not_there_is_dropped_not_panicked() {
    let mut nodes = vec![node("Mix", vec![input("Fac", BSocketDefault::Float(0.5))])];
    let mut e = BValueEdits::default();
    e.set("Mix", "Nonexistent", BSocketDefault::Float(1.0));
    e.set("Ghost Node", "Fac", BSocketDefault::Float(1.0));
    e.apply(&mut nodes); // must not panic
    assert_eq!(nodes[0].inputs[0].default, Some(BSocketDefault::Float(0.5)), "nothing matched, nothing changed");
}

fn vsize() -> super::super::plan::BCardSize {
    super::super::plan::BCardSize { header: 20.0, unit: 24.0, width: 140.0 }
}
fn vview() -> super::super::view::BNodeView {
    super::super::view::BNodeView { xmin: 0.0, ymin: 0.0, xmax: 1000.0, ymax: 1000.0 }
}
fn vinput(id: &str, default: BSocketDefault) -> crate::phiphx::blend::shade::socket::BlendSocket {
    crate::phiphx::blend::shade::socket::BlendSocket { identifier: id.into(), name: id.into(), default: Some(default), ..Default::default() }
}

#[test]
fn a_boolean_input_gets_one_toggle_target_on_the_rows_inner_right() {
    let n = node("Mix", vec![vinput("Clamp", BSocketDefault::Bool(true))]);
    let hits = hit::value_targets(&n, (0.1, 0.1), vsize(), &vview(), &[], None);
    assert_eq!(hits.len(), 1, "a boolean is one target");
    let h = &hits[0];
    assert_eq!(h.socket, "Clamp");
    assert_eq!(h.set, BSocketDefault::Bool(false), "a click toggles the current true to false");
    // On the card's own right half and inside its width fraction.
    let card_w = vsize().width / vview().width();
    assert!(h.at.0 > 0.1 + card_w * 0.5, "inner-right, not the left edge");
    assert!(h.at.0 + h.size.0 <= 0.1 + card_w + 1e-6, "within the card");
    assert!(h.at.1 > 0.1, "below the card's top edge");
}

#[test]
fn a_numeric_input_gets_a_step_down_on_the_left_and_a_step_up_on_the_right() {
    let n = node("Math", vec![vinput("Value", BSocketDefault::Float(0.5))]);
    let hits = hit::value_targets(&n, (0.1, 0.1), vsize(), &vview(), &[], None);
    assert_eq!(hits.len(), 2, "a number is a down target and an up target");
    let (down, up) = (&hits[0], &hits[1]);
    assert_eq!(down.set, BSocketDefault::Float(0.4), "left steps down a tenth");
    assert_eq!(up.set, BSocketDefault::Float(0.6), "right steps up a tenth");
    assert!(down.at.0 < up.at.0, "down sits left of up");
    // An int steps by one.
    let i = node("N", vec![vinput("I", BSocketDefault::Int(5))]);
    let ih = hit::value_targets(&i, (0.0, 0.0), vsize(), &vview(), &[], None);
    assert_eq!((ih[0].set.clone(), ih[1].set.clone()), (BSocketDefault::Int(4), BSocketDefault::Int(6)));
}

#[test]
fn a_vector_input_gets_a_step_pair_per_component_stacked_down_the_column() {
    let n = node("Map", vec![vinput("Location", BSocketDefault::Vector([1.0, 2.0, 3.0]))]);
    let hits = hit::value_targets(&n, (0.0, 0.0), vsize(), &vview(), &[], None);
    assert_eq!(hits.len(), 6, "three components, a down and an up each");
    // Component 1's up steps only the middle number, leaving x and z.
    let mid_up = hits.iter().find(|h| h.set == BSocketDefault::Vector([1.0, 2.1, 3.0]));
    assert!(mid_up.is_some(), "the middle component steps on its own");
    // And the first component's row sits above the third's.
    let first = hits.iter().find(|h| h.set == BSocketDefault::Vector([1.1, 2.0, 3.0])).unwrap();
    let third = hits.iter().find(|h| h.set == BSocketDefault::Vector([1.0, 2.0, 3.1])).unwrap();
    assert!(first.at.1 < third.at.1, "component 0 is drawn above component 2");
}

#[test]
fn a_colour_input_opens_a_channel_picker_only_when_its_swatch_is_picked() {
    let orig = [0.1_f32, 0.2, 0.3, 1.0];
    let n = node("Mix", vec![vinput("A", BSocketDefault::Color(orig))]);
    // Closed: no inline channel steppers, but the swatch is one click target.
    assert!(hit::value_targets(&n, (0.0, 0.0), vsize(), &vview(), &[], None).is_empty());
    assert_eq!(hit::swatch_hits(&n, (0.0, 0.0), vsize(), &vview(), &[]).len(), 1, "the swatch opens the picker");
    // Open: R, G, B, A each step down and up — eight targets, each a colour that
    // moves exactly one channel by a tenth.
    let open = hit::value_targets(&n, (0.0, 0.0), vsize(), &vview(), &[], Some("Mix/A"));
    assert_eq!(open.len(), 8, "four channels, down and up each");
    let moved = |h: &hit::BValueHit| -> usize {
        let BSocketDefault::Color(c) = h.set else { panic!("a colour channel writes a colour") };
        let ch: Vec<usize> = (0..4).filter(|&i| (c[i] - orig[i]).abs() > 1e-4).collect();
        assert_eq!(ch.len(), 1, "exactly one channel changes");
        assert!(((c[ch[0]] - orig[ch[0]]).abs() - 0.1).abs() < 1e-4, "by a tenth");
        ch[0]
    };
    let channels: std::collections::BTreeSet<usize> = open.iter().map(moved).collect();
    assert_eq!(channels.len(), 4, "every channel R G B A is steppable");
}

#[test]
fn a_linked_input_has_no_value_target_to_click() {
    let n = node("Mix", vec![vinput("Clamp", BSocketDefault::Bool(true)), vinput("Val", BSocketDefault::Float(0.5))]);
    // Wires arrive at both: each shows the wire, not its value, so nothing to click.
    assert!(hit::value_targets(&n, (0.1, 0.1), vsize(), &vview(), &["Clamp", "Val"], None).is_empty());
}

#[test]
fn a_malformed_stored_entry_is_dropped() {
    // A value token with an unknown tag, and one with a short vector, are both
    // dropped; the well-formed entry beside them still reads.
    let text = "A/Fac/f0.5;B/Bad/q9;C/V/v1,2";
    let out = decode(text);
    assert_eq!(out.edits.len(), 1);
    assert_eq!(out.get("A", "Fac"), Some(&BSocketDefault::Float(0.5)));
}
