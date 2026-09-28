//! Folding every panel of the shown tab at once — the Properties editor's
//! View ▸ Expand All / Collapse All (Blender's Ctrl-click on a header, done
//! for every panel). One `Set` of `ctx.op.props`; every section of the tab
//! in front, sub-panels included, gets its expansion key written.

use super::set::{BSectionSpec, sections_for};
use crate::v2::part::navbar::set::tabs_for;
use super::BPanelItem;
use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, BPath, BState, OpKey, log, rule::BRule, val::{V_FALSE, V_TRUE}};

/// `ctx.op.props` → `expand` / `collapse`.
pub struct BPropsFoldRule;

fn fold(state: &mut BState, host: BHostType, tab: &str, section: &BSectionSpec, path: &str, open: bool) {
    state.set(&BPanelItem::expand_key(host, tab, path), if open { V_TRUE } else { V_FALSE });
    for child in section.list {
        fold(state, host, tab, child, &format!("{path} {}", child.title), open);
    }
}

impl BRule for BPropsFoldRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Props.path()
    }
    fn after(&self, _: &BPath, op: &str, state: &mut BState) {
        let open = match op {
            "expand" => true,
            "collapse" => false,
            _ => return state.remove(OpKey::Props),
        };
        let host = BHostType::Buts;
        let tabs = tabs_for(host);
        let fallback = tabs.first().map_or("", |t| t.label);
        let tab = state.host(host).get("navbar.tab").unwrap_or(fallback).to_string();
        for section in sections_for(host, &tab) {
            fold(state, host, &tab, section, section.title, open);
        }
        log::push(if open { "Expanded every panel" } else { "Collapsed every panel" }.to_string());
        state.remove(OpKey::Props);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    #[test]
    fn collapse_all_closes_every_panel_of_the_shown_tab_and_expand_all_opens_them() {
        let mut state = BState::new();
        state.set(&BPath::owned(vec![BHostType::Buts.slug(), "navbar", "tab"]), "Render");
        apply_action(&mut state, &BAction::Set(OpKey::Props.path(), "collapse".into()));
        for section in sections_for(BHostType::Buts, "Render") {
            let key = BPanelItem::expand_key(BHostType::Buts, "Render", section.title);
            assert!(!BPanelItem::is_open(&state, &key, section.closed), "{} is closed", section.title);
        }
        apply_action(&mut state, &BAction::Set(OpKey::Props.path(), "expand".into()));
        for section in sections_for(BHostType::Buts, "Render") {
            let key = BPanelItem::expand_key(BHostType::Buts, "Render", section.title);
            assert!(BPanelItem::is_open(&state, &key, section.closed), "{} is open", section.title);
        }
        assert!(state.get(OpKey::Props).is_none());
    }
}
