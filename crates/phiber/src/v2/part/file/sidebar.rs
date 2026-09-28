//! File / Asset browser **Tools region** — the left column.
//!
//! One implementation for the modal dialog (Open / Save As / Import / Export)
//! and the `File` / `Asset` hosts, so headings, order and the accordion look
//! never drift apart again.
//!
//! Blender ref (`scripts/startup/bl_ui/space_filebrowser.py`):
//!   `FILEBROWSER_PT_bookmarks_volumes`   → "Volumes"   (L221)
//!   `FILEBROWSER_PT_bookmarks_system`    → "System"    (L244)
//!   `FILEBROWSER_PT_bookmarks_favorites` → "Bookmarks" (L282, `+`/`-` ops)
//!   `FILEBROWSER_PT_bookmarks_recents`   → "Recent"    (L332)
//! all in `bl_region_type = 'TOOLS'`, `bl_category = "Bookmarks"`, drawn as
//! collapsible panels — the same `BCard` the Properties editor's panels
//! are (`part::props::item`).
//!
//! In asset mode Blender's tools region is the asset-library catalog tree
//! (`ED_asset_catalog*`, `asset_catalog_tree_view`): a library selector on top
//! and a "Catalogs" panel with **All** and **Unassigned** plus the catalogs.

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use super::fsmenu::{self};
use super::fsmenu::BFsEntry;
use super::listing;
use super::set::BFileCategory;
use super::set::CATEGORIES;
use crate::v2::modal::splash::recent;
use crate::v2::state::{
    BPath, BState,
    val::{V_FALSE, V_TRUE},
};
use crate::v2::view::prim::{BAction, BCard, BColumn, BControl, with_peers};
use crate::v2::view::BView;

/// How many user bookmarks the sidebar holds (Blender has no fixed cap; ten
/// mirrors `RECENT_MAX` so both lists feel alike).
pub const BOOKMARK_MAX: usize = 10;
const BOOKMARK_SLOTS: [&str; BOOKMARK_MAX] =
    ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

/// `ctx.file.sidebar.<label>.expanded` — shared by modal and hosts.
fn k_expanded(label: &'static str) -> BPath {
    FileKey::SidebarExpanded(label).path()
}

/// `ctx.file.bookmark.<n>` — one absolute directory per slot.
pub fn k_bookmark(n: usize) -> BPath {
    FileKey::Bookmark(BOOKMARK_SLOTS[n.min(BOOKMARK_MAX - 1)]).path()
}

/// Every stored bookmark, in slot order, stopping at the first empty slot.
pub fn bookmarks(s: &BState) -> Vec<String> {
    (0..BOOKMARK_MAX)
        .map_while(|n| s.get(&k_bookmark(n)).filter(|p| !p.is_empty()).map(str::to_string))
        .collect()
}

/// Add the current directory as a bookmark (Blender `file.bookmark_add`).
/// No-op when it is already bookmarked or the slots are full.
pub fn add_bookmark(state: &mut BState, path: &str) {
    let existing = bookmarks(state);
    if path.is_empty() || existing.iter().any(|p| p == path) {
        return;
    }
    if existing.len() < BOOKMARK_MAX {
        state.set(&k_bookmark(existing.len()), path.to_string());
    }
}

fn expanded(s: &BState, label: &'static str) -> bool {
    // Blender opens every bookmarks panel by default.
    s.get(&k_expanded(label)) != Some(V_FALSE)
}

/// One folder of the source list. The folder the browser is in is lit, the
/// way the active tab of a rail is: the control fill, the parent's width.
fn entry_button(e: &BFsEntry, s: &BState) -> BView {
    let here = listing::current_dir(s) == e.path;
    BView::Button(
        BControl::new()
            .with_icon(e.icon)
            .with_label(e.label.clone())
            .with_active(here)
            .with_action(BAction::Set(FileKey::Path.path(), e.path.clone())),
    )
}

/// One collapsible panel (`FILEBROWSER_PT_bookmarks_*`).
pub(super) fn panel(s: &BState, label: &'static str, rows: Vec<BView>) -> BView {
    let body = if rows.is_empty() { vec![BView::label("Nothing here yet")] } else { rows };
    BCard::panel(label).fold(k_expanded(label), expanded(s, label)).body(body).into()
}

fn bookmark_rows(s: &BState) -> Vec<BView> {
    let mut rows: Vec<BView> = bookmarks(s)
        .into_iter()
        .map(|p| {
            entry_button(&BFsEntry {
                icon: "bookmark",
                label: recent::display_name(&p).to_string(),
                path: p,
            }, s)
        })
        .collect();
    // Blender's `+` (file.bookmark_add) beside the list.
    rows.push(BView::Button(
        BControl::new()
            .with_icon("plus")
            .with_label("Add Bookmark")
            .with_action(BAction::Set(
                FileKey::BookmarkAdd.path(),
                V_TRUE.to_string(),
            )),
    ));
    rows
}

fn recent_rows(s: &BState) -> Vec<BView> {
    recent::recent_paths(s)
        .into_iter()
        .map(|p| {
            entry_button(&BFsEntry {
                icon: "history",
                label: recent::display_name(&p).to_string(),
                path: recent::parent_dir(&p).to_string(),
            }, s)
        })
        .collect()
}

/// Files mode — the four `Bookmarks` panels in Blender's draw order.
pub fn view(s: &BState) -> BView {
    let mut panels: Vec<BView> = CATEGORIES
        .iter()
        .map(|cat| {
            let rows = match cat.kind {
                BFileCategory::System => fsmenu::volumes().iter().map(|e| entry_button(e, s)).collect(),
                BFileCategory::SystemBookmarks => {
                    fsmenu::system_bookmarks().iter().map(|e| entry_button(e, s)).collect()
                }
                BFileCategory::Bookmarks => bookmark_rows(s),
                BFileCategory::Recent => recent_rows(s),
            };
            panel(s, cat.label, rows)
        })
        .collect();
    panels.extend(super::option::panel(s));
    BView::from(BCard::column(BColumn::Sidebar, with_peers(panels)))
}

pub use super::asset_sidebar::LIBRARIES;
pub use super::asset_sidebar::asset_view;
pub use super::asset_sidebar::k_catalog;
pub use super::asset_sidebar::k_library;

/// The Tools region's content for either browser: the file browser's
/// bookmarks panels, the asset browser's libraries and catalogs.
pub fn source_list(host: crate::v2::host::BHostType, s: &BState) -> BView {
    if matches!(host, crate::v2::host::BHostType::Asset) { asset_view(s) } else { view(s) }
}
