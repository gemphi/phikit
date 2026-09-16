//! `BFileItem` — the File / Asset host body.
//!
//! Layout (Blender `space_file.cc`):
//! ```text
//! ┌─ sidebar ──────┬─ nav_row ────────────────────────────────┐
//! │ Volumes        │ [← → ↑ ⟳]  /path/  [search]  [≡][⊞][⊠] [⊤]
//! │ System         ├─ table::header ──────────────────────────┤
//! │ Bookmarks      │  Name          Modified Date        Size  │
//! │ Recent         ├──────────────────────────────────────────┤
//! └────────────────┤  … file rows …                           │
//!                  ├─ execute_row (only in Open/SaveAs mode) ──┤
//!                  │  [filename search field]  [Open] [Cancel] │
//!                  └──────────────────────────────────────────┘
//! ```

use super::{
    fsmenu,
    history,
    keys,
    listing,
    purpose,
    set::{BFileCategory, CATEGORIES, DISPLAY_MODES, FILTERS, NAV},
    table,
};
use crate::v2::state::{
    BPath, BState,
    val::{V_FALSE, V_TRUE},
};
use crate::v2::host::BHostType;
use crate::v2::view::{BAction, BPanelRole, BPopoverSide, BView, BViewItem};

// ── Sidebar ────────────────────────────────────────────────────────────────

/// Build the left sidebar — Volumes, System, Bookmarks, Recent.
/// Mirrors Blender's `FILEBROWSER_PT_bookmarks_*` panels.
fn sidebar(state: &BState) -> BView {
    let mut sections: Vec<BView> = Vec::new();

    for cat in CATEGORIES {
        // cat.label is &'static str from a const table, so BPath::of is safe.
        let section_key = BPath::of(&["file", "sidebar", cat.label, "expanded"]);
        let expanded = state.get(&section_key) != Some(V_FALSE); // default open
        let toggle = BAction::Toggle(section_key);

        sections.push(BView::accordion_header(cat.label, expanded, toggle));

        if expanded {
            let entries: Vec<BView> = match cat.kind {
                BFileCategory::System => fsmenu::volumes()
                    .into_iter()
                    .map(|e| BView::button_action(
                        e.icon,
                        &e.label,
                        BAction::Set(keys::k_path(), e.path),
                    ))
                    .collect(),
                BFileCategory::SystemBookmarks => fsmenu::system_bookmarks()
                    .into_iter()
                    .map(|e| BView::button_action(
                        e.icon,
                        &e.label,
                        BAction::Set(keys::k_path(), e.path),
                    ))
                    .collect(),
                // Bookmarks and Recent are state-backed; placeholder for now.
                BFileCategory::Bookmarks | BFileCategory::Recent => Vec::new(),
            };
            sections.extend(entries);
        }
    }

    BView::panel(BPanelRole::Sidebar, sections)
}

// ── Filter popover ─────────────────────────────────────────────────────────

/// The type-filter popover body: hidden-file toggle, separator, 10 type flags.
fn filter_body(state: &BState) -> Vec<BView> {
    let hidden_on = listing::show_hidden(state);
    let mut body = vec![
        BView::button_action(
            if hidden_on { "check" } else { "" },
            "Show Hidden Files",
            BAction::Toggle(keys::k_show_hidden()),
        ),
        BView::menu_sep(),
    ];
    for f in FILTERS {
        let fkey = BPath::owned(vec!["file", f.key]);
        let active = state.get(&fkey) != Some(V_FALSE);
        body.push(BView::button_action(
            if active { f.icon } else { "" },
            f.label,
            BAction::Toggle(fkey),
        ));
    }
    body
}

// ── Execute bar ────────────────────────────────────────────────────────────

/// Footer bar shown only when the browser is in a named-file mode (Open,
/// Save As, Import, Export). Mirrors Blender's `FILEBROWSER_PT_execute`.
fn execute_row(state: &BState) -> BView {
    let p = purpose::purpose(state);
    let filename = state.get(&keys::k_filename()).unwrap_or("").to_string();
    let open_action = BAction::Set(keys::k_selected(), filename.clone());
    let cancel_action = BAction::Reset(keys::k_path());

    BView::panel(
        BPanelRole::Footer,
        vec![BView::row(vec![
            BView::search_field(
                "File name",
                &filename,
                BAction::Set(keys::k_filename(), String::new()),
            ),
            BView::button_action("", p.exec_label(), open_action),
            BView::button_action("", "Cancel", cancel_action),
        ])],
    )
}

// ── Nav toolbar ────────────────────────────────────────────────────────────

/// The top toolbar row: nav buttons (left), path + search (center, flex),
/// display-mode + filter (right). Mirrors Blender's file browser header.
fn nav_row(state: &BState) -> BView {
    use crate::v2::state::file_nav::{NAV_NEXT, NAV_PREVIOUS};

    // ── Left group: back / forward / parent / refresh ──────────────────────
    let nav_buttons: Vec<BView> = NAV
        .iter()
        .map(|nav| {
            let enabled = match nav.action {
                NAV_PREVIOUS => history::can_back(state),
                NAV_NEXT => history::can_forward(state),
                _ => true,
            };
            if enabled {
                BView::button_action(
                    nav.icon,
                    "",
                    BAction::Set(keys::k_nav(), nav.action.to_string()),
                )
            } else {
                BView::button(nav.icon)
            }
        })
        .collect();
    let left = BView::row(nav_buttons);

    // ── Center group: path field + search — both flex_1, fill remaining ────
    let search_val = state.get(&keys::k_search()).unwrap_or("").to_string();
    let center = BView::row(vec![
        BView::field(listing::current_dir(state)),
        BView::search_field(
            "Filter...",
            &search_val,
            BAction::Set(keys::k_search(), String::new()),
        ),
    ]);

    // ── Right group: display-mode toggles + filter popover ─────────────────
    let current_display = state.get(&keys::k_display()).unwrap_or("Vertical List");
    let mut right_cells: Vec<BView> = DISPLAY_MODES
        .iter()
        .map(|mode| {
            BView::button_action(
                mode.icon,
                "",
                BAction::Set(keys::k_display(), mode.label.to_string()),
            )
        })
        .collect();
    // Mark active display mode with a simple label above the icon (via
    // the button label slot) so the user can tell which one is selected.
    // For now just drop the active highlight — display state is readable.
    let _ = current_display; // suppress unused warning

    let fkey = keys::k_filter_open();
    let fopen = state.get(&fkey) == Some(V_TRUE);
    let ftoggle = BAction::Set(
        fkey.clone(),
        if fopen { V_FALSE.to_string() } else { V_TRUE.to_string() },
    );
    let fclose = BAction::Set(fkey, V_FALSE.to_string());
    let filter_anchor = BView::button_action("filter", "", ftoggle);
    right_cells.push(BView::popover(
        filter_anchor,
        filter_body(state),
        fopen,
        fclose,
        BPopoverSide::Below,
    ));
    let right = BView::row(right_cells);

    // Assemble as a Pane-style Left | Center (flex) | Right layout using a
    // single row where center has flex_1. We encode this by putting center
    // in a Column (which has flex_1) sandwiched between the two groups.
    BView::panel(
        BPanelRole::Toolbar,
        vec![BView::row(vec![
            left,
            BView::column(vec![center]),
            right,
        ])],
    )
}

// ── BFileItem ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct BFileItem {
    pub host: BHostType,
}

impl BFileItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BFileItem {
    fn build(&self, state: &BState) -> BView {
        // Decide whether to show the execute bar (filename + Open/Cancel).
        let show_execute = state.get(&keys::k_purpose()).is_some()
            && matches!(
                purpose::purpose(state),
                purpose::BFilePurpose::Open
                    | purpose::BFilePurpose::SaveAs
                    | purpose::BFilePurpose::Import
                    | purpose::BFilePurpose::Export
            );

        let main_col = BView::column(vec![
            nav_row(state),
            table::view(state),
        ]);

        let content = if show_execute {
            BView::column(vec![main_col, execute_row(state)])
        } else {
            main_col
        };

        // Two-column layout: fixed sidebar on the left, flex content on the
        // right. BSidebar has flex_shrink_0 + min_w so the content column
        // takes all remaining space via flex_1.
        BView::row(vec![sidebar(state), content])
    }
}
