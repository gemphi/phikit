//! Paint tests — split out so each file stays inside the
//! 240-line read budget (`level_structure`), the way `canvas/time/row/` is.

use super::*;

#[test]
fn a_steep_link_gets_the_full_four_tenths_of_handle() {
    // curving 4 → the bracket is 3.5, so any slope at or above 1/3.5
    // clamps to 1 and the handle is the full `curving * 0.1 * dist_x`.
    let (c1, c2) = handles((0.0, 0.0), (100.0, 100.0), CURVING);
    assert!((c1.0 - 40.0).abs() < 1e-4, "{c1:?}");
    assert!((c2.0 - 60.0).abs() < 1e-4, "{c2:?}");
    // A handle keeps its own end's height — that is what makes a link
    // leave a socket horizontally.
    assert_eq!(c1.1, 0.0);
    assert_eq!(c2.1, 100.0);
}

#[test]
fn a_nearly_horizontal_link_is_nearly_straight() {
    // Blender: "Reduce the handle offset when the link endpoints are close
    // to horizontal." Without the clamp this would be a long flat S.
    let (c1, _) = handles((0.0, 0.0), (100.0, 1.0), CURVING);
    let full = handles((0.0, 0.0), (100.0, 100.0), CURVING).0;
    assert!(c1.0 < full.0 * 0.1, "handle {} is not reduced from {}", c1.0, full.0);
}

#[test]
fn a_vertical_link_does_not_divide_by_its_own_width() {
    // `safe_divide` gives 0, not infinity, so the handles collapse and the
    // link is a straight vertical line rather than a NaN.
    let (c1, c2) = handles((50.0, 0.0), (50.0, 90.0), CURVING);
    assert_eq!(c1, (50.0, 0.0));
    assert_eq!(c2, (50.0, 90.0));
}

#[test]
fn curving_zero_degenerates_to_a_line() {
    // The handles at a third and two thirds: a cubic through them IS the
    // straight line between its ends.
    let (c1, c2) = handles((0.0, 0.0), (90.0, 30.0), 0.0);
    assert!((c1.0 - 30.0).abs() < 1e-4 && (c1.1 - 10.0).abs() < 1e-4, "{c1:?}");
    assert!((c2.0 - 60.0).abs() < 1e-4 && (c2.1 - 20.0).abs() < 1e-4, "{c2:?}");
}

#[test]
fn an_input_sits_on_the_left_edge_and_an_output_on_the_right() {
    // Centred ON the edge, not inset from it — so a socket straddles the
    // card's border, which is how Blender draws it.
    let row = |side| BSocketRow {
        identifier: "x".into(),
        name: "x".into(),
        kind: 0,
        side,
        y: 1.0,
        default: None,
        linked: false,
    };
    let card = BNodeCard {
        name: "n".into(),
        x: 100.0,
        y: 50.0,
        width: 200.0,
        header: 20.0,
        unit: 20.0,
        height: 80.0,
        sockets: Vec::new(),
    };
    assert_eq!(card.socket_at(&row(BSocketSide::Input)).0, 100.0);
    assert_eq!(card.socket_at(&row(BSocketSide::Output)).0, 300.0);
    // The row's y is measured from the bottom of the header, in units.
    assert_eq!(card.socket_at(&row(BSocketSide::Input)).1, 50.0 + 20.0 + 20.0);
}

#[test]
fn the_inner_line_is_blenders_fraction_of_the_outline() {
    // `line_thickness *= 0.65f` in the main pass against the outline's full
    // `LINK_WIDTH`. Spelt out because a noodle drawn at one width is a
    // different-looking editor, and nothing else would fail.
    assert_eq!(LINK_WIDTH, 2.5, "LINK_WIDTH (drawnode.cc:1959)");
    assert_eq!(INNER_SCALE, 0.65, "gpu_shader_2D_nodelink.bsl.hh:120");
    assert!(
        LINK_WIDTH * INNER_SCALE < LINK_WIDTH,
        "the main line has to be narrower than the outline it sits on"
    );
}

#[test]
fn an_input_reads_as_linked_only_when_a_wire_arrives_at_it() {
    use crate::phiphx::blend::shade::graph::BlendNodeLink;
    use crate::phiphx::blend::shade::socket::BSocketDefault;
    let input = |id: &str| BSocketRow {
        identifier: id.into(),
        name: id.into(),
        kind: 0,
        side: BSocketSide::Input,
        y: 1.0,
        default: Some(BSocketDefault::Float(0.5)),
        linked: false,
    };
    let output = BSocketRow { side: BSocketSide::Output, ..input("Out") };
    let mut card = BNodeCard {
        name: "Mix".into(),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        header: 10.0,
        unit: 10.0,
        height: 40.0,
        sockets: vec![input("Fac"), input("A"), output],
    };
    // One wire lands on "Fac"; nothing lands on "A".
    let wired = vec![BlendNodeLink {
        from: "Value".into(),
        from_socket: "Value".into(),
        to: "Mix".into(),
        to_socket: "Fac".into(),
    }];
    card.mark_linked(&wired);
    let linked = |id: &str| card.sockets.iter().find(|r| r.identifier == id).unwrap().linked;
    assert!(linked("Fac"), "a wire arrives at Fac");
    assert!(!linked("A"), "nothing arrives at A, so it keeps its value");
    // The output is never marked — a value is only ever an input's.
    assert!(!card.sockets.iter().find(|r| r.side == BSocketSide::Output).unwrap().linked);
}
