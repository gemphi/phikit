//! The main menu's submenus, given what the top-level menus already had.
//!
//! A submenu's PANEL was never the problem: `BPopover::render` builds the
//! surface, elevation, radius, padding, gap, scroll and separators from the
//! same `BPopoverPart` whether the popover opens below a menubar label or
//! beside a row, and only `place::BPlacement` branches on the side. What had
//! not followed were the four things the ROW says:
//!
//! - the row that opened the submenu was never marked active, so it was drawn
//!   exactly like the idle rows around it and nothing said which row the panel
//!   beside it belonged to;
//! - the arrow was the string `"▶"` handed in as the row's `shortcut`, so it
//!   was drawn in the shortcut's quieter caption type AND spent the one column
//!   a row has for a key binding;
//! - clicking the row that opened a submenu could not close it, though
//!   clicking a menubar label closes its menu;
//! - a checked entry showed a 12px glyph and nothing else, where the same
//!   ticked row in a picker or the file browser's filter takes the fill too.

use phiber::v2::part::menubar::item::{BMenuItem, BMenuKind};
use phiber::v2::host::BHostType;
use phiber::v2::part::menubar::BMenuSpec;
use phiber::v2::part::menubar::tree::item_view;
use phiber::v2::state::{BKey as _, BState, MenuKey, OpKey, val::*};
use phiber::v2::view::BView;
use phiber::v2::view::prim::{BAction, BControl};

/// The `Sub` item's anchor row, with `labels` above it, in state `s`.
fn anchor(s: &BState, labels: &[&'static str], it: &BMenuItem) -> BControl {
    match item_view(BHostType::Topbar.slug(), labels, it, s) {
        BView::Popover { anchor, .. } => match *anchor {
            BView::MenuItem(ctl) => ctl,
            other => panic!("a submenu's anchor is a menu row, not {other:?}"),
        },
        other => panic!("a submenu is a popover, not {other:?}"),
    }
}

/// Every `Sub`, `Live`, `Recent` and `History` item anywhere in the topbar,
/// with the chain of labels above it.
fn every_submenu_item() -> Vec<(Vec<&'static str>, BMenuItem)> {
    fn walk(labels: Vec<&'static str>, items: &'static [BMenuItem], out: &mut Vec<(Vec<&'static str>, BMenuItem)>) {
        for it in items {
            if matches!(it.kind, BMenuKind::Sub(_) | BMenuKind::Live(_) | BMenuKind::Recent | BMenuKind::History) {
                out.push((labels.clone(), *it));
            }
            if let BMenuKind::Sub(subs) = it.kind {
                let mut deeper = labels.clone();
                deeper.push(it.label);
                walk(deeper, subs, out);
            }
        }
    }
    let mut out = Vec::new();
    for spec in BMenuSpec::list_for(BHostType::Topbar) {
        walk(vec![spec.label], spec.items, &mut out);
    }
    out
}

#[test]
fn the_topbar_has_submenus_to_test() {
    let all = every_submenu_item();
    assert!(all.len() >= 8, "the topbar's own submenus: {}", all.len());
    // The top bar's own submenus are all one deep — File \u{25b8} Import, not
    // File \u{25b8} Import \u{25b8} something. The labels chain still has to be
    // right for the keys to be, so every one of them carries its menu's label.
    assert!(all.iter().all(|(labels, _)| labels.len() == 1), "every topbar submenu hangs off one menu");
    let names: Vec<&str> = all.iter().map(|(_, it)| it.label).collect();
    assert!(names.contains(&"Import") && names.contains(&"Export"), "the ones you would name first: {names:?}");
}

#[test]
fn the_row_that_opened_a_submenu_is_marked_active() {
    for (labels, it) in every_submenu_item() {
        let mut chain: Vec<&'static str> = labels.clone();
        chain.push(it.label);
        let key = MenuKey::Nested(BHostType::Topbar.slug(), &chain).path();

        let shut = BState::new();
        assert!(!anchor(&shut, &labels, &it).active, "{chain:?}: idle while its submenu is shut");

        let mut open = BState::new();
        open.set(&key, V_TRUE);
        assert!(
            anchor(&open, &labels, &it).active,
            "{chain:?}: the row reads as open while its submenu is, as a menubar label does"
        );
    }
}

#[test]
fn the_arrow_is_its_own_column_and_leaves_the_shortcut_free() {
    let s = BState::new();
    for (labels, it) in every_submenu_item() {
        let ctl = anchor(&s, &labels, &it);
        assert!(ctl.submenu, "{:?}: says it leads to a menu", it.label);
        assert_ne!(
            ctl.shortcut.as_deref(),
            Some("▶"),
            "{:?}: the arrow is not a shortcut any more",
            it.label
        );
    }
    // And the column it vacated is usable: a submenu that names a shortcut
    // keeps it, which the arrow-as-shortcut spelling made impossible.
    let keyed = BMenuItem { label: "Recover", shortcut: "Ctrl Shift O", kind: BMenuKind::Sub(&[]), modal: "", purpose: "", url: "" };
    let ctl = anchor(&s, &["File"], &keyed);
    assert_eq!(ctl.shortcut.as_deref(), Some("Ctrl Shift O"), "a submenu can carry a key binding");
    assert!(ctl.submenu, "and still show its arrow");
}

#[test]
fn the_arrow_is_drawn_as_an_icon_in_a_column_of_its_own() {
    let menu = include_str!("../src/v2/view/draw/menu.rs");
    assert!(menu.contains("Submenu"), "the row has a submenu part");
    assert!(menu.contains("glyph(\"chevron_right\", ctx)"), "drawn as an icon, not as text");
    assert!(menu.contains("fn glyph("), "through the one glyph builder a menu row has");
    assert!(
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/chevron_right.svg")).exists(),
        "and the icon is in the set"
    );
}

#[test]
fn clicking_the_row_that_opened_a_submenu_closes_it_again() {
    use phiber::v2::state::action::apply_action;
    for (labels, it) in every_submenu_item() {
        let mut chain: Vec<&'static str> = labels.clone();
        chain.push(it.label);
        let key = MenuKey::Nested(BHostType::Topbar.slug(), &chain).path();
        let top = MenuKey::Top(BHostType::Topbar.slug(), labels[0]).path();

        let mut s = BState::new();
        s.set(&top, V_TRUE);
        let click = BAction::Toggle(key.clone());
        apply_action(&mut s, &click);
        assert_eq!(s.get(&key), Some(V_TRUE), "{chain:?}: the first click opens it");
        apply_action(&mut s, &click);
        assert_ne!(s.get(&key), Some(V_TRUE), "{chain:?}: the second closes it");
        // And the menu it lives in is still open — closing a submenu is not
        // closing the menu.
        assert_eq!(s.get(&top), Some(V_TRUE), "{chain:?}: its parent menu stays down");
    }
}

#[test]
fn hovering_an_open_submenu_leaves_it_open() {
    use phiber::v2::state::action::apply_action;
    // A click is a toggle; a hover only opens. The two arrive as different
    // actions for exactly this reason — crossing the row on the way to its
    // panel must not close the panel.
    let (labels, it) = every_submenu_item().into_iter().next().expect("a submenu");
    let mut chain: Vec<&'static str> = labels.clone();
    chain.push(it.label);
    let key = MenuKey::Nested(BHostType::Topbar.slug(), &chain).path();
    let mut s = BState::new();
    s.set(&key, V_TRUE);
    apply_action(&mut s, &BAction::Open(key.clone()));
    assert_eq!(s.get(&key), Some(V_TRUE), "hover does not close what is open");
}

#[test]
fn a_checked_entry_takes_the_fill_as_well_as_the_check() {
    // The topbar's own checkables, whatever they happen to be.
    let mut found = 0;
    for spec in BMenuSpec::list_for(BHostType::Topbar) {
        for it in spec.items {
            let BMenuKind::Toggle(key, on_by_default) = it.kind else { continue };
            found += 1;
            let mut on = BState::new();
            on.set(&key(), if on_by_default { V_TRUE } else { V_TRUE });
            let mut off = BState::new();
            off.set(&key(), V_FALSE);
            for (s, want, what) in [(&on, true, "checked"), (&off, false, "unchecked")] {
                let BView::MenuItem(ctl) = item_view(BHostType::Topbar.slug(), &[spec.label], it, s) else {
                    panic!("a checkable is a menu row");
                };
                assert_eq!(ctl.active, want, "{:?}: {what} reads as {want}", it.label);
                assert_eq!(ctl.icon.as_deref() == Some("check"), want, "{:?}: and the glyph agrees", it.label);
            }
        }
    }
    assert!(found > 0, "the topbar has checkable entries");
}

#[test]
fn a_submenu_anchor_still_reports_nothing_silently() {
    // The arrow moving out of the shortcut column must not have cost the row
    // its action: every menu row answers a click (`menus_answer_every_click`).
    let s = BState::new();
    for (labels, it) in every_submenu_item() {
        let ctl = anchor(&s, &labels, &it);
        assert!(ctl.action.is_some(), "{:?}: answers a click", it.label);
        assert!(ctl.hover.is_some(), "{:?}: and opens on hover", it.label);
    }
    // And a leaf that names nothing still reports itself rather than going
    // quiet, which is the rule the arrow change sits next to.
    let bare = BMenuItem { label: "Nothing Yet", shortcut: "", kind: BMenuKind::Op, modal: "", purpose: "", url: "" };
    let BView::MenuItem(ctl) = item_view(BHostType::Topbar.slug(), &["File"], &bare, &s) else { panic!("a leaf is a menu row") };
    assert!(
        matches!(ctl.action, Some(BAction::Set(ref p, _)) if p == &OpKey::Report.path()),
        "a leaf with no target reports itself"
    );
}
