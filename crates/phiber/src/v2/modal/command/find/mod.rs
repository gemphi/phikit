//! What the palette finds — every place the window can take you, as one
//! list: the spaces, each space's editors, each editor's tabs and panels,
//! the Preferences sections, and every menu item (`palette`).
//!
//! A result is a path through the hierarchy — `Shading ▸ 3D Viewport ▸
//! Sidebar ▸ Item` — and picking it walks that path: the space opens, the
//! editor's tab is picked, the panel unfolds (`BFindGoRule`). The tabs along
//! the palette's top narrow the list to one kind (`BFindKind`).

use crate::v2::host::BHostType;
use crate::v2::part::props::BPanelItem;
use crate::v2::space::BSpaceType;
use crate::v2::state::val::{V_EMPTY, V_MODAL_COMMAND, V_MODAL_SETTINGS, V_TRUE};
use crate::v2::state::{AppKey, BKey as _, BPath, BState, CtxKey, ModalKey};
use crate::v2::view::prim::BAction;

mod kind;
pub use kind::BFindKind;

/// One result: what it is, its icon, the path it reads as, a hint at the
/// row's end (a shortcut, or the kind), and what picking it does.
#[derive(Clone, Debug)]
pub struct BFindHit {
    pub kind: BFindKind,
    pub icon: &'static str,
    pub path: String,
    pub hint: &'static str,
    pub action: BAction,
}

/// The keys a pick sets, one `ctx.path=value` per line — `command.go`'s value.
fn go(sets: &[(BPath, &str)]) -> BAction {
    let lines: Vec<String> = sets.iter().map(|(path, value)| format!("{}={value}", path.encode())).collect();
    BAction::Set(AppKey::CommandGo.path(), lines.join("\n"))
}

/// The editors a space opens with: each area's default host, once.
fn editors_of(space: BSpaceType) -> Vec<BHostType> {
    let mut out: Vec<BHostType> = Vec::new();
    for area in space.spec().areas {
        // The window's own bars and the activity rail are chrome, not
        // editors a search should take you to.
        let host = area.default_host;
        let chrome = host.is_fixed_bar() || host.is_fixed_strip() || host == BHostType::Empty;
        if !out.contains(&host) && !chrome {
            out.push(host);
        }
    }
    out
}

/// The space an editor is first found in, for a panel picked without one.
fn home_of(host: BHostType) -> Option<BSpaceType> {
    BSpaceType::ALL.iter().copied().find(|space| editors_of(*space).contains(&host))
}

fn spaces() -> Vec<BFindHit> {
    BSpaceType::ALL
        .iter()
        .map(|space| BFindHit {
            kind: BFindKind::Space,
            icon: space.icon(),
            path: space.label().to_string(),
            hint: "Space",
            action: BAction::Set(CtxKey::Space.path(), space.slug().to_string()),
        })
        .collect()
}

fn editors() -> Vec<BFindHit> {
    let mut out = Vec::new();
    for space in BSpaceType::ALL {
        for host in editors_of(*space) {
            out.push(BFindHit {
                kind: BFindKind::Editor,
                icon: host.icon(),
                path: format!("{} ▸ {}", space.label(), host.label()),
                hint: "Editor",
                action: go(&[(CtxKey::Space.path(), space.slug())]),
            });
        }
    }
    out
}

/// The panels: Properties' tabs and their panels, each sidebar's tabs, and
/// the Preferences sections.
fn panels() -> Vec<BFindHit> {
    let mut out = Vec::new();
    if let Some(space) = home_of(BHostType::Buts) {
        let tab_key = crate::v2::state::ButsKey::NavbarTab.path();
        for tab in crate::v2::part::navbar::set::tabs_for(BHostType::Buts) {
            out.push(BFindHit {
                kind: BFindKind::Panel,
                icon: tab.icon,
                path: format!("{} ▸ Properties ▸ {}", space.label(), tab.label),
                hint: "Tab",
                action: go(&[(CtxKey::Space.path(), space.slug()), (tab_key.clone(), tab.label)]),
            });
            for section in crate::v2::part::props::set::sections_for(BHostType::Buts, tab.label) {
                if section.headless || section.title.is_empty() {
                    continue;
                }
                let open = BPanelItem::expand_key(BHostType::Buts, tab.label, section.title);
                out.push(BFindHit {
                    kind: BFindKind::Panel,
                    icon: tab.icon,
                    path: format!("{} ▸ Properties ▸ {} ▸ {}", space.label(), tab.label, section.title),
                    hint: "Panel",
                    action: go(&[(CtxKey::Space.path(), space.slug()), (tab_key.clone(), tab.label), (open, V_TRUE)]),
                });
            }
        }
    }
    for space in BSpaceType::ALL {
        for host in editors_of(*space) {
            let shown = crate::v2::part::sidebar::show::BSidebarShown(host);
            for tab in crate::v2::part::sidebar::set::tabs_for(host) {
                out.push(BFindHit {
                    kind: BFindKind::Panel,
                    icon: tab.icon,
                    path: format!("{} ▸ {} ▸ Sidebar ▸ {}", space.label(), host.label(), tab.label),
                    hint: "Sidebar",
                    action: go(&[
                        (CtxKey::Space.path(), space.slug()),
                        (BPath::owned(vec![host.slug(), "show", "sidebar"]), V_TRUE),
                        (shown.tab_key(), tab.label),
                    ]),
                });
            }
        }
    }
    let prefs = BPath::owned(vec![BHostType::Prefs.slug(), "navbar", "tab"]);
    for tab in crate::v2::modal::setting::NAV_TABS {
        out.push(BFindHit {
            kind: BFindKind::Panel,
            icon: tab.icon,
            path: format!("Preferences ▸ {}", tab.label),
            hint: "Preferences",
            action: go(&[(prefs.clone(), tab.label), (ModalKey::Open.path(), V_MODAL_SETTINGS)]),
        });
    }
    out
}

fn menus(over: Option<BHostType>) -> Vec<BFindHit> {
    super::palette::hits("", over)
        .into_iter()
        .map(|hit| BFindHit { kind: BFindKind::Menu, icon: "menu", path: hit.path, hint: hit.shortcut, action: hit.action })
        .collect()
}

/// Every result of `kind` matching `query` (case-insensitive, every word
/// somewhere in the path), spaces first, menus last.
pub fn hits(query: &str, kind: BFindKind, over: Option<BHostType>) -> Vec<BFindHit> {
    let mut out = Vec::new();
    let wants = |k: BFindKind| kind == BFindKind::All || kind == k;
    if wants(BFindKind::Space) {
        out.extend(spaces());
    }
    if wants(BFindKind::Editor) {
        out.extend(editors());
    }
    if wants(BFindKind::Panel) {
        out.extend(panels());
    }
    if wants(BFindKind::Menu) {
        out.extend(menus(over));
    }
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    out.retain(|hit| {
        let path = hit.path.to_lowercase();
        words.iter().all(|word| path.contains(word.as_str()))
    });
    out
}

/// A palette pick: each key `command.go` names is set, in order, and the
/// palette closes (unless the pick opened another dialog — Preferences).
pub struct BFindGoRule;

impl crate::v2::state::rule::BRule for BFindGoRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == AppKey::CommandGo.path()
    }
    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        if s.get(ModalKey::Open) == Some(V_MODAL_COMMAND) {
            s.set(ModalKey::Open, V_EMPTY);
        }
        for line in value.lines() {
            if let Some((path, to)) = line.split_once('=') {
                s.set_encoded(path, to.to_string());
            }
        }
        crate::v2::state::edit::stop(s);
    }
}

/// Clicking the top bar's search field opens the palette, with the caret in
/// its own field, rather than typing into the bar.
pub struct BFindFromBarRule;

impl crate::v2::state::rule::BRule for BFindFromBarRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == AppKey::Edit.path()
    }
    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        if value == AppKey::CommandQuery.path().encode() && s.get(ModalKey::Open) != Some(V_MODAL_COMMAND) {
            s.set(ModalKey::Open, V_MODAL_COMMAND);
            super::palette::opened(s);
        }
    }
}

#[cfg(test)]
mod tests;
