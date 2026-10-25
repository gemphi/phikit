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
//! Left → right: Back · Forward · Up · Refresh · New Folder │ directory │
//! search │ display modes │ filter toggle ▾ │ gear.

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

/// The directory: breadcrumbs, each folder a link (`listing::trail`); a
/// click between them, and the path is typed — Return goes there, as
/// Blender's directory field does.
fn directory_field(s: &BState) -> BView {
    let key = FileKey::Path.path();
    if crate::v2::state::edit::is_editing(s, &key) {
        let shown = s.get(&key).unwrap_or("").to_string();
        return BView::text_field(shown.clone(), shown, true, BAction::Set(key, String::new()));
    }
    let here = listing::current_dir(s);
    let trail = listing::trail(&here).into_iter().map(|(name, path)| (name, BAction::Set(key.clone(), path))).collect();
    // Typing starts from the folder shown, even before one was ever chosen.
    let edit = vec![BAction::Set(key.clone(), here), BAction::Set(crate::v2::state::AppKey::Edit.path(), key.encode())];
    BView::Crumbs { trail, edit }
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
    // The card size (View ▸ Thumbnail Size): a contact sheet to a gallery.
    let size = super::thumb::size(s);
    for (label, px) in super::thumb::SIZES {
        body.push(BView::MenuItem(
            BControl::new()
                .with_icon(if px == size { "check_circle" } else { "circle" })
                .with_label(label)
                .with_active(px == size)
                .with_action(BAction::Set(FileKey::ThumbSize.path(), label.to_string())),
        ));
    }
    body.push(BView::menu_sep());
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
    let anchor = BView::Button(BControl::new().with_icon("chevron_down").with_active(open).with_action(toggle));
    // The chevron sits at the right end of the path strip (its Right pane),
    // so the panel opens trailing-edge and spans back across the strip.
    BView::popover(anchor, filter_body(s), open, close, BPopoverSide::BelowEnd)
}

/// The directory, over the file list (`host/file`): the header's middle,
/// starting where the list does. The search is the header's end, over the
/// details column (`region::complete_header`).
pub struct BPathBar;

impl BPathBar {
    pub const fn new(_host: crate::v2::host::BHostType) -> Self {
        Self
    }
}

impl BViewItem for BPathBar {
    fn build(&self, s: &BState) -> BView {
        directory_field(s)
    }
}

/// The path strip's right-hand icons, Blender's order: the display-mode
/// radio, the filter toggle and its settings popover
/// (`FILEBROWSER_PT_filter`, where the hidden-files box lives), and the
/// gear. The `TypeFilter` part, in the tool header's Right pane.
pub struct BTypeFilter;

impl BTypeFilter {
    pub const fn new(_host: crate::v2::host::BHostType) -> Self {
        Self
    }
}

impl BViewItem for BTypeFilter {
    fn build(&self, s: &BState) -> BView {
        // Spread over the details column: the modes, the filters, the gear.
        BView::row(vec![display_modes(s), BView::spacer(), filter_toggle(s), filter_popover(s), BView::spacer(), sidebar_toggle(s)])
    }
}

/// The gear at the strip's end: shows or hides the side column with the
/// operator's options, as Blender's does its options region.
fn sidebar_toggle(s: &BState) -> BView {
    let hidden = crate::v2::view::size::sidebar_hidden(s);
    BView::Button(
        BControl::new()
            .with_icon("settings")
            .with_tooltip("Toggle Region")
            .with_active(!hidden)
            .with_action(BAction::Set(crate::v2::view::size::k_sidebar_hidden(), (!hidden).to_string())),
    )
}
