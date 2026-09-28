//! One type per level, matched — not a struct per element.
//!
//! A level states its parts as the variants of one `B…Part` enum and answers
//! `BFlexed` once, matching on itself: `BCardPart`, `BModalPart`, `BZonePart`,
//! `BAreaPart`, `BPanePart`, `BTablePart`, `BTreePart`. The parts of one
//! thing are variants of one type; they are not a zero-sized struct each.
//!
//! The renderer drifted the other way once — twenty-four unit structs
//! (`BMenuRow`, `BMenuIconSlot`, `BProgressTrack`, …), one per element, each
//! with its own `impl BFlexed`. Every one of them was a name for a shape
//! nothing else could reach, so nothing was shared and the hierarchy said
//! nothing. This test is the ratchet that keeps that from happening again.

use std::path::{Path, PathBuf};

fn draw_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/v2/view/draw")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("draw dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn body(path: &Path) -> String {
    let src = std::fs::read_to_string(path).expect("source");
    match src.find("#[cfg(test)]") {
        Some(at) => src[..at].to_string(),
        None => src,
    }
}

#[test]
fn a_level_answers_bflexed_once() {
    let root = draw_root();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    files.sort();

    let mut offenders: Vec<String> = Vec::new();
    for path in &files {
        let impls = body(path).matches("impl BFlexed for").count();
        if impls > 1 {
            let rel = path.strip_prefix(&root).unwrap().display();
            offenders.push(format!("  {rel}: {impls} `impl BFlexed` — make them variants of one enum"));
        }
    }
    assert!(
        offenders.is_empty(),
        "a level states its parts as one enum's variants, not a struct each:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_parts_of_a_level_are_an_enum() {
    // A file that answers `BFlexed` for a *struct* is only right when that
    // struct is the level itself carrying its own fields (a measurement, a
    // published handle) — never when it is one element of several. The
    // enum-named ones are listed here so adding a struct is a deliberate act.
    let root = draw_root();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    files.sort();

    let mut struct_backed: Vec<String> = Vec::new();
    for path in &files {
        let src = body(path);
        for line in src.lines() {
            let Some(rest) = line.trim().strip_prefix("impl BFlexed for ") else {
                continue;
            };
            let name = rest.trim_end_matches(" {").trim();
            let declared_as_enum = src.contains(&format!("enum {name} "))
                || src.contains(&format!("enum {name}<"));
            if !declared_as_enum {
                let rel = path.strip_prefix(&root).unwrap().display();
                struct_backed.push(format!("{rel}: {name}"));
            }
        }
    }
    // The survivors are levels whose single part carries its own fields.
    let allowed = ["region/mod.rs: BRegionFrame", "space/part.rs: BSpacePart", "space/seam.rs: BMeasure", "bar.rs: BBarPart"];
    let unexpected: Vec<&String> = struct_backed
        .iter()
        .filter(|s| !allowed.contains(&s.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "these answer BFlexed for a struct — should they be one enum's variants?\n  {:?}\n(if deliberate, add to the allowlist above with a reason)",
        unexpected
    );
}
