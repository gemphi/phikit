//! Menu Search rows (Blender F3 `wm.search_menu`): every leaf of the Topbar
//! menus and of the editor F3 was pressed over, found by recursing the
//! declarative menu set, shown as "File ▸ Import ▸ glTF 2.0" with its
//! shortcut and firing the same action as the menu item. Filtered by
//! `command.query` (case-insensitive); Return runs the first.

use crate::v2::state::{AppKey, BState, ModalKey};
use crate::v2::host::BHostType;
use crate::v2::part::menubar::tree::{item_path, leaf_action};
use crate::v2::part::menubar::item::{BMenuItem, BMenuKind, BMenuSpec};
use crate::v2::state::BKey as _;
use crate::v2::state::val::V_EMPTY;
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BRowShade, BTableCell};

pub struct BCommandHit {
    pub path: String,
    pub shortcut: &'static str,
    pub action: BAction,
}

fn walk(host: BHostType, trail: &mut Vec<&'static str>, items: &'static [BMenuItem], out: &mut Vec<BCommandHit>) {
    for it in items {
        match it.kind {
            BMenuKind::Sep => {}
            BMenuKind::Sub(subs) => {
                trail.push(it.label);
                walk(host, trail, subs, out);
                trail.pop();
            }
            BMenuKind::Recent | BMenuKind::History | BMenuKind::Live(_) => {}
            BMenuKind::Op | BMenuKind::Toggle(..) | BMenuKind::HostToggle(..) | BMenuKind::Act(_) | BMenuKind::Fire(..) | BMenuKind::HostFire(..) => {
                // The same action the menu fires; an operator closes the
                // palette as it runs (`BPaletteDismissRule`).
                let action = leaf_action(host.slug(), trail, it);
                let path = item_path(trail, it.label);
                out.push(BCommandHit { path, shortcut: it.shortcut, action });
            }
        }
    }
}

/// The editor the palette searches beside the Topbar, as recorded when it
/// opened.
pub fn host(s: &BState) -> Option<BHostType> {
    s.get(AppKey::CommandHost).and_then(BHostType::from_slug).filter(|h| *h != BHostType::Topbar)
}

/// All searchable menu leaves matching `query`: the Topbar's, then those
/// of `over`, the editor under the pointer.
pub fn hits(query: &str, over: Option<BHostType>) -> Vec<BCommandHit> {
    let mut out = Vec::new();
    for host in std::iter::once(BHostType::Topbar).chain(over) {
        for menu in BMenuSpec::list_for(host) {
            walk(host, &mut vec![menu.label], menu.items, &mut out);
        }
    }
    let q = query.trim().to_lowercase();
    out.retain(|h| q.is_empty() || h.path.to_lowercase().contains(&q));
    out
}

/// The palette's result rows: each result's icon, its path through the
/// hierarchy, and its hint (a shortcut, or what kind of place it is).
pub fn rows(query: &str, kind: super::find::BFindKind, over: Option<BHostType>, layout: super::layout::BFindLayout) -> Vec<BView> {
    let hits = super::find::hits(query, kind, over);
    if let Some(size) = layout.card() {
        // A card each: the place's own name on it, what kind of place it is
        // under, the whole path in its tooltip's stead as the subtitle's line.
        let cards = hits
            .into_iter()
            .map(|h| {
                let name = h.path.rsplit(" \u{25B8} ").next().unwrap_or(&h.path).to_string();
                BView::thumbnail(crate::v2::view::BThumb {
                    preview_path: String::new(),
                    icon: h.icon.to_string(),
                    label: name,
                    subtitle: h.hint.to_string(),
                    size,
                    selected: false,
                    action: Some(h.action),
                })
            })
            .collect();
        return vec![BView::grid(cards)];
    }
    hits
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            let shade = if i % 2 == 1 { BRowShade::Alt } else { BRowShade::Flat };
            // The icon on its tile, as every icon in the window is.
            let tile = BView::Button(crate::v2::view::prim::BControl::new().with_icon(h.icon).with_action(h.action.clone()));
            let cell_list = vec![
                BTableCell::fixed(tile, 36.0),
                BTableCell::flex(BView::label(h.path)),
                BTableCell::fixed_end(BView::label(h.hint), 96.0),
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

/// The palette opening (`wm.search_menu`, F3): the query empties, the
/// editor under the pointer is remembered for its menus, and the field
/// takes the caret so typing searches at once.
pub struct BPaletteOpenRule;

impl crate::v2::state::rule::BRule for BPaletteOpenRule {
    fn wants(&self, path: &crate::v2::state::BPath) -> bool {
        *path == ModalKey::Open.path()
    }
    fn after(&self, _: &crate::v2::state::BPath, value: &str, state: &mut BState) {
        if value == crate::v2::state::val::V_MODAL_COMMAND {
            opened(state);
        }
    }
}

/// The palette has opened: the query empties, the editor under the pointer
/// is remembered for its menus, and the field takes the caret so typing
/// searches at once.
pub fn opened(state: &mut BState) {
    let over = crate::v2::view::draw::host::BPointer::host().map_or("", |h| h.slug());
    state.set(AppKey::CommandHost, over);
    state.set(AppKey::CommandQuery, V_EMPTY);
    crate::v2::state::edit::start(state, AppKey::CommandQuery);
}
