//! `BlendSocket`'s own tests — split out of `socket.rs`, which outgrew the
//! 240-line read budget when the nine display shapes landed. Reached by
//! `#[path]` from it, so the module path is still `shade::socket::tests`.

use super::*;

fn sock(flag: i32) -> BlendSocket {
    BlendSocket { flag, ..Default::default() }
}

#[test]
fn an_unavailable_socket_is_the_one_the_layout_skips() {
    // `is_available` is exactly one bit, and it is the bit that decides
    // whether a socket takes a row — so a node in single-operand mode is
    // shorter than the same node in two-operand mode.
    assert!(sock(0).available());
    assert!(!sock(SOCK_UNAVAIL).available());
    // Being linked, or selected, is not being unavailable.
    assert!(sock(SOCK_IS_LINKED).available());
    assert!(sock(1).available(), "SOCK_SELECT is not SOCK_UNAVAIL");
}

#[test]
fn a_hidden_socket_is_available_but_not_drawn() {
    // The two are separate bits and mean separate things: hidden is the
    // user's choice (Ctrl+H), unavailable is the node type's.
    let hidden = sock(SOCK_HIDDEN);
    assert!(hidden.available(), "hiding a socket does not make it unavailable");
    assert!(!hidden.visible());
    assert!(sock(0).visible());
}

#[test]
fn the_flag_bits_are_blenders_own_values() {
    // Spelt out, because a wrong shift here silently hides every socket or
    // shows every disabled one, and both look like a layout bug.
    assert_eq!(SOCK_HIDDEN, 2);
    assert_eq!(SOCK_IS_LINKED, 4);
    assert_eq!(SOCK_UNAVAIL, 8);
    assert_eq!(SOCK_MULTI_INPUT, 2048);
}

#[test]
fn a_multi_input_socket_says_so() {
    assert!(sock(SOCK_MULTI_INPUT).multi());
    assert!(!sock(SOCK_IS_LINKED).multi());
}

#[test]
fn an_input_takes_one_link_and_an_output_takes_four_thousand() {
    // `sock->limit = (in_out == SOCK_IN ? 1 : 0xFFF)` at creation
    // (`node.cc:3090`). The asymmetry is the whole reason a link dropped on
    // a connected input replaces it: one in, many out.
    let input = BlendSocket { limit: 1, ..Default::default() };
    let output = BlendSocket { limit: 0xFFF, ..Default::default() };
    assert_eq!(input.link_limit(), 1);
    assert_eq!(output.link_limit(), 4095);
    assert_eq!(0xFFF, MULTI_INPUT_LIMIT, "an output's default IS the multi limit");
}

#[test]
fn a_multi_input_ignores_its_stored_limit() {
    // The first thing `node_socket_link_limit` checks, before the type and
    // before the field — a Join Geometry input is stored with limit 1.
    let joined = BlendSocket { limit: 1, flag: SOCK_MULTI_INPUT, ..Default::default() };
    assert_eq!(joined.link_limit(), MULTI_INPUT_LIMIT);
}
