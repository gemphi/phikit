//! The Outliner's View menu operators.
//!
//! Blender ref: `outliner.show_hierarchy` (collections open, everything
//! under them closed), `outliner.show_one_level` with `open = True`
//! (Numpad +: the shallowest closed rows open) and `open = False`
//! (Numpad −: the deepest open rows close). Each is one `Set` of
//! `ctx.op.outliner`, turned here into the rows' own expansion keys.

use super::tree::{BOutlineRow, BRowKind, flatten};
use crate::v2::state::{BKey as _, BPath, BState, OpKey, OutlinerKey, log, rule::BRule, val::{V_FALSE, V_TRUE}};

/// `ctx.op.outliner` → `hierarchy` / `expand` / `collapse`.
pub struct BOutlinerOpRule;

impl BRule for BOutlinerOpRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Outliner.path()
    }
    fn after(&self, _: &BPath, op: &str, state: &mut BState) {
        let rows = flatten(state);
        let set = |state: &mut BState, row: &BOutlineRow, open: bool| {
            state.set(OutlinerKey::Expanded(row.id), if open { V_TRUE } else { V_FALSE });
        };
        let said = match op {
            "hierarchy" => {
                for row in rows.iter().filter(|r| r.expanded.is_some()) {
                    set(state, row, matches!(row.kind, BRowKind::SceneCollection | BRowKind::Collection));
                }
                "Showing the hierarchy"
            }
            "expand" => {
                let shallowest = rows.iter().filter(|r| r.expanded == Some(false)).map(|r| r.depth).min();
                for row in rows.iter().filter(|r| r.expanded == Some(false) && Some(r.depth) == shallowest) {
                    set(state, row, true);
                }
                "Expanded one level"
            }
            "collapse" => {
                let deepest = rows.iter().filter(|r| r.expanded == Some(true)).map(|r| r.depth).max();
                for row in rows.iter().filter(|r| r.expanded == Some(true) && Some(r.depth) == deepest) {
                    set(state, row, false);
                }
                "Collapsed one level"
            }
            _ => return state.remove(OpKey::Outliner),
        };
        log::push(said.to_string());
        state.remove(OpKey::Outliner);
    }
}

/// Hiding an object deselects it: the eye (`hide_get`) or the monitor
/// (`hide_viewport`) set on an object takes it out of the selection, as
/// Blender's base evaluation does (`BKE_base_eval_flags` drops
/// `BASE_SELECTED` from what cannot be seen) — and as the saved file does.
pub struct BHideDeselectsRule;

impl BRule for BHideDeselectsRule {
    fn wants(&self, path: &BPath) -> bool {
        matches!(path.segs.as_ref(), ["outliner", object, "hide_get" | "hide_viewport"] if !matches!(*object, "col" | "filter" | "expanded"))
    }
    fn after(&self, path: &BPath, value: &str, state: &mut BState) {
        let (Some(object), true) = (path.segs.get(1), value == V_TRUE) else { return };
        let mut selection = crate::v2::scene::select::BSelection::now(state);
        if selection.contains(object) {
            selection.list.retain(|name| name != object);
            selection.write(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    fn op(state: &mut BState, what: &str) {
        apply_action(state, &BAction::Set(OpKey::Outliner.path(), what.into()));
    }

    fn open_depths(state: &BState) -> Vec<(u16, bool)> {
        flatten(state).iter().filter_map(|r| r.expanded.map(|open| (r.depth, open))).collect()
    }

    #[test]
    fn collapse_closes_the_deepest_open_rows_and_expand_opens_the_shallowest_closed() {
        crate::v2::scene::source::clear_loaded_scene();
        let mut state = BState::new();
        let before = open_depths(&state);
        assert!(before.iter().any(|(_, open)| *open), "the default scene's groups start open");
        op(&mut state, "collapse");
        let deepest = before.iter().filter(|(_, open)| *open).map(|(d, _)| *d).max().unwrap();
        assert!(open_depths(&state).iter().all(|(d, open)| *d != deepest || !open), "the deepest open level closed");
        op(&mut state, "expand");
        assert_eq!(open_depths(&state), before, "and one level back opens it again");
        assert!(state.get(OpKey::Outliner).is_none());
    }

    #[test]
    fn hiding_an_object_takes_it_out_of_the_selection() {
        use crate::v2::scene::select::BSelection;
        let mut state = BState::new();
        BSelection { list: vec!["Cube".into(), "Light".into()] }.write(&mut state);
        apply_action(&mut state, &BAction::Set(super::super::key::k_restrict("Cube", "hide_get"), V_TRUE.into()));
        assert_eq!(BSelection::now(&state).list, ["Light"], "the eye deselects");
        apply_action(&mut state, &BAction::Set(super::super::key::k_restrict("Light", "hide_render"), V_TRUE.into()));
        assert_eq!(BSelection::now(&state).list, ["Light"], "the camera does not");
        apply_action(&mut state, &BAction::Set(super::super::key::k_restrict("Light", "hide_viewport"), V_TRUE.into()));
        assert!(BSelection::now(&state).list.is_empty(), "the monitor does");
    }
}
