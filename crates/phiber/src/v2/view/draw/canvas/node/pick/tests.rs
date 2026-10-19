//! Hit-test tests — split out so `mod.rs` stays inside the 240-line read
//! budget (`level_structure`), the way `canvas/time/row/` is.

use super::*;

const UNIT: f32 = 20.0;

fn row(side: BSocketSide, y: f32) -> BSocketRow {
    BSocketRow { identifier: "s".into(), name: "s".into(), kind: 0, shape: 0, side, y, default: None, linked: false }
}

/// A card at `(x, y)`, 200 wide, with one input and one output.
fn card(x: f32, y: f32) -> BNodeCard {
    BNodeCard {
        name: format!("card at {x},{y}"),
        x,
        y,
        width: 200.0,
        header: UNIT,
        unit: UNIT,
        height: UNIT * 4.0,
        sockets: vec![row(BSocketSide::Output, 0.75), row(BSocketSide::Input, 2.0)],
    }
}

#[test]
fn the_reach_is_the_sockets_radius_plus_a_unit_of_slack() {
    // `NODE_SOCKSIZE + clamp(20 / scale, 5, 30)`, in units: 0.25 + 1.
    assert_eq!(max_distance(UNIT, 1.0), (0.25 + 1.0) * UNIT);
    // Zoomed out, the slack grows in view units so it stays constant on
    // screen — until the clamp stops it at one and a half units.
    assert_eq!(max_distance(UNIT, 0.1), 1.75 * UNIT, "clamped at SLACK_MAX");
    // Zoomed in, it shrinks — to a quarter unit and no further.
    assert_eq!(max_distance(UNIT, 100.0), 0.5 * UNIT, "clamped at SLACK_MIN");
}

#[test]
fn a_cursor_on_a_socket_finds_it() {
    let cards = vec![card(100.0, 100.0)];
    let (_, found) = socket_at(&cards, cards[0].socket_at(&cards[0].sockets[1]), None, 1.0)
        .expect("the input is right under the cursor");
    assert_eq!(found.side, BSocketSide::Input);
}

#[test]
fn a_cursor_well_clear_of_every_socket_finds_nothing() {
    let cards = vec![card(100.0, 100.0)];
    assert!(socket_at(&cards, (900.0, 900.0), None, 1.0).is_none());
}

#[test]
fn the_nearer_of_two_sockets_wins() {
    // Blender keeps a `best_distance` rather than returning the first hit.
    // The two have to be on the SAME side to test that: sockets on opposite
    // edges are 200px apart horizontally, so the comparison would be decided
    // by x and say nothing about nearest-wins.
    let mut card = card(100.0, 100.0);
    card.sockets = vec![
        BSocketRow { identifier: "upper".into(), y: 1.0, ..row(BSocketSide::Input, 1.0) },
        BSocketRow { identifier: "lower".into(), y: 2.0, ..row(BSocketSide::Input, 2.0) },
    ];
    let cards = vec![card];
    let upper = cards[0].socket_at(&cards[0].sockets[0]).1;
    let lower = cards[0].socket_at(&cards[0].sockets[1]).1;
    let at = |t: f32| (cards[0].x, upper + (lower - upper) * t);

    let (_, found) = socket_at(&cards, at(0.6), None, 1.0).expect("one of the two");
    assert_eq!(found.identifier, "lower", "nearer to the lower one");
    let (_, found) = socket_at(&cards, at(0.4), None, 1.0).expect("one of the two");
    assert_eq!(found.identifier, "upper");
}

#[test]
fn asking_for_one_side_ignores_the_other() {
    // `node_link_find_socket` searches only for the opposite kind, which is how
    // Blender makes an output-to-output link unexpressible.
    let cards = vec![card(100.0, 100.0)];
    let on_input = cards[0].socket_at(&cards[0].sockets[1]);
    assert!(socket_at(&cards, on_input, Some(BSocketSide::Output), 1.0).is_none());
    assert!(socket_at(&cards, on_input, Some(BSocketSide::Input), 1.0).is_some());
}

#[test]
fn a_cursor_over_the_header_picks_no_socket() {
    // "Don't pick socket when cursor is over node header. This allows the user
    // to always resize by dragging on the left and right side of the header."
    // An input sits ON the left edge, so without this the header's corner is
    // unreachable.
    let cards = vec![card(100.0, 100.0)];
    // The card's TOP, which in this canvas is `card.y` — Blender's `ymax`
    // under a Y that grows upward.
    let top_left = (cards[0].x, cards[0].y + UNIT * 0.5);
    assert!(
        socket_at(&cards, top_left, None, 1.0).is_none(),
        "within one row of the card's top, nothing answers",
    );
    // And the card's BOTTOM is NOT excluded — porting the subtraction without
    // flipping it guards this edge instead, and the sockets near the bottom
    // of a tall card are exactly the ones you reach for.
    let mut tall = card(100.0, 100.0);
    tall.height = UNIT * 8.0;
    tall.sockets = vec![BSocketRow { identifier: "last".into(), ..row(BSocketSide::Input, 6.0) }];
    let bottom = tall.socket_at(&tall.sockets[0]);
    assert!(
        socket_at(&[tall], bottom, None, 1.0).is_some(),
        "the last socket on a tall card has to be grabbable",
    );
}

#[test]
fn a_socket_under_another_card_is_not_grabbable_through_it() {
    // `socket_is_occluded`, walking the draw order reversed: the card on top
    // answers. Without this, dragging a card over another lets you grab the
    // one underneath.
    let under = card(100.0, 100.0);
    let on_input = under.socket_at(&under.sockets[1]);
    // A card drawn later, covering that socket.
    let over = BNodeCard { x: on_input.0 - 50.0, y: on_input.1 - 50.0, ..card(0.0, 0.0) };
    assert!(socket_at(&[under.clone()], on_input, None, 1.0).is_some(), "reachable alone");
    assert!(socket_at(&[under, over], on_input, None, 1.0).is_none(), "covered");
}
