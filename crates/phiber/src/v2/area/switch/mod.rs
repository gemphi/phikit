//! `wm.context_set_enum` on `area.type` — the Toggle Graph Editor / Toggle
//! Dope Sheet entries in the animation editors' View menus
//! (`space_dopesheet.py`, `space_graph.py`): one area's editor type swaps for
//! another, Blender's Ctrl Tab in the Dope Sheet.
//!
//! Blender targets the area under the pointer. phiber's menus do not carry
//! the pointer; the firing host is known (`fire_host`), so the rule finds
//! every area in the active space whose stored host is the firing one and
//! writes the target host into its slot. Usually one area — a workspace that
//! stacks two of them (Shading) would swap both, which is a recorded
//! divergence. The flip survives like any other area host choice
//! (`area/current::area_key`), so it rides the session with the rest.

use crate::v2::area::current::area_key;
use crate::v2::host::BHostType;
use crate::v2::space::BSpaceList;
use crate::v2::state::rule::BRule;
use crate::v2::state::{BPath, BState, OpKey};
use crate::v2::state::BKey as _;

pub struct BEditorTypeRule;

impl BRule for BEditorTypeRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::EditorType.path()
    }

    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        s.remove(OpKey::EditorType);
        let Some((from, to)) = value.split_once(':') else { return };
        let (Some(from_host), Some(to_host)) = (BHostType::from_slug(from), BHostType::from_slug(to)) else { return };
        let space = BSpaceList::active(s);
        // Every area whose stored (or default) host is the firing one, in the
        // active space's own area list — switching the Dope Sheet in the
        // Animation workspace does not reach into Layout.
        let changes: Vec<(BPath, &'static str)> = space
            .spec()
            .areas
            .iter()
            .filter_map(|area| {
                let key = area_key(space, area);
                let current = s.get(&key).and_then(BHostType::from_slug).unwrap_or(area.default_host);
                (current == from_host && area.hosts.contains(&to_host)).then(|| (key, to_host.slug()))
            })
            .collect();
        for (key, slug) in changes {
            s.set(&key, slug);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::space::BSpaceType;
    use crate::v2::state::{CtxKey, action::apply_action};
    use crate::v2::view::prim::BAction;

    fn fire(s: &mut BState, value: &str) {
        apply_action(s, &BAction::Set(OpKey::EditorType.path(), value.to_string()));
    }

    #[test]
    fn the_rule_answers_only_its_own_op() {
        assert!(BEditorTypeRule.wants(&OpKey::EditorType.path()));
        assert!(!BEditorTypeRule.wants(&OpKey::ObjectParent.path()));
    }

    #[test]
    fn an_unknown_host_or_malformed_value_writes_nothing() {
        let mut s = BState::new();
        s.set(CtxKey::Space, BSpaceType::Animation.slug());
        let before: Vec<(String, String)> = BSpaceType::Animation
            .spec()
            .areas
            .iter()
            .map(|a| (area_key(BSpaceType::Animation, a).encode(), s.get_encoded(&area_key(BSpaceType::Animation, a).encode()).unwrap_or("").to_string()))
            .collect();
        fire(&mut s, "nonsense");
        fire(&mut s, "no:such");
        for (key, was) in &before {
            assert_eq!(s.get_encoded(key).unwrap_or("").to_string(), *was, "{key} unchanged");
        }
    }

    #[test]
    fn toggling_from_an_action_area_writes_the_graph_slug_into_its_slot() {
        // The Animation workspace's Center area defaults to the Dope Sheet
        // (`BHostType::Action`) and can switch to the Graph editor
        // (`BHostType::Ipo`). The toggle moves it.
        let mut s = BState::new();
        s.set(CtxKey::Space, BSpaceType::Animation.slug());
        let Some(area) = BSpaceType::Animation.spec().areas.iter().find(|a| a.default_host == BHostType::Action && a.hosts.contains(&BHostType::Ipo)) else {
            return; // this workspace has no such area; the test is a no-op
        };
        let key = area_key(BSpaceType::Animation, area);
        fire(&mut s, "action:ipo");
        assert_eq!(s.get(&key), Some("ipo"), "the slot now holds the Graph editor");
        // Toggling back puts it where it was.
        fire(&mut s, "ipo:action");
        assert_eq!(s.get(&key), Some("action"));
    }
}
