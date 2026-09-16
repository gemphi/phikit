//! Execute region — the filename field plus the Cancel / <Execute> pair.
//! Blender ref: `file_panels.cc:138` (`file_panel_execution_buttons_draw`).
//!
//! The execute button's label comes from the browser's purpose
//! (`FileSelectParams::title`) rather than being hardcoded to "Open", and the
//! button order follows the platform: Windows puts the affirmative action
//! first, everywhere else it comes last.

use crate::v2::part::custom::file::{keys, listing, purpose::purpose};
use crate::v2::state::{BState, key::K_MODAL, val::V_EMPTY};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BPanelRole};

fn filename(s: &BState) -> BView {
    let name = s
        .get(&keys::k_filename())
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .or_else(|| {
            s.get(&keys::k_selected())
                .map(|p| p.rsplit(['/', '\\']).next().unwrap_or(p).to_string())
        })
        .unwrap_or_default();
    BView::field(name)
}

/// Commit: resolve `<dir>/<filename>` and write it as the selection, which
/// the action reducer turns into an import + recent-list entry.
fn execute_btn(s: &BState) -> BView {
    let dir = listing::current_dir(s);
    let target = s
        .get(&keys::k_selected())
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            let name = s.get(&keys::k_filename()).unwrap_or("");
            if name.is_empty() {
                String::new()
            } else {
                listing::join(&dir, name)
            }
        });
    BView::button_action(
        "check",
        purpose(s).exec_label(),
        BAction::Set(keys::k_selected(), target),
    )
}

fn cancel_btn() -> BView {
    BView::button_action("x", "Cancel", BAction::Set(K_MODAL, V_EMPTY.to_string()))
}

pub fn view(s: &BState) -> BView {
    let mut children = vec![filename(s), BView::spacer()];
    if cfg!(windows) {
        children.push(execute_btn(s));
        children.push(cancel_btn());
    } else {
        children.push(cancel_btn());
        children.push(execute_btn(s));
    }
    BView::panel(BPanelRole::Footer, children)
}
