//! File-browser header — Blender's `FILEBROWSER_HT_header`, which carries
//! only the View and Select menus (`space_filebrowser.py:16-77`). Everything
//! navigational lives one strip below, in `path_bar`.

use crate::v2::state::{
    BState, k_menu,
    val::{V_EMPTY, V_TRUE},
};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BPanelRole, BPopoverSide};

/// Menus are host-scoped like every other menubar; the modal browser uses the
/// `file` host slug so it shares the File editor's namespace and cannot
/// collide with the View menu of any other editor on screen.
const HOST: &str = "file";

fn menu(s: &BState, label: &'static str, items: &[(&'static str, &'static str)]) -> BView {
    let key = k_menu(HOST, label);
    let open = s.get(&key) == Some(V_TRUE);
    let anchor = BView::button_action("", label, BAction::Toggle(key.clone()));
    let body = items
        .iter()
        .map(|(ic, text)| {
            BView::menu_item(
                *ic,
                *text,
                "",
                BAction::Set(key.clone(), V_EMPTY.to_string()),
            )
        })
        .collect();
    BView::popover(
        anchor,
        body,
        open,
        BAction::Set(key, V_EMPTY.to_string()),
        BPopoverSide::Below,
    )
}

pub fn view(s: &BState) -> BView {
    BView::panel(
        BPanelRole::Header,
        vec![
            menu(
                s,
                "View",
                &[
                    ("zoom_in", "Zoom In"),
                    ("zoom_out", "Zoom Out"),
                    ("eye", "Show Hidden Files"),
                    ("refresh", "Refresh"),
                ],
            ),
            menu(
                s,
                "Select",
                &[
                    ("select_box", "Box Select"),
                    ("check", "Select All"),
                    ("x", "Deselect All"),
                    ("shuffle", "Invert Selection"),
                ],
            ),
        ],
    )
}
