//! The scene range from the playhead — Blender's `anim.start_frame_set`
//! (Ctrl Home) and `anim.end_frame_set` (Ctrl End): the current frame
//! becomes the start or the end of the range, and the other end gives way
//! if the range would fold.

use crate::v2::state::key::int_or;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, log, rule::BRule};

/// `ctx.op.frame_range` → `start` / `end`.
pub struct BFrameRangeRule;

impl BRule for BFrameRangeRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::FrameRange.path()
    }
    fn after(&self, _: &BPath, which: &str, state: &mut BState) {
        let frame = int_or(state, CtxKey::Frame, 1);
        match which {
            "start" => {
                state.set(CtxKey::FrameStart, frame.to_string());
                if int_or(state, CtxKey::FrameEnd, 250) < frame {
                    state.set(CtxKey::FrameEnd, frame.to_string());
                }
                log::push(format!("Start frame {frame}"));
            }
            "end" => {
                state.set(CtxKey::FrameEnd, frame.to_string());
                if int_or(state, CtxKey::FrameStart, 1) > frame {
                    state.set(CtxKey::FrameStart, frame.to_string());
                }
                log::push(format!("End frame {frame}"));
            }
            _ => {}
        }
        state.remove(OpKey::FrameRange);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    #[test]
    fn the_playhead_becomes_the_start_or_the_end_and_the_range_never_folds() {
        let mut state = BState::new();
        state.set(CtxKey::Frame, "40");
        apply_action(&mut state, &BAction::Set(OpKey::FrameRange.path(), "start".into()));
        assert_eq!(state.get(CtxKey::FrameStart), Some("40"));
        state.set(CtxKey::Frame, "20");
        apply_action(&mut state, &BAction::Set(OpKey::FrameRange.path(), "end".into()));
        assert_eq!((state.get(CtxKey::FrameStart), state.get(CtxKey::FrameEnd)), (Some("20"), Some("20")));
        assert!(state.get(OpKey::FrameRange).is_none());
    }
}
