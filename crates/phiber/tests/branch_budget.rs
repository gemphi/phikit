//! Branching budget: a chain of `else if` is a match that has not been written
//! yet, and the number of them may only fall.
//!
//! Phiber's levels exist so complexity is expressed as types and tables rather
//! than as control flow — a part is a `BPartType`, a cap is a `BCapShape`, a
//! socket's colour is a row in a table. A ladder of `} else if` is the opposite
//! shape: it hides the ORDER of its tests inside its own structure, so the one
//! thing that decides the answer is the one thing you cannot see.
//!
//! This is the same budget idea as `style_budget.rs`, pointed at branching:
//! `golden/branch_budget.txt` records what each file has today, a file may
//! never exceed its own number, and a new file starts at zero. Lowering one is
//! always allowed and is the direction of travel.
//!
//! Two that came down writing this, both of them the same mistake:
//!
//! - `theme/style/node_class/infer.rs` had **eleven**, classifying a node by
//!   searching its name for keywords. The order was the entire rule — "Mapping"
//!   is a vector node and "MapRange" a converter, and only position decides —
//!   and it is a table now, where that order is data you read down.
//! - `app/op/mod.rs` had one per operator it handles, each comparing an encoded
//!   path against a freshly encoded `OpKey`. It is a `match OpKey::of(path)`,
//!   which also stops it allocating a path per comparison to answer a question
//!   the key can answer itself.
//!
//! `else if` is not banned and should not be: two tests on unrelated things are
//! clearer as an if than as a match on a tuple. The budget is about ladders.

use std::collections::BTreeMap;

fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2")
}

fn record() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/branch_budget.txt")
}

/// `} else if` per file, with comments stripped so prose about a ladder is not
/// counted as one.
fn counted() -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            let code: String = text
                .lines()
                .map(|l| l.split("//").next().unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n");
            let n = code
                .match_indices("else")
                .filter(|(i, _)| code[i + 4..].trim_start().starts_with("if"))
                .count();
            if n > 0 {
                let rel = path.strip_prefix(root()).unwrap_or(&path).display().to_string();
                out.insert(rel.replace('\\', "/"), n);
            }
        }
    }
    out
}

/// The recorded budget, written on the first run so the baseline is this
/// tree's own rather than one measured somewhere else.
///
/// It is read at RUN time, not with `include_str!`, for exactly that reason:
/// the two trees this crate lives in have diverged, and a baseline compiled in
/// from one of them would fail in the other for reasons that have nothing to do
/// with branching. The record is committed, so a tree that loses it and
/// re-baselines shows that as a diff.
fn budget() -> BTreeMap<String, usize> {
    let at = record();
    let text = match std::fs::read_to_string(&at) {
        Ok(text) => text,
        Err(_) => {
            let now = counted();
            let mut out = String::from(
                "# `} else if` per file under src/v2, as of the day this was recorded.\n\
                 # A file may never exceed its number; a file not listed has a budget of\n\
                 # zero. Lowering one is always allowed — raising one is the thing this\n\
                 # guard exists to refuse. Written by `tests/branch_budget.rs` on its\n\
                 # first run, so the baseline is this tree's own.\n",
            );
            for (file, n) in &now {
                out.push_str(&format!("{n} {file}\n"));
            }
            let _ = std::fs::create_dir_all(at.parent().unwrap_or(&at));
            let _ = std::fs::write(&at, &out);
            out
        }
    };
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .filter_map(|l| {
            let (n, p) = l.trim().split_once(' ')?;
            Some((p.trim().to_string(), n.trim().parse().ok()?))
        })
        .collect()
}

#[test]
fn no_file_branches_more_than_its_budget() {
    let (now, allowed) = (counted(), budget());
    let mut over = Vec::new();
    for (file, count) in &now {
        let cap = allowed.get(file).copied().unwrap_or(0);
        if *count > cap {
            over.push(format!("{file}: {count} else-if (budget {cap})"));
        }
    }
    assert!(
        over.is_empty(),
        "branching grew — a ladder of tests is a match waiting to be written:\n  {}\n\
         Lowering a budget is always allowed; raising one is not.",
        over.join("\n  "),
    );
}

#[test]
fn the_budget_has_no_rows_for_files_that_came_down_to_zero() {
    // A file that lost its last `else if` should lose its row, so the record
    // stays a list of what is left rather than a history of what there was.
    let (now, allowed) = (counted(), budget());
    let stale: Vec<&String> = allowed.keys().filter(|f| !now.contains_key(*f)).collect();
    assert!(stale.is_empty(), "these have no `else if` left and can leave the budget: {stale:?}");
}
