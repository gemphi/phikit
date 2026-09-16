//! Left sidebar — Blender's `RGN_TYPE_TOOLS` "Bookmarks" category, with the
//! four `FSMenuCategory` panels in Blender's draw order.
//! Blender ref: `space_filebrowser.py:221-358`, `fsmenu.cc`.
//!
//! Volumes and System are enumerated at runtime and filtered to paths that
//! actually exist, so the sidebar never offers a dead link. Sections are real
//! accordions keyed on `ctx.file.section.<name>`.

use crate::v2::modal::splash::recent;
use crate::v2::part::custom::file::{
    fsmenu::{self, BFsEntry},
    keys,
};
use crate::v2::state::{
    BPath, BState,
    val::{V_FALSE, V_TRUE},
};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BPanelRole};

fn k_section(name: &'static str) -> BPath {
    BPath::owned(vec!["file", "section", name])
}

fn expanded(s: &BState, name: &'static str) -> bool {
    s.get(&k_section(name)) != Some(V_FALSE)
}

fn header(s: &BState, title: &'static str) -> BView {
    let open = expanded(s, title);
    let next = if open { V_FALSE } else { V_TRUE };
    BView::panel(
        BPanelRole::CardHeader,
        vec![BView::button_action(
            if open {
                "chevron_down"
            } else {
                "chevron_right"
            },
            title,
            BAction::Set(k_section(title), next.to_string()),
        )],
    )
}

fn entry(e: &BFsEntry) -> BView {
    BView::button_action(
        e.icon,
        e.label.clone(),
        BAction::Set(keys::k_path(), e.path.clone()),
    )
}

fn section(s: &BState, title: &'static str, entries: Vec<BFsEntry>) -> BView {
    let mut children = vec![header(s, title)];
    if expanded(s, title) {
        if entries.is_empty() {
            children.push(BView::label("Nothing here yet"));
        } else {
            children.extend(entries.iter().map(entry));
        }
    }
    BView::panel(BPanelRole::Card, children)
}

/// "Recent" reuses the welcome window's most-recently-used list, so opening a
/// project from either place keeps both in step.
fn recent_entries(s: &BState) -> Vec<BFsEntry> {
    recent::recent_paths(s)
        .into_iter()
        .map(|p| BFsEntry {
            icon: "history",
            label: recent::display_name(&p).to_string(),
            path: recent::parent_dir(&p).to_string(),
        })
        .collect()
}

pub fn view(s: &BState) -> BView {
    BView::panel(
        BPanelRole::Sidebar,
        vec![
            section(s, "Volumes", fsmenu::volumes()),
            section(s, "System", fsmenu::system_bookmarks()),
            section(s, "Recent", recent_entries(s)),
        ],
    )
}
