//! The path bar — Blender's `RGN_TYPE_UI` region, panel
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

use crate::v2::part::custom::file::set::{DISPLAY_MODES, NAV};
use crate::v2::part::custom::file::{history, keys, listing};
use crate::v2::state::file_nav::{NAV_NEXT, NAV_PARENT, NAV_PREVIOUS};
use crate::v2::state::{
    BState,
    val::{V_FALSE, V_TRUE},
};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BPanelRole};

fn nav_group(s: &BState) -> BView {
    let mut out = Vec::with_capacity(NAV.len());
    for n in NAV {
        // Back / Forward render dimmed when their stack is empty, matching
        // Blender's greyed-out nav buttons.
        let enabled = match n.action {
            NAV_PREVIOUS => history::can_back(s),
            NAV_NEXT => history::can_forward(s),
            NAV_PARENT => listing::parent_of(&listing::current_dir(s)).is_some(),
            _ => true,
        };
        out.push(if enabled {
            BView::button_action(
                n.icon,
                "",
                BAction::Set(keys::k_nav(), n.action.to_string()),
            )
        } else {
            BView::button(n.icon)
        });
    }
    BView::row(out)
}

fn directory_field(s: &BState) -> BView {
    BView::field(listing::current_dir(s))
}

fn search_field(s: &BState) -> BView {
    let q = s.get(&keys::k_search()).unwrap_or("");
    BView::row(vec![
        BView::icon("viewzoom"),
        BView::field(if q.is_empty() {
            "Search".to_string()
        } else {
            q.to_string()
        }),
    ])
}

fn display_modes(s: &BState) -> BView {
    let active = s.get(&keys::k_display()).unwrap_or(DISPLAY_MODES[0].label);
    BView::row(
        DISPLAY_MODES
            .iter()
            .map(|v| {
                BView::tab(
                    v.icon,
                    v.label == active,
                    BAction::Set(keys::k_display(), v.label.to_string()),
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
    BView::tab(
        "filter",
        on,
        BAction::Set(keys::k_use_filter(), next.to_string()),
    )
}

/// `FILE_HIDE_DOT`, inverted — the eye is open when hidden files are shown.
fn hidden_toggle(s: &BState) -> BView {
    let on = listing::show_hidden(s);
    let next = if on { V_FALSE } else { V_TRUE };
    BView::button_action(
        if on { "eye" } else { "eye_off" },
        "",
        BAction::Set(keys::k_show_hidden(), next.to_string()),
    )
}

pub fn view(s: &BState) -> BView {
    BView::panel(
        BPanelRole::Toolbar,
        vec![BView::row(vec![
            nav_group(s),
            directory_field(s),
            search_field(s),
            BView::spacer(),
            display_modes(s),
            hidden_toggle(s),
            filter_toggle(s),
        ])],
    )
}
