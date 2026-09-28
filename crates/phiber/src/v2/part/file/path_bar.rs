//! The path bar — shared by the modal dialog and the File / Asset hosts.
//! Blender's `RGN_TYPE_UI` region, panel
//! `FILEBROWSER_PT_directory_path` (`space_filebrowser.py:414`).
//!
//! This strip is NOT the header. Blender's file-browser header carries only
//! the View and Select menus; the navigation buttons, the directory field,
//! the search field, the display-mode radio and the filter toggle live in a
//! dynamic-height UI region directly beneath it. Phiber had them in the
//! header, which is why the strip overflowed and the menus had nowhere to go.
//!
//! Left → right: Back · Forward · Up · Refresh │ New Folder │ directory │
//! search │ display modes │ filter toggle.

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use super::set::DISPLAY_MODES;
use super::set::FILTERS;
use super::listing;
use crate::v2::view::prim::BPopoverSide;
use crate::v2::state::{
    BState,
    val::{V_FALSE, V_TRUE},
};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};
use crate::v2::view::BViewItem;

fn directory_field(s: &BState) -> BView {
    // Typing a path and pressing Return navigates there, as Blender's
    // directory field does.
    let key = FileKey::Path.path();
    let editing = crate::v2::state::edit::is_editing(s, &key);
    let shown = if editing {
        s.get(&key).unwrap_or("").to_string()
    } else {
        listing::current_dir(s)
    };
    BView::text_field(shown.clone(), shown, editing, BAction::Set(key, String::new()))
}

fn display_modes(s: &BState) -> BView {
    let active = s.get(&FileKey::Display.path()).unwrap_or(DISPLAY_MODES[0].label);
    BView::row(
        DISPLAY_MODES
            .iter()
            .map(|v| {
                // A header toggle is a button, as the switcher beside it is
                // — one height, one shape — lit when it is the mode in force.
                BView::Button(
                    BControl::new()
                        .with_icon(v.icon)
                        .with_active(v.label == active)
                        .with_action(BAction::Set(FileKey::Display.path(), v.label.to_string())),
                )
            })
            .collect(),
    )
}

/// The master filter switch — `FILTER` / `FILTER_FILLED` in Blender. Rendered
/// as a tab so the "on" state gets the same lit cap as every other toggle.
fn filter_toggle(s: &BState) -> BView {
    let on = listing::use_filter(s);
    let next = if on { V_FALSE } else { V_TRUE };
    BView::Button(
        BControl::new()
            .with_icon("filter")
            .with_active(on)
            .with_action(BAction::Set(FileKey::UseFilter.path(), next.to_string())),
    )
}

/// `FILE_HIDE_DOT`, inverted — the eye is open when hidden files are shown.
fn hidden_toggle(s: &BState) -> BView {
    let on = listing::show_hidden(s);
    let next = if on { V_FALSE } else { V_TRUE };
    BView::Button(
        BControl::new()
            .with_icon(if on { "eye" } else { "eye_off" })
            .with_action(BAction::Set(FileKey::ShowHidden.path(), next.to_string())),
    )
}

// ── Filter popover ─────────────────────────────────────────────────────────

/// The type-filter popover body: hidden-file toggle, separator, 10 type flags.
fn filter_body(s: &BState) -> Vec<BView> {
    let hidden_on = listing::show_hidden(s);
    // Rows of a menu, not buttons: a menu item leads with its check and
    // runs its label from the left edge like every other popover row.
    let mut body = vec![
        BView::MenuItem(
            BControl::new()
                .with_icon(if hidden_on { "check_square" } else { "square" })
                .with_label("Show Hidden Files")
                .with_active(hidden_on)
                .with_action(BAction::Toggle(FileKey::ShowHidden.path())),
        ),
        BView::menu_sep(),
    ];
    for f in FILTERS {
        let fkey = FileKey::Filter(f.key).path();
        let active = s.get(&fkey) != Some(V_FALSE);
        body.push(BView::MenuItem(
            BControl::new()
                .with_icon(f.icon)
                .with_label(f.label)
                .with_active(active)
                .with_action(BAction::Toggle(fkey)),
        ));
    }
    body
}

/// The filter-settings popover (`FILEBROWSER_PT_filter`, a HEADER-region
/// panel Blender opens from the arrow beside the filter toggle).
fn filter_popover(s: &BState) -> BView {
    let fkey = FileKey::FilterOpen.path();
    let open = s.get(&fkey) == Some(V_TRUE);
    let toggle = BAction::Set(fkey.clone(), if open { V_FALSE } else { V_TRUE }.to_string());
    let close = BAction::Set(fkey, V_FALSE.to_string());
    let anchor = BView::Button(BControl::new().with_icon("chevron_down").with_action(toggle));
    BView::popover(anchor, filter_body(s), open, close, BPopoverSide::Below)
}

/// The strip's row, in Blender's order (`FILEBROWSER_PT_directory_path`):
/// nav, directory, search. It is the `ui` region's content — the region is
/// the strip, so this is a row and paints nothing itself. The search field
/// lives here rather than in the header's middle, as Blender's
/// `filter_search` does (`host.searches_in_tool_header()`).
/// The directory and the search — the strip's centre, starting where the
/// file list does; the folder moves are the column before it (`nav`).
pub fn view(host: crate::v2::host::BHostType, s: &BState) -> BView {
    BView::row(vec![directory_field(s), crate::v2::part::search::BSearch::new(host).build(s)])
}

/// The header's right-hand icons: the display-mode radio. The `Filter` part
/// of the File and Asset hosts.
pub fn header_icons(s: &BState) -> BView {
    display_modes(s)
}

/// The path strip's right-hand icons: the hidden-files eye, the filter
/// toggle and its settings popover (`FILEBROWSER_PT_filter`). The
/// `TypeFilter` part, in the tool header's Right pane.
pub fn type_filter(s: &BState) -> BView {
    BView::row(vec![hidden_toggle(s), filter_toggle(s), filter_popover(s)])
}
