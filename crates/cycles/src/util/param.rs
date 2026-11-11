// SPDX-License-Identifier: Apache-2.0
//! What the members of Cycles' parameter classes start as: `data/param.tsv`,
//! which `tool/param.py` writes from the constructors of `BVHParams` and
//! `SceneParams`.

use super::row_list;

/// The number `member` of the class `owner` starts as; true is 1 and an
/// enumerator is its number. Zero for a member the table does not have.
pub fn param_default(owner: &str, member: &str) -> f64 {
    let mut row = row_list(include_str!("../../data/param.tsv"));
    let found = row.find(|row| row.first() == Some(&owner) && row.get(1) == Some(&member)).and_then(|row| row.get(2)?.parse().ok());
    debug_assert!(found.is_some(), "data/param.tsv has no {owner}::{member}");
    found.unwrap_or(0.0)
}
