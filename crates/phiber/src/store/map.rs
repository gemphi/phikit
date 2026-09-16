//! BMap — generic key → spec lookup.
//!
//! Every level builds one `BMap` from its key type to its spec type
//! and exposes a `spec_for(key)` helper through `set.rs`.

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug, Default)]
pub struct BMap<K, S> {
    pub map: HashMap<K, S>,
}

impl<K, S> BMap<K, S>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn insert(&mut self, key: K, spec: S) {
        self.map.insert(key, spec);
    }

    pub fn spec_for(&self, key: K) -> Option<&S> {
        self.map.get(&key)
    }

    pub fn all(&self) -> Vec<&S> {
        self.map.values().collect()
    }

    pub fn contains(&self, key: K) -> bool {
        self.map.contains_key(&key)
    }
}

impl<K, S> BMap<K, S>
where
    K: Eq + Hash + Clone,
    S: Clone,
{
    pub fn from_pairs(pairs: &[(K, S)]) -> Self {
        Self {
            map: pairs.iter().cloned().collect(),
        }
    }
}
