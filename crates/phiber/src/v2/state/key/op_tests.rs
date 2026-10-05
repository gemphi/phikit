//! `OpKey` tests — a sibling file so `op.rs` stays inside the 240-line read
//! budget (`level_structure`), the way `scene/cursor_tests.rs` sits beside
//! `scene/cursor.rs`.

use super::*;
use std::collections::BTreeSet;

#[test]
fn every_op_is_found_by_its_own_path() {
    // The round trip is the whole point of `tail` existing: `path` writes
    // the spelling and `of` reads it, so neither can drift from the other.
    for op in OpKey::ALL {
        assert_eq!(OpKey::of(&op.path()), Some(*op), "{op:?}");
    }
}

#[test]
fn all_lists_every_variant_exactly_once() {
    // `ALL` is hand-kept because Rust cannot walk variants. Nothing but
    // this notices a variant added to the enum and not to the list — and
    // `of` would then silently never return it.
    let tails: BTreeSet<&str> = OpKey::ALL.iter().map(|op| op.tail()).collect();
    assert_eq!(tails.len(), OpKey::ALL.len(), "two ops share a tail");
    let source = include_str!("op.rs");
    let body = {
        let from = source.find("pub enum OpKey {").expect("the enum");
        let rest = &source[from..];
        &rest[..rest.find("\n}\n").expect("its end")]
    };
    let declared = body
        .lines()
        .filter_map(|line| {
            let name = line.split("//").next()?.trim().strip_suffix(',')?;
            (!name.is_empty()
                && name.chars().next()?.is_ascii_uppercase()
                && !name.contains(' '))
            .then_some(name)
        })
        .count();
    assert_eq!(declared, OpKey::ALL.len(), "the enum declares {declared}");
}

#[test]
fn a_path_outside_the_op_family_is_not_an_op() {
    // `of` is strict about the family, which is what let the dispatcher
    // stop checking it separately. A key that ends in an op's word — and
    // `ctx.save` would — must not be read as that op.
    assert_eq!(OpKey::of(&BPath::owned(vec!["ctx", "save"])), None);
    assert_eq!(OpKey::of(&BPath::owned(vec!["op", "save", "as"])), None, "two segments only");
    assert_eq!(OpKey::of(&BPath::owned(vec!["op"])), None);
    assert_eq!(OpKey::of(&BPath::owned(vec!["op", "nothing_of_the_sort"])), None);
}
