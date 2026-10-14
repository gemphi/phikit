//! Every entry Blender's topbar menus hold, phiber holds too — in the same
//! order, under the same label, on the same key
//! (`truth/blender_menus.py`, `truth/menus.txt`).
//!
//! A menu is not data in Blender: `TOPBAR_MT_render` is a Python class whose
//! `draw(self, context)` calls `layout.operator(...)` in order. So the dump
//! runs those draws against a recording layout, which is the only way to get
//! the real order with the real labels, and reads the shortcuts out of
//! `blender_default.py` — matching a binding to an entry by operator *and*
//! properties, because "Render Image" and "Render Animation" are the same
//! operator and differ only in `animation = True`.
//!
//! `topbar/set.rs` carries a `Blender ref:` comment per menu, which is a
//! claim no test checked. This checks it. What it found the first time it
//! ran is the shortcut *spelling*: `WM_keymap_item_raw_to_string`
//! (`wm_keymap.cc:1222`) appends **Shift, Ctrl, Alt, OS** in that order, and
//! names a key by the event enum's `name` under `compact = false`
//! (`interface.cc:1678`) — so Blender writes "Shift Ctrl S", "Ctrl ," and
//! "Ctrl Page Down". Two of phiber's were spelled another way.
//!
//! Where phiber deliberately differs, the difference is a row in [`EXTRA`]
//! or [`RENAMED`] rather than a silence.

use phiber::v2::part::menubar::item::{BMenuItem, BMenuKind};
use phiber::v2::part::menubar::topbar::set::TOPBAR;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// One recorded entry of a Blender menu.
struct Entry {
    menu: String,
    kind: String,
    label: String,
    shortcut: String,
}

fn truth() -> Vec<Entry> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/menus.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let field: Vec<&str> = line.split('\t').collect();
            (field.len() >= 7).then(|| Entry {
                menu: field[0].to_string(),
                kind: field[2].to_string(),
                label: field[4].to_string(),
                shortcut: field[6].to_string(),
            })
        })
        .collect()
}

/// Blender's menu class → the phiber menu that stands in for it.
///
/// `TOPBAR_MT_blender` has no row here: it is phiber's own "Phiber" menu,
/// whose entries are about phiber and not comparable.
const MENU: &[(&str, &str)] = &[
    ("TOPBAR_MT_file", "File"),
    ("TOPBAR_MT_edit", "Edit"),
    ("TOPBAR_MT_render", "Render"),
    ("TOPBAR_MT_window", "Window"),
    ("TOPBAR_MT_help", "Help"),
];

/// Entries phiber adds that Blender's menu does not have.
///
/// Each is a decision, and writing it down is what keeps it one: an extra
/// that stops being wanted shows up here rather than in a diff nobody runs.
/// `(menu, label, why)`.
const EXTRA: &[(&str, &str, &str)] = &[
    ("Render", "Render Turnaround", "a turntable of the scene, which Blender has no menu entry for"),
    ("Edit", "Operator Search...", "Blender dropped it in 2.9x; phiber keeps both searches"),
];

/// Entries phiber shows under another name. `(menu, Blender's, phiber's, why)`.
const RENAMED: &[(&str, &str, &str, &str)] = &[
    ("Window", "Save Screenshot", "Save Screenshot...", "it opens the file browser, so it takes an ellipsis"),
    ("Window", "Save Screenshot (Editor)", "Save Screenshot (Editor)...", "likewise"),
    ("Help", "Save System Info", "Save System Info...", "likewise"),
];

/// The phiber menu of that name, flattened to the entries a comparison can
/// see: a label and the shortcut beside it, with separators kept because
/// their *positions* are half of what a menu reads like.
fn phiber(menu: &str) -> Vec<(&'static str, &'static str)> {
    TOPBAR
        .iter()
        .find(|spec| spec.label == menu)
        .map(|spec| spec.items.iter().map(entry).collect())
        .unwrap_or_default()
}

fn entry(item: &BMenuItem) -> (&'static str, &'static str) {
    match item.kind {
        BMenuKind::Sep => ("", ""),
        _ => (item.label, item.shortcut),
    }
}

/// What phiber calls the entry Blender calls `label`.
fn expected(menu: &str, label: &str) -> String {
    RENAMED
        .iter()
        .find(|(of, blender, _, _)| *of == menu && *blender == label)
        .map_or_else(|| label.to_string(), |(_, _, phiber, _)| (*phiber).to_string())
}

#[test]
fn every_entry_blenders_topbar_holds_phibers_holds_too() {
    let truth = truth();
    assert!(!truth.is_empty(), "the dump is there and parses");

    let mut wrong = Vec::new();
    for (blender, mine) in MENU {
        let have = phiber(mine);
        let want: Vec<&Entry> = truth.iter().filter(|row| row.menu == *blender).collect();
        assert!(!want.is_empty(), "{blender} is in the dump");
        for row in want {
            if row.kind == "sep" {
                continue;
            }
            let label = expected(mine, &row.label);
            let Some((_, shortcut)) = have.iter().find(|(seen, _)| *seen == label) else {
                wrong.push(format!("{mine} ▸ {label:?}: Blender has it and phiber's menu does not"));
                continue;
            };
            if *shortcut != row.shortcut {
                wrong.push(format!(
                    "{mine} ▸ {label:?}: Blender shows {:?}, phiber shows {shortcut:?}",
                    row.shortcut
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{} entries differ from Blender's menus:\n  {}", wrong.len(), wrong.join("\n  "));
}

#[test]
fn every_entry_phiber_adds_is_one_it_means_to_add() {
    // The other direction: an entry phiber shows that Blender does not have
    // is either in `EXTRA` or it is a drift nobody decided on.
    let truth = truth();
    let mut unrecorded = Vec::new();
    for (blender, mine) in MENU {
        let known: BTreeSet<String> = truth
            .iter()
            .filter(|row| row.menu == *blender && row.kind != "sep")
            .map(|row| expected(mine, &row.label))
            .collect();
        for (label, _) in phiber(mine).iter().filter(|(label, _)| !label.is_empty()) {
            let extra = EXTRA.iter().any(|(of, seen, _)| of == mine && seen == label);
            if !known.contains(*label) && !extra {
                unrecorded.push(format!("{mine} ▸ {label:?}"));
            }
        }
    }
    assert!(
        unrecorded.is_empty(),
        "{} entries phiber shows that Blender does not, and `EXTRA` does not account for:\n  {}",
        unrecorded.len(),
        unrecorded.join("\n  ")
    );
}

#[test]
fn the_render_menu_keeps_blenders_four_shortcuts() {
    // The four the user reaches for without opening a menu, spelled as
    // `WM_keymap_item_raw_to_string` spells them. A regression here is a key
    // that silently stops working.
    let have = phiber("Render");
    for (label, key) in [
        ("Render Image", "F12"),
        ("Render Animation", "Ctrl F12"),
        ("View Render", "F11"),
        ("View Animation", "Ctrl F11"),
    ] {
        let found = have.iter().find(|(seen, _)| *seen == label);
        assert_eq!(found.map(|(_, key)| *key), Some(key), "{label} is on {key}");
    }
}

#[test]
fn a_recorded_extra_or_rename_is_one_the_menu_really_has() {
    // `EXTRA` and `RENAMED` are records, and a stale record is worse than
    // none — it says a difference was considered when it no longer exists.
    for (menu, label, why) in EXTRA {
        assert!(
            phiber(menu).iter().any(|(seen, _)| seen == label),
            "{menu} ▸ {label:?} is recorded as an extra ({why}) but the menu does not have it"
        );
    }
    let truth = truth();
    for (menu, blender, mine, why) in RENAMED {
        let class = MENU.iter().find(|(_, name)| name == menu).map(|(class, _)| *class);
        assert!(
            truth.iter().any(|row| Some(row.menu.as_str()) == class && row.label == *blender),
            "{menu} ▸ {blender:?} is recorded as renamed to {mine:?} ({why}) but Blender's menu has no such entry"
        );
        assert!(
            phiber(menu).iter().any(|(seen, _)| seen == mine),
            "{menu} ▸ {mine:?} is recorded as the rename of {blender:?} but phiber's menu does not have it"
        );
    }
}

#[test]
fn every_menu_blender_names_is_one_this_test_maps() {
    let truth = truth();
    let unmapped: BTreeSet<&str> = truth
        .iter()
        .map(|row| row.menu.as_str())
        .filter(|menu| !MENU.iter().any(|(blender, _)| blender == menu))
        .collect();
    assert!(unmapped.is_empty(), "menus the dump names and this test does not map: {unmapped:?}");
}
