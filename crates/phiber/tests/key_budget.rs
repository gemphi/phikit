//! Keys are spelt once. A domain that has moved to `state/key/` is never
//! addressed by a hand-written path anywhere else: no `vec!["file", …]`, no
//! `"ctx.file.…"` string, outside `state/key/file.rs`. The list of moved
//! domains grows as `structure.md` §5 proceeds; each one added here stays
//! moved.

use std::path::{Path, PathBuf};

/// Domains whose keys have moved, and the `state/key/` file each lives in.
const MOVED: &[(&str, &str)] = &[
    ("file", "file"),
    ("outliner", "outliner"),
    ("buts", "buts"),
    ("view3d", "view3d"),
    ("image", "image"),
    ("scene", "scene"),
    ("zone", "zone"),
    ("zone_size", "zone"),
    ("area", "area"),
    ("area_size", "area"),
    ("switcher", "area"),
    ("area_resize", "area"),
    ("menu", "menu"),
    ("modal", "modal"),
    ("reader", "modal"),
    ("activity", "activity"),
    ("window", "app"),
    ("theme", "app"),
    ("use_gradients", "app"),
    ("link", "app"),
    ("edit", "app"),
    ("edit_undo", "app"),
    ("command", "app"),
    ("status", "app"),
    ("profile", "app"),
    ("collab", "app"),
    ("space", "ctx"),
    ("view_layer", "ctx"),
    ("object", "ctx"),
    ("mode", "ctx"),
    ("tool", "ctx"),
    ("frame", "ctx"),
    ("frame_step", "ctx"),
    ("frame_start", "ctx"),
    ("frame_end", "ctx"),
    ("fps", "ctx"),
    ("playing", "ctx"),
    ("selected", "ctx"),
    ("markers", "ctx"),
    ("op", "op"),
    ("screen", "screen"),
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            // A sibling `*_tests.rs` is a test module in a file of its own
            // (`#[path] mod …_tests`), so it is skipped for the same reason
            // an inline `#[cfg(test)]` module is — see `test_module_at`.
            let is_test_module =
                path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.ends_with("_tests.rs"));
            if !is_test_module {
                out.push(path);
            }
        }
    }
}

/// The line a file's `#[cfg(test)]` module starts on, or its length. A test
/// asserting `is_transient("ctx.file.selected")` is stating the encoding it
/// checks: routing that through the key enum would make the assertion agree
/// with itself whatever the enum encoded to, which is the one thing it exists
/// to catch. The rule is about the code that READS and WRITES state.
fn test_module_at(source: &str) -> usize {
    source
        .lines()
        .position(|line| line.trim_start().starts_with("#[cfg(test)]"))
        .unwrap_or(usize::MAX)
}

#[test]
fn a_moved_domains_keys_are_spelt_only_in_its_keys_file() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let mut violations = Vec::new();
    for file in files {
        let rel = file.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        let src = std::fs::read_to_string(&file).unwrap();
        for (domain, keys_file) in MOVED {
            if rel == format!("state/key/{keys_file}.rs") {
                continue;
            }
            for (n, line) in src.lines().enumerate().take(test_module_at(&src)) {
                let spelt = line.contains(&format!("vec![\"{domain}\","))
                    || line.contains(&format!("vec![\"{domain}\"]"))
                    || line.contains(&format!("\"ctx.{domain}."))
                    || line.contains(&format!("\"ctx.{domain}\""));
                if spelt {
                    violations.push(format!("{rel}:{}: {}", n + 1, line.trim()));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "keys spelt outside state/keys — use the domain's BKey enum:\n  {}",
        violations.join("\n  ")
    );
}

#[test]
fn every_file_key_round_trips_through_its_path() {
    use phiber::v2::state::{BKey, FileKey};
    for key in [FileKey::Path, FileKey::Selected, FileKey::Filter("filter_blender"), FileKey::SidebarExpanded("Bookmarks"), FileKey::Bookmark("3")] {
        let path = key.path();
        assert!(path.encode().starts_with("ctx.file."), "{key:?} → {}", path.encode());
    }
    assert_eq!(FileKey::SidebarExpanded("Recent").path().encode(), "ctx.file.sidebar.Recent.expanded");
}
