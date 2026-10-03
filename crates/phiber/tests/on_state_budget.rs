//! One question: is this control ON? One carrier: `BControl::active`.
//!
//! A control that can be on said so two ways, and most said only one of them.
//! Some swapped a glyph — `"check"` for `""`, `"check_square"` for `"square"`,
//! a chevron for its mirror. Some set `active`, which is what `chrome::paint`
//! reads to give the control its fill. The two are not alternatives: the glyph
//! says WHICH row is current when you are reading the list, the fill says it
//! when you are not. Fourteen call sites set the glyph alone, so they rendered
//! identically on and off at a glance; three popover anchors set neither, so an
//! open popover's button looked shut.
//!
//! This counts them. The list below only shrinks, and a file not on it may
//! never join — the same contract `level_structure`'s `KNOWN_OVER` and
//! `style_budget` run on, because a backlog written down is one that gets
//! worked off and a backlog waved at is not.
//!
//! Why it went unnoticed for so long is worth keeping: `view/text.rs`, the
//! snapshot every view test reads, printed `active` for `Tab`, `RailEntry` and
//! `MenuAnchor` but not for `Button` or `MenuItem` — and every divergence is a
//! `Button` or a `MenuItem`. No snapshot could see them. It prints both now,
//! which is the half of this fix that stops it happening again.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn v2() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2")
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// A `BControl` chain, from `BControl::new()` to the end of its builder run.
/// Chains wrap across lines, so this walks from each `BControl::new()` to the
/// first `;` — which is where a builder chain ends in this codebase.
fn control_chains(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let bytes: Vec<&str> = text.lines().collect();
    for (n, line) in bytes.iter().enumerate() {
        if !line.contains("BControl::new()") {
            continue;
        }
        let mut chain = String::new();
        for row in bytes.iter().skip(n) {
            chain.push_str(row);
            chain.push(' ');
            if row.contains(';') {
                break;
            }
        }
        out.push((n + 1, chain));
    }
    out
}

/// The glyph pairs that mean "this is on": a tick, a ticked box, and the
/// chevrons a fold turns over.
const STATE_GLYPHS: [&str; 4] = ["\"check\"", "\"check_square\"", "\"chevron_down\"", "\"triangle_down\""];

/// Files whose controls still say "on" with a glyph and no fill.
///
/// Each line is a decision someone has to make, not a shrug — the outliner's
/// nine restriction toggles need one about what "off" looks like for an icon
/// that has no off glyph, and `props/pref/control/toggle.rs` is a checkbox
/// rebuilt by hand that should be `BView::check_row`. The rest are mechanical.
const KNOWN_QUIET: &[&str] = &[
    // The picker families: a tick replaces the row's own icon, so "on" costs
    // the reader the icon that told them what the row was. Mechanical, but it
    // needs the icon-slot decision the menu rows already took.
    "part/switcher/mod.rs",
    "part/modebar/mod.rs",
    "part/file/asset_sidebar.rs",
    // The undo history: a tick marks the step you are on, in a list where
    // every row is a step. One line.
    "part/menubar/tree/mod.rs",
];

#[test]
fn a_control_that_ticks_also_lights() {
    let mut files = Vec::new();
    sources(&v2(), &mut files);
    assert!(files.len() > 100, "the view tree was found: {}", files.len());

    let mut quiet: BTreeSet<String> = BTreeSet::new();
    let mut rows: Vec<String> = Vec::new();
    for file in &files {
        let rel = file.strip_prefix(v2()).unwrap().to_string_lossy().replace('\\', "/");
        let Ok(text) = std::fs::read_to_string(file) else { continue };
        for (line, chain) in control_chains(&text) {
            // Only chains that CHOOSE a glyph by state. A fixed one is not a
            // claim: the file browser's Open button wears a tick because it
            // confirms, not because anything is on (`part/file/execute.rs`).
            if !chain.contains("with_icon(if ") {
                continue;
            }
            if !STATE_GLYPHS.iter().any(|g| chain.contains(g)) {
                continue;
            }
            if chain.contains("with_active") {
                continue;
            }
            quiet.insert(rel.clone());
            rows.push(format!("{rel}:{line}"));
        }
    }

    let known: BTreeSet<String> = KNOWN_QUIET.iter().map(|s| s.to_string()).collect();
    let fresh: Vec<&String> = quiet.difference(&known).collect();
    assert!(
        fresh.is_empty(),
        "these controls tick without lighting — set `with_active` beside the glyph:\n  {}\n(all: {rows:#?})",
        fresh.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n  ")
    );

    let fixed: Vec<&String> = known.difference(&quiet).collect();
    assert!(
        fixed.is_empty(),
        "these are answered now — take them off KNOWN_QUIET:\n  {:?}",
        fixed
    );
}

/// The shared builders answer in full, whatever their callers do. These are
/// the ones every other site inherits from, so they are the ones that matter.
#[test]
fn the_shared_builders_say_both() {
    let pick = include_str!("../src/v2/view/prim/menu.rs");
    assert!(
        pick.contains("with_active(ticked)"),
        "`BPick::item` is THE picker row; it had picked the glyph alone, so every picker built on it was quieter than the same dropdown built through `BCtrlKind::Select`"
    );
    let ctrl = include_str!("../src/v2/view/draw/ctrl/mod.rs");
    assert!(ctrl.contains(".with_active(on)"), "`flag` — the checkbox — lights");
    let tree = include_str!("../src/v2/part/menubar/tree/mod.rs");
    assert!(tree.contains(".with_active(on)"), "a checkable menu row lights");
}

/// A popover's button reads as open while its popover is.
#[test]
fn every_popover_anchor_shows_that_it_is_open() {
    let mut files = Vec::new();
    sources(&v2(), &mut files);
    let mut idle: Vec<String> = Vec::new();
    for file in &files {
        let rel = file.strip_prefix(v2()).unwrap().to_string_lossy().replace('\\', "/");
        let Ok(text) = std::fs::read_to_string(file) else { continue };
        if !text.contains("BView::popover(") {
            continue;
        }
        for (line, chain) in control_chains(&text) {
            // The anchor of a popover: a control built into one.
            if !chain.contains("let anchor") && !chain.contains("anchor =") {
                continue;
            }
            if chain.contains("with_active") || chain.contains("with_submenu") {
                continue;
            }
            idle.push(format!("{rel}:{line}"));
        }
    }
    // `part/file/header.rs` is an unreferenced second menubar — nothing draws
    // it (see the menu audit); it goes when that file does.
    let live: Vec<&String> = idle.iter().filter(|r| !r.starts_with("part/file/header.rs")).collect();
    assert!(
        live.is_empty(),
        "popover anchors drawn idle while their popover is open:\n  {}",
        live.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n  ")
    );
}

/// And the snapshot can see all of it — the half of the fix that keeps it
/// fixed. Without this, none of the above is visible to any view test.
#[test]
fn the_snapshot_prints_whether_a_button_or_a_row_is_on() {
    let text = include_str!("../src/v2/view/text.rs");
    for kind in ["BView::Button(ctl)", "BView::MenuItem(ctl)"] {
        let arm = text.split(kind).nth(1).unwrap_or("").split('\n').next().unwrap_or("");
        assert!(
            arm.contains("ctl.active"),
            "{kind} must print its active state, or no snapshot can tell on from off"
        );
    }
}
