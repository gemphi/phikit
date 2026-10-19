//! `item_view` — recursive `BMenuItem` → `BView` translation. `BMenuKind::Sub`
//! becomes a nested popover opening to the RIGHT of its parent row. Sub
//! anchors and top-level items carry a hover-Open action so hovering either
//! opens the target menu (with `apply_open`'s sibling-closing logic this
//! gives clean hover-switch between sibling submenus).

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use super::item::BMenuItem;
use super::item::BMenuKind;
use crate::v2::state::{BState, MenuKey, OpKey, val::*};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};

/// Resolve the action a leaf menu item fires. When the item declares a
/// `modal` slug (set-driven), it opens that modal; an operator fires its
/// intent; a checkable entry flips its key; and an item that names nothing
/// reports itself as not available yet (`menubar/op.rs`), so no entry is
/// ever silent. This reads the item's declarative intent — there is no
/// per-`(menu, label)` routing table.
pub fn leaf_action(host: &'static str, labels: &[&'static str], it: &BMenuItem) -> BAction {
    match it.kind {
        BMenuKind::Act(act) => return act(),
        // A `!` marks a key that is on by default (`menubar/op.rs`).
        BMenuKind::Toggle(key, on_by_default) => {
            return BAction::Set(OpKey::Flip.path(), format!("{}{}", if on_by_default { "!" } else { "" }, key().encode()));
        }
        // The showing editor's own key (a shared table): a host the slug does
        // not name — the palette walks every table as the top bar — reports
        // the entry instead of flipping a stranger's region.
        BMenuKind::HostToggle(key, on_by_default) => {
            if let Some(h) = crate::v2::host::BHostType::from_slug(host) {
                return BAction::Set(OpKey::Flip.path(), format!("{}{}", if on_by_default { "!" } else { "" }, key(h).encode()));
            }
        }
        // Ctrl Space carries the editor whose menu fired it, so the rule can
        // grow that editor's area (`area::maximize`) — a shared table cannot
        // write a host into a static value.
        BMenuKind::Fire(OpKey::AreaMaximize, _) => return BAction::Set(OpKey::AreaMaximize.path(), host.to_string()),
        // The showing editor's slug in the value: `node_view` with
        // `<host>:all`. A host the slug does not name (the palette) falls
        // through to the report, as `HostToggle` does.
        BMenuKind::HostFire(key, suffix) => {
            if let Some(h) = crate::v2::host::BHostType::from_slug(host) {
                return BAction::Set(key.path(), format!("{}{suffix}", h.slug()));
            }
        }
        // An export names its format and opens the browser for the path;
        // the format rides the purpose until the path is picked (`file::op`).
        BMenuKind::Fire(key, value) if it.purpose.is_empty() => return BAction::Set(key.path(), value.to_string()),
        BMenuKind::Fire(OpKey::ExportFormat, format) => {
            return BAction::Set(FileKey::Purpose.path(), format!("{}:{format}", it.purpose));
        }
        _ => {}
    }
    // A file-browser job: setting the purpose opens the browser (the reducer
    // raises the modal and titles it after the purpose).
    if !it.purpose.is_empty() {
        return BAction::Set(
            FileKey::Purpose.path(),
            it.purpose.to_string(),
        );
    }
    if !it.url.is_empty() {
        return BAction::Set(crate::v2::state::AppKey::Link.path(), it.url.to_string());
    }
    if !it.modal.is_empty() {
        return BAction::Set(crate::v2::state::ModalKey::Open.path(), it.modal.to_string());
    }
    BAction::Set(OpKey::Report.path(), item_path(labels, it.label))
}

/// `File ▸ Revert`: the item's place in its menu, as the status bar names it.
pub fn item_path(labels: &[&'static str], label: &str) -> String {
    let mut path = labels.join(" ▸ ");
    if !path.is_empty() {
        path.push_str(" ▸ ");
    }
    path.push_str(label);
    path
}

impl BMenuItem {
    /// A submenu listed live — the recent files, the undo steps, a `Live`
    /// list — as a popover to the right of its row, opened on hover.
    fn live_sub(&self, host: &'static str, labels: &[&'static str], s: &BState, body: Vec<BView>) -> BView {
        let mut sub_labels: Vec<&'static str> = labels.to_vec();
        sub_labels.push(self.label);
        let key = MenuKey::Nested(host, &sub_labels).path();
        let open = s.get(&key) == Some(V_TRUE);
        let anchor = BView::MenuItem(
            BControl::new()
                .with_label(self.label)
                .with_shortcut(self.shortcut)
                .with_submenu(true)
                .with_active(open)
                .with_action(BAction::Toggle(key.clone()))
                .with_hover(BAction::Open(key.clone())),
        );
        BView::popover(anchor, body, open, BAction::Set(key, V_EMPTY.to_string()), BPopoverSide::Right)
    }
}

/// `labels` is the chain of menu labels above `it`, from the menubar down.
pub fn item_view(host: &'static str, labels: &[&'static str], it: &BMenuItem, s: &BState) -> BView {
    match it.kind {
        BMenuKind::Sep => BView::menu_sep(),
        BMenuKind::Sub(subs) => {
            let mut sub_labels: Vec<&'static str> = labels.to_vec();
            sub_labels.push(it.label);
            let key = MenuKey::Nested(host, &sub_labels).path();
            let open = s.get(&key) == Some(V_TRUE);
            let toggle = BAction::Toggle(key.clone());
            let opener = BAction::Open(key.clone());
            let close = BAction::Set(key, V_EMPTY.to_string());
            let anchor = BView::MenuItem(
                BControl::new()
                    .with_label(it.label)
                    .with_shortcut(it.shortcut)
                    .with_submenu(true)
                    // The row that opened this submenu reads as open, the way
                    // a menubar label does while its menu is down. Without it
                    // the row you had just clicked was drawn exactly like the
                    // idle rows around it, so nothing on screen said which of
                    // them the panel beside it belonged to.
                    .with_active(open)
                    .with_action(toggle)
                    .with_hover(opener),
            );
            let body = subs.iter().map(|c| item_view(host, &sub_labels, c, s)).collect();
            BView::popover(anchor, body, open, close, BPopoverSide::Right)
        }
        BMenuKind::Live(list) => it.live_sub(host, labels, s, list(s)),
        BMenuKind::Recent => {
            let paths = crate::v2::modal::splash::recent::recent_paths(s);
            let body: Vec<BView> = if paths.is_empty() {
                vec![BView::MenuItem(BControl::new().with_label("No Recent Files"))]
            } else {
                paths
                    .iter()
                    .map(|path| {
                        BView::MenuItem(
                            BControl::new()
                                .with_icon("blender")
                                .with_label(crate::v2::modal::splash::recent::display_name(path).to_string())
                                .with_action(BAction::Set(FileKey::Selected.path(), path.clone())),
                        )
                    })
                    .collect()
            };
            it.live_sub(host, labels, s, body)
        }
        BMenuKind::History => {
            let listing = crate::v2::app::undo::listing();
            let body: Vec<BView> = listing
                .names
                .iter()
                .enumerate()
                .map(|(n, name)| {
                    BView::MenuItem(
                        BControl::new()
                            .with_icon(if n == listing.at { "check" } else { "" })
                            .with_label(name.clone())
                            .with_action(BAction::Set(OpKey::UndoHistory.path(), n.to_string())),
                    )
                })
                .collect();
            it.live_sub(host, labels, s, body)
        }
        // A checked entry takes the active fill as well as the check, which is
        // what a ticked row in a picker or the file browser's filter has
        // always done. The glyph alone made the menubar's checkables the one
        // family where "on" was a 12px mark and nothing else.
        BMenuKind::Toggle(key, on_by_default) => {
            let on = if on_by_default { s.get(&key()) != Some(V_FALSE) } else { s.get(&key()) == Some(V_TRUE) };
            BView::MenuItem(
                BControl::new()
                    .with_icon(if on { "check" } else { "" })
                    .with_label(it.label)
                    .with_shortcut(it.shortcut)
                    .with_active(on)
                    .with_action(leaf_action(host, labels, it)),
            )
        }
        // Checked against the showing editor's own key (a shared table).
        BMenuKind::HostToggle(key, on_by_default) => {
            let on = crate::v2::host::BHostType::from_slug(host).is_some_and(|h| {
                let k = key(h);
                if on_by_default { s.get(&k) != Some(V_FALSE) } else { s.get(&k) == Some(V_TRUE) }
            });
            BView::MenuItem(
                BControl::new()
                    .with_icon(if on { "check" } else { "" })
                    .with_label(it.label)
                    .with_shortcut(it.shortcut)
                    .with_active(on)
                    .with_action(leaf_action(host, labels, it)),
            )
        }
        _ => BView::MenuItem(
            BControl::new()
                .with_label(it.label)
                .with_shortcut(it.shortcut)
                .with_action(leaf_action(host, labels, it)),
        ),
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
