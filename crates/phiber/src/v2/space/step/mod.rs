//! Cycling workspaces — Blender's `screen.workspace_cycle` (Ctrl PageUp /
//! PageDown): the next or previous tab of the top bar, round the end.

use crate::v2::space::BSpaceType;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, log, rule::BRule};

/// `ctx.op.space_step` → `+1` / `-1` along `BSpaceType::ALL`.
pub struct BSpaceStepRule;

impl BRule for BSpaceStepRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::SpaceStep.path()
    }
    fn after(&self, _: &BPath, step: &str, state: &mut BState) {
        let tabs = BSpaceType::ALL;
        let now = state.get(CtxKey::Space).and_then(BSpaceType::from_slug).and_then(|s| tabs.iter().position(|t| *t == s)).unwrap_or(0);
        let by: i64 = step.parse().unwrap_or(1);
        let next = tabs[(now as i64 + by).rem_euclid(tabs.len() as i64) as usize];
        super::switch::switch_space(state, next);
        log::push(format!("Switched to {}", next.label()));
        state.remove(OpKey::SpaceStep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    #[test]
    fn next_and_previous_walk_the_tabs_round_the_end() {
        let mut state = BState::new();
        state.set(CtxKey::Space, BSpaceType::ALL[0].slug());
        apply_action(&mut state, &BAction::Set(OpKey::SpaceStep.path(), "-1".into()));
        assert_eq!(state.get(CtxKey::Space), Some(BSpaceType::ALL.last().unwrap().slug()));
        apply_action(&mut state, &BAction::Set(OpKey::SpaceStep.path(), "+1".into()));
        assert_eq!(state.get(CtxKey::Space), Some(BSpaceType::ALL[0].slug()));
    }
}
