//! `ACTION_OT_previewrange_set` and `anim.previewrange_clear` — the preview
//! range the animation editors set from a selection (Ctrl Alt P) and clear
//! (Alt P).
//!
//! Ported from `editors/space_action/action_edit.cc::actkeys_previewrange_exec`:
//! take the x-extents of the selected keyframes, floor the start and ceil the
//! end, bump the end by one if they collapse, write them into the scene's
//! preview start/end, and set the preview-range flag. In phiber the preview
//! range is `(CtxKey::PreviewStart, CtxKey::PreviewEnd, CtxKey::PreviewRange)`,
//! and the selection is `BKeySelection` — columns rather than per-key bits, so
//! the frame of each pick is already the x used by `get_keyframe_extents`.
//!
//! Clear writes `PreviewRange = V_FALSE` so the scene plays its whole range,
//! matching `BAnimMenu::clear_preview` (`part/menubar/editor/anim/set.rs`).

use super::dope::BKeySelection;
use crate::v2::state::rule::BRule;
use crate::v2::state::val::{V_FALSE, V_TRUE};
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, log};

pub struct BPreviewRangeRule;

impl BRule for BPreviewRangeRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::PreviewRange.path()
    }

    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        s.remove(OpKey::PreviewRange);
        match value {
            "clear" => {
                s.set(CtxKey::PreviewRange, V_FALSE);
            }
            "selected" => {
                let picks = BKeySelection::now(s);
                if picks.0.is_empty() {
                    log::push("No keyframes selected to make a preview range from".to_string());
                    return;
                }
                // `get_keyframe_extents`: the floor of the earliest and ceil
                // of the latest. The columns are already frames, so the
                // conversion is just floor / ceil (`action_edit.cc`).
                let (min, max) = picks.0.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), p| (lo.min(p.frame), hi.max(p.frame)));
                let start = min.floor() as i32;
                let end = {
                    let e = max.ceil() as i32;
                    // `psfra == pefra ? pefra = psfra + 1`.
                    if e == start { start + 1 } else { e }
                };
                s.set(CtxKey::PreviewStart, start.to_string());
                s.set(CtxKey::PreviewEnd, end.to_string());
                s.set(CtxKey::PreviewRange, V_TRUE);
                log::push(format!("Preview range set to {start} .. {end}"));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::dope::BKeyPick;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    fn fire(s: &mut BState, value: &str) {
        apply_action(s, &BAction::Set(OpKey::PreviewRange.path(), value.to_string()));
    }

    #[test]
    fn the_rule_answers_only_its_own_op() {
        assert!(BPreviewRangeRule.wants(&OpKey::PreviewRange.path()));
        assert!(!BPreviewRangeRule.wants(&OpKey::EditorType.path()));
    }

    #[test]
    fn selected_takes_the_floor_and_ceil_of_the_selection_and_turns_on() {
        let mut s = BState::new();
        let picks = BKeySelection(vec![
            BKeyPick { object: "Cube".into(), frame: 12.4 },
            BKeyPick { object: "Cube".into(), frame: 20.6 },
            BKeyPick { object: "Cube".into(), frame: 5.9 },
        ]);
        picks.write(&mut s);
        fire(&mut s, "selected");
        assert_eq!(s.get(CtxKey::PreviewStart), Some("5"), "floor of 5.9");
        assert_eq!(s.get(CtxKey::PreviewEnd), Some("21"), "ceil of 20.6");
        assert_eq!(s.get(CtxKey::PreviewRange), Some(V_TRUE));
    }

    #[test]
    fn selected_bumps_the_end_by_one_when_start_and_end_collapse() {
        // `psfra == pefra ? pefra = psfra + 1`: a single whole-frame pick.
        let mut s = BState::new();
        BKeySelection(vec![BKeyPick { object: "Cube".into(), frame: 10.0 }]).write(&mut s);
        fire(&mut s, "selected");
        assert_eq!(s.get(CtxKey::PreviewStart), Some("10"));
        assert_eq!(s.get(CtxKey::PreviewEnd), Some("11"), "bumped by one");
    }

    #[test]
    fn selected_with_an_empty_selection_leaves_the_range_alone() {
        let mut s = BState::new();
        s.set(CtxKey::PreviewRange, V_TRUE);
        fire(&mut s, "selected");
        assert_eq!(s.get(CtxKey::PreviewRange), Some(V_TRUE), "nothing to extend");
        assert_eq!(s.get(CtxKey::PreviewStart), None);
    }

    #[test]
    fn clear_turns_the_preview_range_off() {
        let mut s = BState::new();
        s.set(CtxKey::PreviewRange, V_TRUE);
        fire(&mut s, "clear");
        assert_eq!(s.get(CtxKey::PreviewRange), Some(V_FALSE));
    }
}
