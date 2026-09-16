//! `item_view` — recursive `BMenuItem` → `BView` translation. `BMenuKind::Sub`
//! becomes a nested popover opening to the RIGHT of its parent row. Sub
//! anchors and top-level items carry a hover-Open action so hovering either
//! opens the target menu (with `apply_open`'s sibling-closing logic this
//! gives clean hover-switch between sibling submenus).

use super::super::set::{BMenuItem, BMenuKind};
use crate::v2::state::{BPath, BState, val::*};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};

/// Resolve the action a leaf menu item fires. When the item declares a
/// `modal` slug (set-driven), it opens that modal; otherwise it is an
/// ordinary operator that just clears (closes) the menu. This reads the
/// item's declarative intent — there is no per-`(menu, label)` routing table.
fn leaf_action(parents: &[&'static str], it: &BMenuItem) -> BAction {
    if !it.modal.is_empty() {
        return BAction::Set(BPath::owned(vec!["modal"]), it.modal.to_string());
    }
    BAction::Set(BPath::owned(parents.to_vec()), V_EMPTY.to_string())
}

pub fn item_view(parents: &[&'static str], it: &BMenuItem, s: &BState) -> BView {
    match it.kind {
        BMenuKind::Sep => BView::menu_sep(),
        BMenuKind::Sub(subs) => {
            let mut sub_segs: Vec<&'static str> = parents.to_vec();
            sub_segs.push(it.label);
            let key = BPath::owned(sub_segs.clone());
            let open = s.get(&key) == Some(V_TRUE);
            let toggle = BAction::Toggle(key.clone());
            let opener = BAction::Open(key.clone());
            let close = BAction::Set(key, V_EMPTY.to_string());
            let anchor = BView::MenuItem(
                BControl::new()
                    .with_label(it.label)
                    .with_shortcut("▶")
                    .with_action(toggle)
                    .with_hover(opener),
            );
            let body = subs.iter().map(|c| item_view(&sub_segs, c, s)).collect();
            BView::popover(anchor, body, open, close, BPopoverSide::Right)
        }
        _ => BView::MenuItem(
            BControl::new()
                .with_label(it.label)
                .with_shortcut(it.shortcut)
                .with_action(leaf_action(parents, it)),
        ),
    }
}
