//! Socket tests — split out so each file stays inside the
//! 240-line read budget (`level_structure`), the way `canvas/time/row/` is.

use super::*;

fn sock(identifier: &str, kind: i32, flag: i32) -> BlendSocket {
    BlendSocket {
        identifier: identifier.into(),
        name: identifier.into(),
        kind,
        flag,
        ..Default::default()
    }
}

fn node(inputs: Vec<BlendSocket>, outputs: Vec<BlendSocket>) -> BlendNode {
    BlendNode { inputs, outputs, ..Default::default() }
}

#[test]
fn the_outputs_sit_above_the_inputs() {
    // The fact that makes a Blender graph read as a staircase: a result
    // leaves from the top of the card, not the bottom.
    let n = node(vec![sock("Base Color", 2, 0)], vec![sock("BSDF", 3, 0)]);
    let rows = rows(&n, &[]);
    let out = rows.iter().find(|r| r.side == BSocketSide::Output).unwrap();
    let inp = rows.iter().find(|r| r.side == BSocketSide::Input).unwrap();
    assert!(out.y < inp.y, "output at {} is not above input at {}", out.y, inp.y);
}

#[test]
fn a_socket_sits_at_the_middle_of_its_row() {
    // `locy - NODE_DYS` with `locy` at the row's top: half a row down.
    let n = node(Vec::new(), vec![sock("BSDF", 3, 0)]);
    assert_eq!(rows(&n, &[])[0].y, PAD + HALF, "the first output is half a row below the pad");
}

#[test]
fn consecutive_sockets_are_a_row_and_a_gap_apart() {
    let n = node(Vec::new(), vec![sock("A", 0, 0), sock("B", 0, 0)]);
    let rows = rows(&n, &[]);
    assert!(
        ((rows[1].y - rows[0].y) - (ROW + SPACING)).abs() < 1e-6,
        "{} apart, wanted {}",
        rows[1].y - rows[0].y,
        ROW + SPACING
    );
}

#[test]
fn an_unavailable_socket_takes_no_room() {
    use crate::phiphx::blend::shade::socket::SOCK_UNAVAIL;
    // A Math node in single-operand mode: the second operand is switched
    // off, and the card is a whole row shorter for it.
    let both = node(vec![sock("A", 0, 0), sock("B", 0, 0)], Vec::new());
    let one = node(vec![sock("A", 0, 0), sock("B", 0, SOCK_UNAVAIL)], Vec::new());
    assert_eq!(rows(&one, &[]).len(), 1, "the disabled operand has no row");
    assert!(
        body_height(&one, &[]) < body_height(&both, &[]),
        "{} is not shorter than {}",
        body_height(&one, &[]),
        body_height(&both, &[])
    );
}

#[test]
fn blenders_asymmetric_gap_is_ported_not_tidied() {
    use crate::phiphx::blend::shade::socket::SOCK_UNAVAIL;
    // Outputs space only before an AVAILABLE next socket; inputs space
    // before any next socket. Two lines apart in Blender, and the reason a
    // node whose last input is followed by a disabled one is a tenth of a
    // unit taller.
    //
    // The comparison has to be within one list, not between them: an
    // outputs-only node also collects `NODE_DY / 4` after its outputs, so
    // comparing one against an inputs-only node measures that pad as well
    // and says nothing about the gap.
    let one_in = node(vec![sock("A", 0, 0)], Vec::new());
    let trailing_in = node(vec![sock("A", 0, 0), sock("B", 0, SOCK_UNAVAIL)], Vec::new());
    let one_out = node(Vec::new(), vec![sock("A", 0, 0)]);
    let trailing_out = node(Vec::new(), vec![sock("A", 0, 0), sock("B", 0, SOCK_UNAVAIL)]);

    // One visible row either way — the trailing socket is never drawn.
    for n in [&one_in, &trailing_in, &one_out, &trailing_out] {
        assert_eq!(rows(n, &[]).len(), 1);
    }
    // An input followed by a disabled socket still gets its gap …
    assert!(
        (body_height(&trailing_in, &[]) - body_height(&one_in, &[]) - SPACING).abs() < 1e-6,
        "inputs: {} against {}",
        body_height(&trailing_in, &[]),
        body_height(&one_in, &[])
    );
    // … and an output followed by the same disabled socket does not.
    assert_eq!(
        body_height(&trailing_out, &[]),
        body_height(&one_out, &[]),
        "an output's gap waits for an available socket"
    );
}

#[test]
fn a_node_with_no_sockets_is_just_its_padding() {
    // No sockets, no inputs, no outputs: Blender adds the top space and
    // neither of the two conditional pads, so the body is a quarter unit.
    assert_eq!(body_height(&node(Vec::new(), Vec::new()), &[]), PAD);
    assert!(rows(&node(Vec::new(), Vec::new()), &[]).is_empty());
}

#[test]
fn a_link_finds_its_end_by_identifier() {
    let n = node(vec![sock("Base Color", 2, 0), sock("Metallic", 0, 0)], Vec::new());
    assert_eq!(row_of(&n, "Metallic", &[]).unwrap().kind, 0);
    assert!(row_of(&n, "Nonexistent", &[]).is_none(), "a missing socket is None, not the origin");
}

#[test]
fn an_input_reads_as_linked_only_when_its_identifier_is_in_the_set() {
    // The flag the painter reads to decide whether to draw the value: set from
    // the `linked` identifiers the layout is given, which is the wire set's
    // overlay-applied truth (`link::linked_inputs`), not any file flag.
    let n = node(vec![sock("Fac", 0, 0), sock("A", 0, 0)], vec![sock("Out", 0, 0)]);
    let rows = rows(&n, &["Fac"]);
    let flag = |id: &str| rows.iter().find(|r| r.identifier == id).unwrap().linked;
    assert!(flag("Fac"), "Fac is in the linked set");
    assert!(!flag("A"), "A is not, so it keeps its value");
    assert!(!flag("Out"), "an output is never a value socket, never linked");
}

#[test]
fn an_unconnected_vector_input_is_taller_and_collapses_when_linked() {
    use crate::phiphx::blend::shade::socket::BSocketDefault;
    // `std_node_socket_draw` draws an unconnected vector as a name row above a
    // field per component — a taller socket — and a connected one as a plain
    // row. So the node is taller exactly when the vector input is unlinked.
    let vector = BlendSocket {
        identifier: "Scale".into(),
        name: "Scale".into(),
        kind: 1,
        default: Some(BSocketDefault::Vector([1.0, 1.0, 1.0])),
        ..Default::default()
    };
    let n = node(vec![vector], Vec::new());
    let unlinked = body_height(&n, &[]);
    let linked = body_height(&n, &["Scale"]);
    // Three field rows' difference: the vector's `1 + 3` against a plain `1`.
    assert!((unlinked - linked - 3.0).abs() < 1e-6, "unlinked {unlinked} vs linked {linked}");
    // The row the painter draws carries the component count through the widget,
    // which is what makes the fields and the reserved height agree.
    let row = rows(&n, &[]).into_iter().find(|r| r.identifier == "Scale").unwrap();
    assert_eq!(value::rows_tall(row.side, row.linked, row.default.as_ref()), 4.0);
}

#[test]
fn the_metrics_are_blenders_own_fractions() {
    // Spelt out so a mistyped fraction fails here rather than as sockets
    // that drift out of their cards.
    assert_eq!(HALF, 0.5, "NODE_DYS is half a widget unit");
    assert_eq!(PAD, 0.25, "NODE_DYS / 2");
    assert_eq!(AFTER_OUTPUTS, 0.25, "NODE_DY / 4");
    assert_eq!(SPACING, 0.1, "NODE_ITEM_SPACING_Y is a tenth");
    assert_eq!(DOT, 0.25, "NODE_SOCKSIZE is a quarter");
}

// ── reroute nodes (`reroute_node_prepare_for_draw`) ─────────────────────────

/// The same node, typed as a reroute.
fn reroute(inputs: Vec<BlendSocket>, outputs: Vec<BlendSocket>) -> BlendNode {
    BlendNode { inputs, outputs, idname: "NodeReroute".into(), ..Default::default() }
}

#[test]
fn a_reroutes_two_sockets_sit_at_the_same_point() {
    // `node.input_socket(0).location = node.output_socket(0).location = loc`:
    // both ends are at the node's location, which is the one dot a reroute
    // draws. If the two rows drifted apart the reroute would sprout a second
    // dot and its links would land on empty canvas.
    let n = reroute(vec![sock("Input", 2, 0)], vec![sock("Output", 2, 0)]);
    let rows = rows(&n, &[]);
    assert_eq!(rows.len(), 2, "one input and one output");
    assert_eq!(rows[0].y, rows[1].y, "both at the same height");
    assert_eq!(rows[0].y, 0.0, "and that height is the card's top — the location");
    // One on each side, so a link into the reroute and a link out of it each
    // resolve to an end.
    assert!(rows.iter().any(|r| r.side == BSocketSide::Input));
    assert!(rows.iter().any(|r| r.side == BSocketSide::Output));
}

#[test]
fn a_reroute_has_no_body_and_paints_no_socket_labels() {
    // A reroute is one painted dot, not a card: no body to size and no socket
    // names drawn beside it (its only text is an optional node label, which is
    // not drawn yet). A reader that left `body_height` on the basis walk would
    // give the reroute a tall invisible card to click through.
    let n = reroute(vec![sock("Input", 2, 0)], vec![sock("Output", 2, 0)]);
    assert_eq!(body_height(&n, &[]), 0.0, "no body at all");
    assert!(rows(&n, &[]).iter().all(|r| r.name.is_empty()), "no socket labels");
    // The type is what decides this, not any flag — an ordinary node with the
    // same sockets is a full card.
    let ordinary = node(vec![sock("Input", 2, 0)], vec![sock("Output", 2, 0)]);
    assert!(body_height(&ordinary, &[]) > 0.0, "an ordinary node still has a body");
}
