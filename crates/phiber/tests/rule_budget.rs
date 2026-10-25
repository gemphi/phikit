//! Rules live with their level; the dispatcher only dispatches.
//!
//! `state/action.rs` stores a value and hands the key to `state::rule` — it
//! never compares a path or names a key enum itself. And every `impl BRule`
//! in the crate is in `state/rule.rs`'s table, so a rule cannot be written
//! and then silently never run.

use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn src(rel: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2");
    std::fs::read_to_string(root.join(rel)).unwrap()
}

#[test]
fn the_dispatcher_compares_no_paths() {
    let action = src("state/action.rs");
    let code: Vec<&str> = action.lines().filter(|l| !l.trim_start().starts_with("//")).collect();
    let offending: Vec<&&str> = code
        .iter()
        .filter(|l| l.contains("path ==") || l.contains("Key::") && !l.contains("MenuKey::owns"))
        .collect();
    assert!(offending.is_empty(), "state/action.rs reads keys itself — make it a BRule:\n{offending:#?}");
}

#[test]
fn every_rule_is_in_the_table() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let table = src("state/rule.rs");
    let mut missing = Vec::new();
    let mut found = 0;
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        for line in text.lines() {
            let Some(rest) = line.trim().strip_prefix("impl BRule for ") else { continue };
            let name = rest.trim_end_matches(" {").trim();
            found += 1;
            if !table.contains(&format!("::{name},")) {
                missing.push(format!("{} ({})", name, file.display()));
            }
        }
    }
    assert!(found >= 12, "expected the rules to be found; saw {found}");
    assert!(missing.is_empty(), "rules not in state/rule.rs's table:\n  {}", missing.join("\n  "));
}

#[test]
fn a_viewpoint_becomes_two_angles_and_is_not_kept() {
    use phiber::v2::state::{BState, View3dKey, action::apply_action};
    use phiber::v2::view::prim::BAction;
    use phiber::v2::state::BKey as _;
    let mut s = BState::new();
    apply_action(&mut s, &BAction::Set(View3dKey::View.path(), "top".into()));
    assert_eq!(s.get(View3dKey::Pitch), Some("90"));
    assert_eq!(s.get(View3dKey::Yaw), Some("0"));
    assert_eq!(s.get(View3dKey::View), None);
}

#[test]
fn the_reader_gives_back_the_modal_it_covered() {
    use phiber::v2::state::{BKey as _, BState, ModalKey, action::apply_action};
    use phiber::v2::view::prim::BAction;
    let mut s = BState::new();
    s.set(ModalKey::Open, "help");
    apply_action(&mut s, &BAction::Set(ModalKey::ReaderDoc.path(), "manual".into()));
    assert_eq!(s.get(ModalKey::Open), Some("reader"));
    apply_action(&mut s, &BAction::Set(ModalKey::Open.path(), String::new()));
    assert_eq!(s.get(ModalKey::Open), Some("help"));
}

/// `state/` is the store, its keys and its rules. A reducer that belongs to
/// a level (area sizes, zone sizes, switching space, the file browser's
/// history) lives with that level; this list only ever shrinks.
#[test]
fn state_holds_only_the_store_the_keys_and_the_rules() {
    const ALLOWED: &[&str] = &[
        "action.rs", "edit.rs", "key", "log", "menu.rs", "mod.rs", "path.rs",
        "rule.rs", "seed.rs", "val.rs",
    ];
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2/state");
    let mut extra = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let code = name.ends_with(".rs") || entry.path().is_dir();
        if code && !name.starts_with('_') && !ALLOWED.contains(&name.as_str()) {
            extra.push(name);
        }
    }
    extra.sort();
    assert!(extra.is_empty(), "state/ grew a module — move it to its level: {extra:?}");
}
