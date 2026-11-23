//! A signal: what one caller sends another through the line, which does
//! not read it: a WebRTC description or candidate, or how they are. And
//! a post: what a page whose line is open asks of the line.

use serde_json::Value;

use super::Wire;

/// What one caller sends another.
#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    /// The page it is for.
    pub to: String,
    pub body: Value,
}

/// What a page asks of the line: to be in the call or out of it, and
/// signals to pass on.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Post {
    /// The page that asks.
    pub peer: String,
    pub join: Option<bool>,
    pub send: Vec<Signal>,
}

impl Post {
    /// A post as a page writes it. Of what it sends the first `most` are
    /// read, and of those the ones that name a page and carry something:
    /// an object, or a list.
    pub fn read(value: &Value, wire: &Wire, most: usize) -> Post {
        let carried = |body: &Value| body.is_object() || body.is_array();
        let one = |row: &Value| {
            let (to, body) = (row.get(&wire.to)?.as_str()?, row.get(&wire.signal)?);
            carried(body).then(|| Signal { to: to.to_string(), body: body.clone() })
        };
        let sent = value.get(&wire.send).and_then(Value::as_array).cloned().unwrap_or_default();
        let peer = value.get(&wire.peer).and_then(Value::as_str).unwrap_or_default().to_string();
        Post { peer, join: value.get(&wire.join).and_then(Value::as_bool), send: sent.iter().take(most).filter_map(one).collect() }
    }
}
