//! A roster: who is in a room's call, in the order their lines were
//! opened. A page is told a new one each time someone joins or leaves.

use serde_json::{Map, Value};

use super::Wire;
use crate::entity::Caller;

/// Who is in a call: each one's page, and who they are.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Roster {
    pub list: Vec<(String, Caller)>,
}

impl Roster {
    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// The pages in it, in order.
    pub fn peer(&self) -> Vec<&str> {
        self.list.iter().map(|(peer, _)| peer.as_str()).collect()
    }

    /// The roster as a page reads it: a list of objects, each a peer, a
    /// name and a handle, which is null for a guest.
    pub fn json(&self, wire: &Wire) -> Value {
        let one = |(peer, caller): &(String, Caller)| {
            let mut row = Map::new();
            row.insert(wire.peer.clone(), Value::String(peer.clone()));
            row.insert(wire.name.clone(), Value::String(caller.name.clone()));
            row.insert(wire.handle.clone(), caller.handle.clone().map_or(Value::Null, Value::String));
            Value::Object(row)
        };
        Value::Array(self.list.iter().map(one).collect())
    }
}
