//! Every level of the tree has the same shape, and no file outgrows a read.
//!
//! The rule, in one line: **`mod.rs` is the structure, `set.rs` is the
//! content, and a member of a level gets a folder of its own with the same
//! two files.** One shape everywhere means "where is X defined?" has one
//! answer whichever level you are in.
//!
//! The line cap is the other half of the same idea. A file that has grown
//! past a screenful has usually stopped being one thing.

use std::path::{Path, PathBuf};

/// The eight levels of the space tree (`docs/bui-tree.md`), plus the two
/// that hang off them.
const LEVEL: &[&str] = &["space", "zone", "area", "host", "region", "pane", "part", "modal"];

/// Longest a source file may be. Not a style preference: past this a file is
/// doing more than one job, and the split is nearly always obvious.
const CAP: usize = 240;

fn v2() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2")
}

fn lines(path: &Path) -> usize {
    std::fs::read_to_string(path).map(|s| s.lines().count()).unwrap_or(0)
}

/// Every `.rs` under `src/v2`, relative to it.
fn source_list() -> Vec<PathBuf> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entry_list) = std::fs::read_dir(dir) else { return };
        for entry in entry_list.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    let root = v2();
    let mut out = Vec::new();
    walk(&root, &root, &mut out);
    out.sort();
    out
}

#[test]
fn every_level_is_a_folder_with_a_mod_and_a_set() {
    let root = v2();
    let mut wrong = Vec::new();
    for level in LEVEL {
        let dir = root.join(level);
        if !dir.is_dir() {
            wrong.push(format!("{level}: not a folder"));
            continue;
        }
        // A level file beside its folder is the old shape and the reason the
        // tree used to have four different ones.
        if root.join(format!("{level}.rs")).exists() {
            wrong.push(format!("{level}.rs sits beside {level}/ — the type belongs in {level}/mod.rs"));
        }
        for file in ["mod.rs", "set.rs"] {
            if !dir.join(file).exists() {
                wrong.push(format!("{level}/{file} is missing"));
            }
        }
    }
    assert!(wrong.is_empty(), "levels are not the same shape:\n  {}", wrong.join("\n  "));
}

#[test]
fn a_member_of_a_level_is_a_folder_not_a_loose_file() {
    let root = v2();
    let mut wrong = Vec::new();
    for level in LEVEL {
        let Ok(entry_list) = std::fs::read_dir(root.join(level)) else { continue };
        for entry in entry_list.flatten() {
            let path = entry.path();
            let Some(name) = path.file_stem().map(|n| n.to_string_lossy().to_string()) else { continue };
            if path.is_dir() || path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            // `mod.rs` and `set.rs` are the level's own two files.
            if name != "mod" && name != "set" {
                wrong.push(format!("{level}/{name}.rs is a loose member — give it {level}/{name}/mod.rs"));
            }
        }
    }
    assert!(wrong.is_empty(), "members are not folders:\n  {}", wrong.join("\n  "));
}

/// A module is a file *or* a folder, never both. `part/activity.rs` sitting
/// beside `part/activity/` splits one subject across two places with no rule
/// saying which half goes where — which is what the tree used to do in
/// seventeen places.
#[test]
fn no_file_sits_beside_a_folder_of_the_same_name() {
    let root = v2();
    let pair: Vec<String> = source_list()
        .into_iter()
        .filter(|path| {
            let Some(stem) = path.file_stem() else { return false };
            path.file_name().is_some_and(|n| n != "mod.rs") && root.join(path.parent().unwrap()).join(stem).is_dir()
        })
        .map(|path| format!("{} — fold it into {}/mod.rs", path.display(), path.with_extension("").display()))
        .collect();
    assert!(pair.is_empty(), "a module is a file or a folder, not both:\n  {}", pair.join("\n  "));
}

#[test]
fn no_source_file_outgrows_a_read() {
    let over: Vec<String> = source_list()
        .into_iter()
        .map(|path| (lines(&v2().join(&path)), path))
        .filter(|(n, _)| *n > CAP)
        .map(|(n, path)| format!("{} ({n})", path.display()))
        .collect();
    // The backlog is written down rather than waved at: this list only
    // shrinks, and a file not on it may never cross the cap.
    let known: Vec<&str> = KNOWN_OVER.to_vec();
    let fresh: Vec<&String> = over.iter().filter(|row| !known.iter().any(|k| row.starts_with(k))).collect();
    assert!(fresh.is_empty(), "these files went over {CAP} lines:\n  {}", fresh.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n  "));

    let fixed: Vec<&&str> = known.iter().filter(|k| !over.iter().any(|row| row.starts_with(**k))).collect();
    assert!(fixed.is_empty(), "these are under {CAP} now — take them off KNOWN_OVER:\n  {fixed:?}");
}

/// Files still over the cap, worked down over time. Every entry here is a
/// file that has not been split yet, not a file that is allowed to be long.
const KNOWN_OVER: &[&str] = &[
    "app/mod.rs",
    "area/size/mod.rs",
    "host/mod.rs",
    "part/file/listing.rs",
    "part/file/set.rs",
    "part/mod.rs",
    "part/outliner/mod.rs",
    "part/pref/mod.rs",
    "region/mod.rs",
    "scene/render_job.rs",
    "scene/render_save.rs",
    "scene/source.rs",
    "view/draw/card.rs",
    "view/draw/pane.rs",
    "view/draw/region/mod.rs",
    "view/draw/tab/mod.rs",
    "view/mod.rs",
    "view/prim/card.rs",
];
