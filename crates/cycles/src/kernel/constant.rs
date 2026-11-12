// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! The kernel's flags and limits, by the names Cycles gives them in
//! `intern/cycles/kernel/types.h` and `kernel/features.h`. The numbers are
//! `data/constant.tsv`, which `tool/constant.py` writes from those files.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::util::row_list;

fn table() -> &'static HashMap<&'static str, i64> {
    static TABLE: OnceLock<HashMap<&'static str, i64>> = OnceLock::new();
    let read = || row_list(include_str!("../../data/constant.tsv")).filter_map(|row| Some((*row.get(1)?, row.get(2)?.parse().ok()?))).collect();
    TABLE.get_or_init(read)
}

/// The number of an enumerator or `#define`, such as `PASS_COMBINED` or
/// `MAX_TAB_SOBOL_SAMPLES`.
pub fn constant(name: &str) -> Option<i64> {
    table().get(name).copied()
}

/// A constant as the bits of a flag word, such as `FILTER_CLOSURE_DIFFUSE`.
/// Zero for a name the table does not have.
pub fn flag(name: &str) -> u32 {
    let found = constant(name);
    debug_assert!(found.is_some(), "data/constant.tsv has no {name}");
    found.unwrap_or(0) as u32
}

#[cfg(test)]
mod test;
