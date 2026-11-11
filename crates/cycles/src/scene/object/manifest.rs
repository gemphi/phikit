// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/object.cpp`
//! (`ObjectManager::get_cryptomatte_objects`, `get_cryptomatte_assets`):
//! the manifest a Cryptomatte pass is read with, each name with the hash
//! it is drawn as.

use std::collections::HashSet;

use super::{Object, ObjectManager};
use crate::util::murmur_hash3;

/// `{"name":"hash",...}` for each name once, in the order met. As in
/// Cycles, no names at all gives `}` alone.
fn manifest<'a>(name: impl Iterator<Item = &'a str>) -> String {
    let mut seen = HashSet::new();
    let mut out = String::from("{");
    for name in name.filter(|name| seen.insert(*name)) {
        out.push_str(&format!("\"{name}\":\"{:08x}\",", murmur_hash3(name.as_bytes(), 0)));
    }
    out.pop();
    out.push('}');
    out
}

impl ObjectManager {
    /// `get_cryptomatte_objects`.
    pub fn cryptomatte_object(object: &[Object]) -> String {
        manifest(object.iter().map(|object| object.node.name.as_str()))
    }

    /// `get_cryptomatte_assets`.
    pub fn cryptomatte_asset(object: &[Object]) -> String {
        manifest(object.iter().map(|object| object.node.string("asset_name")))
    }
}
