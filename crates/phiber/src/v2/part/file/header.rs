//! File / Asset browser header menus — shared by the modal and the hosts.
//!
//! Blender ref (`space_filebrowser.py`):
//!   `FILEBROWSER_MT_editor_menus`  → View, Select                 (L486)
//!   `ASSETBROWSER_MT_editor_menus` → View, Select, Library, Catalog (L664)
//! Items are Blender's, in Blender's order.

use crate::v2::state::{
    BKey as _, BState, MenuKey,
    val::{V_EMPTY, V_TRUE},
};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BCard, BControl, BPopoverSide, BStrip};

/// `SpaceFile.browse_mode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BBrowseMode {
    Files,
    Assets,
}

/// Menus are host-scoped; the modal browser uses the `file` host slug so it
/// shares the File editor's namespace and cannot collide with any other
/// editor's View menu on screen.
fn host_slug(mode: BBrowseMode) -> &'static str {
    match mode {
        BBrowseMode::Files => "file",
        BBrowseMode::Assets => "asset",
    }
}

fn menu(s: &BState, mode: BBrowseMode, label: &'static str, items: &[(&'static str, &'static str)]) -> BView {
    let key = MenuKey::Top(host_slug(mode), label).path();
    let open = s.get(&key) == Some(V_TRUE);
    let anchor =
        BView::Button(BControl::new().with_label(label).with_action(BAction::Toggle(key.clone())));
    let body = items
        .iter()
        .map(|(ic, text)| {
            if text.is_empty() {
                BView::menu_sep()
            } else {
                BView::MenuItem(
                    BControl::new()
                        .with_icon(*ic)
                        .with_label(*text)
                        .with_action(BAction::Set(key.clone(), V_EMPTY.to_string())),
                )
            }
        })
        .collect();
    BView::popover(anchor, body, open, BAction::Set(key, V_EMPTY.to_string()), BPopoverSide::Below)
}

const SEP: (&str, &str) = ("", "");

/// `FILEBROWSER_MT_view` (L500).
const FILE_VIEW: &[(&str, &str)] = &[
    ("check_square", "Source List"),
    ("check_square", "File Path"),
    ("zoom_in", "Frame Selected"),
    SEP,
    ("preferences", "Display Size"),
    ("file_folder", "Recursion Level"),
    SEP,
    ("panel", "Area"),
];
/// `ASSETBROWSER_MT_view` (L676).
const ASSET_VIEW: &[(&str, &str)] = &[
    ("check_square", "Source List"),
    ("check_square", "Asset Details"),
    ("zoom_in", "Frame Selected"),
    SEP,
    ("preferences", "Display Size"),
    SEP,
    ("panel", "Area"),
];
/// `FILEBROWSER_MT_select` / `ASSETBROWSER_MT_select` (L522, L697).
const SELECT: &[(&str, &str)] = &[
    ("selected", "All"),
    ("selected", "None"),
    ("select", "Invert"),
    SEP,
    ("select_box", "Box Select"),
];
/// `ASSETBROWSER_MT_library` (L712).
const LIBRARY: &[(&str, &str)] = &[
    ("refresh", "Refresh"),
    ("refresh", "Refresh Remote Listing"),
];
/// `ASSETBROWSER_MT_catalog` (L722).
const CATALOG: &[(&str, &str)] = &[
    ("loop_back", "Undo"),
    ("loop_forwards", "Redo"),
    SEP,
    ("file_tick", "Save Catalogs"),
    ("plus", "New Catalog"),
];

pub fn view_mode(s: &BState, mode: BBrowseMode) -> BView {
    let mut menus = match mode {
        BBrowseMode::Files => vec![menu(s, mode, "View", FILE_VIEW), menu(s, mode, "Select", SELECT)],
        BBrowseMode::Assets => vec![
            menu(s, mode, "View", ASSET_VIEW),
            menu(s, mode, "Select", SELECT),
            menu(s, mode, "Library", LIBRARY),
            menu(s, mode, "Catalog", CATALOG),
        ],
    };
    menus.insert(0, BView::icon(match mode {
        BBrowseMode::Files => "folder_open",
        BBrowseMode::Assets => "asset",
    }));
    BView::from(BCard::strip(BStrip::Header, menus))
}

/// Files-mode header (the modal dialog).
pub fn view(s: &BState) -> BView {
    view_mode(s, BBrowseMode::Files)
}
