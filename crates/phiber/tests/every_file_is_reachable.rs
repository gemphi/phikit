//! Every source file is reachable from the crate root.
//!
//! Twice now a finished feature has sat in the tree doing nothing because no
//! `mod` declared its file: `theme/style/elevation.rs` (the elevation scale —
//! `BElevation` was defined and never exported, so seven call sites failed to
//! resolve) and `view/draw/popover/leave.rs` (closing a menu the pointer has
//! strayed from, Blender's `BLOCK_MOVEMOUSE_QUIT`, which simply never ran).
//!
//! An unwired file is worse than a missing one. It reads as done, it is
//! listed in the folder's README, `cargo check` says nothing about it — and
//! the behaviour it implements is absent. Nothing in the compiler catches it,
//! so this does.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn v2() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2")
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

/// The module names a file declares, whatever the visibility: `mod name;`,
/// `pub mod name;`, `pub(crate) mod name;`, `pub(super) mod name;` — plus
/// `#[path = "other.rs"] mod name;`, which is how the `app` unit tests hang
/// off `import.rs`. Missing a visibility spelling would report a wired file
/// as unwired, which is the one failure that would get this test deleted.
fn declared_in(text: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("#[path = \"") {
            if let Some(file) = rest.split('"').next() {
                paths.insert(file.to_string());
            }
        }
        // Drop a leading `pub`, then a `(..)` restriction if there is one,
        // and the line must then begin with `mod ` — so `use a::mod_x` and
        // prose mentioning a module never count as declarations.
        let mut rest = line.strip_prefix("pub").unwrap_or(line).trim_start();
        if rest.starts_with('(') {
            match rest.find(')') {
                Some(close) => rest = rest[close + 1..].trim_start(),
                None => continue,
            }
        }
        if let Some(rest) = rest.strip_prefix("mod ") {
            if let Some(name) = rest.split(&[';', ' ', '{'][..]).next() {
                if !name.is_empty() {
                    names.insert(name.to_string());
                }
            }
        }
    }
    (names, paths)
}

/// A file is reachable when the module that owns its folder declares it —
/// by name, or by an explicit `#[path]`.
#[test]
fn no_source_file_is_unwired() {
    let root = v2();
    let mut unwired = Vec::new();
    for file in source_list() {
        let name = file.file_stem().unwrap().to_string_lossy().to_string();
        let file_name = file.file_name().unwrap().to_string_lossy().to_string();
        // `v2/mod.rs` is the crate's own entry; a folder's `mod.rs` is
        // declared by its PARENT, which the folder case below covers.
        if file_name == "mod.rs" {
            continue;
        }
        let dir = root.join(file.parent().unwrap());
        // The owning module is the folder's `mod.rs`; any sibling may also
        // carry the `#[path]` that pulls a file in.
        let mut owners = vec![dir.join("mod.rs")];
        if let Ok(list) = std::fs::read_dir(&dir) {
            owners.extend(list.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "rs")));
        }
        let wired = owners.iter().any(|owner| {
            let Ok(text) = std::fs::read_to_string(owner) else { return false };
            let (names, paths) = declared_in(&text);
            names.contains(&name) || paths.contains(&file_name)
        });
        if !wired {
            unwired.push(format!("{} — nothing declares it; add `mod {name};` to its folder's mod.rs", file.display()));
        }
    }
    assert!(
        unwired.is_empty(),
        "these files are in the tree but reachable from nothing:\n  {}",
        unwired.join("\n  ")
    );
}

/// The same for folders: a folder with a `mod.rs` its parent never declares
/// is an entire subtree the build does not see.
#[test]
fn no_folder_is_unwired() {
    let root = v2();
    let mut unwired = Vec::new();
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        let Ok(entry_list) = std::fs::read_dir(dir) else { return };
        for entry in entry_list.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if path.join("mod.rs").exists() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let owner = dir.join("mod.rs");
                let declared = std::fs::read_to_string(&owner)
                    .map(|text| declared_in(&text).0.contains(&name))
                    .unwrap_or(false);
                if !declared {
                    let shown = path.strip_prefix(root).unwrap_or(&path).display().to_string();
                    out.push(format!("{shown}/ — nothing declares it; add `mod {name};`"));
                }
            }
            walk(&path, root, out);
        }
    }
    walk(&root, &root, &mut unwired);
    assert!(
        unwired.is_empty(),
        "these folders are in the tree but reachable from nothing:\n  {}",
        unwired.join("\n  ")
    );
}
