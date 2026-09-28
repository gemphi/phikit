//! Menu Search rows (Blender F3 `wm.search_menu`): every leaf of the Topbar
//! menus, found by recursing the declarative menu set, shown as
//! "File ▸ Import ▸ glTF 2.0" with its shortcut and firing the same action as
//! the menu item. Filtered by `command.query` (case-insensitive).

use crate::v2::state::ModalKey;
use crate::v2::host::BHostType;
use crate::v2::part::menubar::tree::{item_path, leaf_action};
use crate::v2::part::menubar::item::{BMenuItem, BMenuKind, BMenuSpec};
use crate::v2::state::val::V_EMPTY;
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BRowShade, BTableCell};

pub struct BCommandHit {
    pub path: String,
    pub shortcut: &'static str,
    pub action: BAction,
}

fn walk(trail: &mut Vec<&'static str>, items: &'static [BMenuItem], out: &mut Vec<BCommandHit>) {
    for it in items {
        match it.kind {
            BMenuKind::Sep => {}
            BMenuKind::Sub(subs) => {
                trail.push(it.label);
                walk(trail, subs, out);
                trail.pop();
            }
            BMenuKind::Recent | BMenuKind::History => {}
            BMenuKind::Op | BMenuKind::Toggle(..) | BMenuKind::Act(_) | BMenuKind::Fire(..) => {
                // The same action the menu fires; an operator closes the
                // palette as it runs (`BPaletteDismissRule`).
                let action = leaf_action(BHostType::Topbar.slug(), trail, it);
                let path = item_path(trail, it.label);
                out.push(BCommandHit { path, shortcut: it.shortcut, action });
            }
        }
    }
}

/// All searchable menu leaves matching `query`.
pub fn hits(query: &str) -> Vec<BCommandHit> {
    let mut out = Vec::new();
    for menu in BMenuSpec::list_for(BHostType::Topbar) {
        walk(&mut vec![menu.label], menu.items, &mut out);
    }
    let q = query.trim().to_lowercase();
    out.retain(|h| q.is_empty() || h.path.to_lowercase().contains(&q));
    out
}

pub fn rows(query: &str) -> Vec<BView> {
    hits(query)
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            let shade = if i % 2 == 1 { BRowShade::Alt } else { BRowShade::Flat };
            let cell_list = vec![
                BTableCell::flex(BView::label(h.path)),
                BTableCell::fixed_end(BView::label(h.shortcut), 96.0),
            ];
            BView::table_row(cell_list, shade, Some(h.action))
        })
        .collect()
}

/// An operator picked from the palette runs and the palette goes: any
/// `ctx.op.*` intent closes the command window (Blender's menu search
/// closes on the pick, the operator running from where it was).
pub struct BPaletteDismissRule;

impl crate::v2::state::rule::BRule for BPaletteDismissRule {
    fn wants(&self, path: &crate::v2::state::BPath) -> bool {
        path.segs.first() == Some(&"op")
    }
    fn after(&self, _: &crate::v2::state::BPath, _: &str, state: &mut crate::v2::state::BState) {
        if state.get(ModalKey::Open) == Some(crate::v2::state::val::V_MODAL_COMMAND) {
            state.set(ModalKey::Open, V_EMPTY);
        }
    }
}
