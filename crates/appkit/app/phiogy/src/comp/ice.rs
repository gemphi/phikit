//! The ICE servers a browser is told to use to find a way to another
//! caller. `data/ice.tsv` has them: its first row names its columns,
//! which are the names a browser reads, and each row after it is a
//! server. A JSON list given by the system replaces them. With none, a
//! call reaches the same machine and the same network.

use serde_json::{Map, Value};

use crate::data;

/// The servers, each as a browser reads it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ice(pub Vec<Value>);

impl Ice {
    /// The servers of a table, or of a JSON list when one is given and
    /// is a list.
    pub fn read(table: &str, given: Option<&str>) -> Ice {
        let listed = given.and_then(|text| serde_json::from_str::<Value>(text).ok()).and_then(|value| value.as_array().cloned());
        let row = data::row(table);
        let (name, server) = row.split_first().map(|(name, server)| (name.clone(), server.to_vec())).unwrap_or_default();
        let one = |cell: &Vec<&str>| {
            let pair = name.iter().zip(cell).filter(|(_, value)| !value.is_empty());
            Value::Object(pair.map(|(name, value)| (name.to_string(), Value::String(value.to_string()))).collect::<Map<String, Value>>())
        };
        Ice(listed.unwrap_or_else(|| server.iter().map(one).collect()))
    }

    pub fn json(&self) -> Value {
        Value::Array(self.0.clone())
    }
}
