//! How a host's socket-value edits are spelled in one state value, and read
//! back. One key holds the whole list (`NodeKey::Values`), the same reasoning
//! as the link and add overlays: one store write is one undo step.
//!
//! Each entry is `node/socket/value`, joined by `;`. `node` and `socket` are
//! separated by `/`, the character `bNode.name` and `bNodeSocket.identifier`
//! do not contain (`link/store.rs` says why). The value is a tagged token so
//! its type survives the trip: `b0`/`b1` boolean, `f…` float, `i…` int,
//! `v x,y,z` vector, `c r,g,b,a` colour — comma-separated, which neither a node
//! name nor an identifier uses either.

use super::{BValueEdit, BValueEdits};
use crate::phiphx::blend::shade::socket::BSocketDefault;

const FIELD: char = '/';
const NEXT: char = ';';
const COMMA: char = ',';

/// `node/socket/value;…` — every edited socket, in order.
pub fn encode(edits: &BValueEdits) -> String {
    let mut out = String::new();
    for e in &edits.edits {
        if !out.is_empty() {
            out.push(NEXT);
        }
        out.push_str(&e.node);
        out.push(FIELD);
        out.push_str(&e.socket);
        out.push(FIELD);
        out.push_str(&encode_value(&e.value));
    }
    out
}

fn encode_value(v: &BSocketDefault) -> String {
    match v {
        BSocketDefault::Bool(b) => format!("b{}", u8::from(*b)),
        BSocketDefault::Float(x) => format!("f{x}"),
        BSocketDefault::Int(n) => format!("i{n}"),
        BSocketDefault::Vector([x, y, z]) => format!("v{x}{COMMA}{y}{COMMA}{z}"),
        BSocketDefault::Color([r, g, b, a]) => format!("c{r}{COMMA}{g}{COMMA}{b}{COMMA}{a}"),
    }
}

/// A tagged value token back into a default. `None` — a malformed token — drops
/// its whole entry rather than guessing a type or a number.
fn decode_value(text: &str) -> Option<BSocketDefault> {
    let mut chars = text.chars();
    let tag = chars.next()?; // panic-safe: never splits a multibyte char
    let rest = chars.as_str();
    let nums = || rest.split(COMMA).map(|s| s.parse::<f32>().ok()).collect::<Option<Vec<f32>>>();
    match tag {
        'b' => Some(BSocketDefault::Bool(rest == "1")),
        'f' => rest.parse().ok().map(BSocketDefault::Float),
        'i' => rest.parse().ok().map(BSocketDefault::Int),
        'v' => nums().filter(|n| n.len() == 3).map(|n| BSocketDefault::Vector([n[0], n[1], n[2]])),
        'c' => nums().filter(|n| n.len() == 4).map(|n| BSocketDefault::Color([n[0], n[1], n[2], n[3]])),
        _ => None,
    }
}

/// The edits back out of the store. An entry that does not parse is dropped
/// rather than guessed at — a half-read value would show the wrong number.
pub fn decode(text: &str) -> BValueEdits {
    let mut edits = BValueEdits::default();
    for entry in text.split(NEXT).filter(|e| !e.is_empty()) {
        let mut parts = entry.splitn(3, FIELD);
        let (Some(node), Some(socket), Some(value)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        if node.is_empty() || socket.is_empty() {
            continue;
        }
        let Some(value) = decode_value(value) else { continue };
        edits.edits.push(BValueEdit { node: node.into(), socket: socket.into(), value });
    }
    edits
}

/// This host's socket-value edits, out of the store.
pub fn edits_of(state: &crate::v2::state::BState, host: crate::v2::host::BHostType) -> BValueEdits {
    use crate::v2::state::{BKey as _, NodeKey};
    decode(state.get(&NodeKey::Values(host).path()).unwrap_or(""))
}
