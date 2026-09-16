//! BState — generic key-value state store.
//!
//! Every key is just a key. No name-specific logic lives here.
//! All state is accessed via `get`, `set`, `has`.

pub mod def;
pub mod key;
pub mod set;

pub use def::WireframeMesh;
pub use key::BKey;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BState {
    pub hash: HashMap<String, String>,
    pub scene: Option<crate::phiphx::AssetImportResult>,
}

impl BState {
    pub fn new() -> Self {
        Self {
            hash: set::default_state(),
            scene: None,
        }
    }

    pub fn get(&self, key: &BKey) -> Option<&str> {
        self.hash.get(&key.key()).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &BKey, val: &str) {
        self.hash.insert(key.key(), val.to_string());
    }

    pub fn has(&self, key: &BKey) -> bool {
        self.hash.contains_key(&key.key())
    }
}

impl Default for BState {
    fn default() -> Self {
        Self::new()
    }
}
