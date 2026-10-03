//! How many colours mean "on", and does each one have a reason.
//!
//! `on_state_budget` settled who CARRIES the state: `BControl::active`, one
//! field, said once. This is the other half — what that field RESOLVES to.
//! There were five answers, and one of them had no callers at all.
//!
//! The dead one is the point. `rail_tab_active` was `sheen(bg_active)`, which
//! is `host_item_active_gradient` spelled again, and nothing in `src/` called
//! it. It survived because a test asserted it equalled `bg_active` — so the
//! suite was green on a function the window never ran. A budget that counts
//! surplus would not have caught that; one that insists every token is
//! REACHED does.
//!
//! The survivors are not redundant, and this file is where the difference is
//! written down rather than rediscovered:
//!
//! - a strip control's fill is swept, because it sits on the strip's surface;
//! - a free control's is the accent, because it sits on nothing in particular;
//! - a tab's is the BODY's shade, because a tab is a hole cut in its strip onto
//!   the region it selects — the one "on" that is defined by what is behind it;
//! - a sorted heading's is flat, because the heading IS its row edge to edge.
//!
//! That last pair is the same question `panel_fill` answers for the activity
//! strip (`the_activity_strip_is_card_content`): swept when a thing lies ON a
//! surface, flat when it IS one. Two places, one rule, still spelled twice.

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

/// Every semantic that paints "on", and why it is not one of the others.
const ACTIVE_FILLS: &[(&str, &str)] = &[
    ("host_item_active_gradient", "a strip control: swept, because it lies on the strip's surface"),
    ("control_fill", "a free control: the accent, because it lies on nothing in particular"),
    ("tab_active_gradient", "a tab: the body's shade, because a tab is a hole cut onto the region it selects"),
    ("ControlActive", "a sorted heading: flat, because the heading IS its row, edge to edge"),
];

/// Read every `src/v2` file once.
fn all_source() -> String {
    let mut files = Vec::new();
    sources(&v2(), &mut files);
    assert!(files.len() > 100, "the view tree was found: {}", files.len());
    files.iter().filter_map(|f| std::fs::read_to_string(f).ok()).collect::<Vec<_>>().join("\n")
}

#[test]
fn every_colour_that_means_on_is_reached_from_the_window() {
    let src = all_source();
    let mut dead: Vec<&str> = Vec::new();
    for (name, _) in ACTIVE_FILLS {
        // A definition is not a use: `fn name(` and `Name,` (an enum arm) do
        // not count, only a call or a construction somewhere else.
        let uses = src.matches(name).count();
        let definitions = src.matches(&format!("fn {name}(")).count() + src.matches(&format!("    {name},")).count();
        if uses <= definitions {
            dead.push(name);
        }
    }
    assert!(
        dead.is_empty(),
        "these mean \"on\" and nothing asks for them — delete them or call them:\n  {}\n\
         (`rail_tab_active` sat here for exactly this reason, kept alive by a test of its own)",
        dead.join("\n  ")
    );
}

#[test]
fn there_are_no_more_answers_than_these() {
    let src = all_source();
    // Anything that derives a colour FROM the active shade is a candidate
    // answer. If a new one appears, it belongs in `ACTIVE_FILLS` with a reason
    // beside it, or it is a fifth spelling of one that is already there.
    let mut found: BTreeSet<String> = BTreeSet::new();
    for line in src.lines() {
        let line = line.trim();
        if line.starts_with("//") || !line.contains("bg_active") {
            continue;
        }
        // The owner is the function or arm the line sits in; name it by the
        // nearest identifier before the use.
        found.insert(line.to_string());
    }
    // One derivation per surviving answer, plus the ladder's own definition
    // and the resolver that copies it.
    assert!(
        found.len() <= 8,
        "{} places derive a colour from `bg_active`; four answers were agreed:\n  {}",
        found.len(),
        found.iter().cloned().collect::<Vec<_>>().join("\n  ")
    );
}

/// And the reasons are written where the answer is, not only here.
#[test]
fn each_answer_says_why_it_is_not_the_others() {
    let semantic = include_str!("../src/v2/theme/semantic/set.rs");
    assert!(
        semantic.contains("never the accent") || semantic.contains("the accent"),
        "the free-control answer says it is the accent"
    );
    assert!(
        !semantic.contains("fn rail_tab_active"),
        "the dead one is gone; its hover partner `rail_tab_hover` stays, because that one is called"
    );
    assert!(semantic.contains("fn rail_tab_hover"), "and the partner that IS called is still here");
    // The tab's answer is the one defined by what is behind it, so it is the
    // one most easily broken by lightening a strip — say so where it lives.
    let tab = semantic.split("fn tab_active_gradient").nth(1).unwrap_or("");
    assert!(tab.contains("Buts") || tab.contains("host_body_surface"), "the tab's answer reads the body");
}
