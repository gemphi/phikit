//! Transport — Blender's `screen.frame_offset`, `screen.frame_jump` and
//! `screen.keyframe_jump`.
//!
//! The header's buttons write a step intent (`"+1"`, `"-1"`, `"start"`,
//! `"end"`, `"key-"`, `"key+"`); [`BFrameStepRule`] moves the current frame
//! and clears the intent. Offsets wrap inside the scene range like looping
//! playback; a keyframe jump lands on the nearest key column of the shown
//! objects in that direction and, like Blender ("No more keyframes to jump
//! to in this direction"), stays put when there is none.
//!
//! Playback's own step is [`tick`]: what `screen_animation_step` does at
//! the end of the range depends on the Playback popover's loop mode, and
//! `anim.change_frame` keeps a scrub inside the range while "Limit to
//! Frame Range" is on ([`BFrameLockRule`]).

use crate::phiphx::blend::BlendLoopMode;
use crate::v2::part::playback::settings;
use crate::v2::state::key::int_or;
use crate::v2::state::val::{V_FALSE, V_REVERSE, V_TRUE};
use crate::v2::state::{BKey, BPath, BState, CtxKey, rule::BRule};

/// Blender's factory range, until a file sets its own.
const FACTORY_START: i32 = 1;
const FACTORY_END: i32 = 250;

/// The frame `step` lands on from `cur` inside `start..=end`, given the
/// sorted key columns a keyframe jump may land on.
pub fn stepped(cur: i32, start: i32, end: i32, step: &str, keys: &[f32]) -> i32 {
    match step {
        "start" => start,
        "end" => end,
        "key+" => keys.iter().map(|k| k.round() as i32).find(|&k| k > cur).unwrap_or(cur),
        "key-" => keys.iter().rev().map(|k| k.round() as i32).find(|&k| k < cur).unwrap_or(cur),
        d => match d.parse::<i32>() {
            Ok(d) if cur + d > end => start,
            Ok(d) if cur + d < start => end,
            Ok(d) => cur + d,
            Err(_) => cur,
        },
    }
}

/// `frame_step` → `frame`.
pub struct BFrameStepRule;

impl BRule for BFrameStepRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == CtxKey::FrameStep.path()
    }
    fn after(&self, _: &BPath, step: &str, s: &mut BState) {
        let start = int_or(s, CtxKey::FrameStart, FACTORY_START);
        let end = int_or(s, CtxKey::FrameEnd, FACTORY_END).max(start);
        let cur = int_or(s, CtxKey::Frame, start);
        let keys = if step.starts_with("key") { crate::v2::scene::BShownKeys::now(s) } else { Default::default() };
        s.set(CtxKey::Frame, stepped(cur, start, end, step, keys.columns()).to_string());
        s.remove(CtxKey::FrameStep);
    }
}

/// The range playback runs in and the lock clamps to — the preview range
/// while it is on (`PSFRA` / `PEFRA`), else the scene's.
pub fn play_range(s: &BState) -> (i32, i32) {
    let start = int_or(s, CtxKey::FrameStart, FACTORY_START);
    let end = int_or(s, CtxKey::FrameEnd, FACTORY_END);
    if s.get(CtxKey::PreviewRange) == Some(V_TRUE) {
        let ps = int_or(s, CtxKey::PreviewStart, start);
        return (ps, int_or(s, CtxKey::PreviewEnd, end).max(ps));
    }
    (start, end.max(start))
}

/// `anim.change_frame` with `SCER_LOCK_FRAME_SELECTION`: the frame set is
/// kept inside the range.
pub struct BFrameLockRule;

impl BRule for BFrameLockRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == CtxKey::Frame.path()
    }
    fn after(&self, _: &BPath, _: &str, s: &mut BState) {
        if !settings::is_on(s, "lock_frame_selection_to_range") {
            return;
        }
        let (start, end) = play_range(s);
        let cur = int_or(s, CtxKey::Frame, start);
        let inside = cur.clamp(start, end);
        if inside != cur {
            s.set(CtxKey::Frame, inside.to_string());
        }
    }
}

/// Playback starting remembers the frame it started from
/// (`ScreenAnimData::sfra`), which Restore Frame goes back to.
pub struct BPlayFromRule;

impl BRule for BPlayFromRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == CtxKey::Playing.path()
    }
    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        if matches!(value, V_TRUE | V_REVERSE) && s.get(CtxKey::PlayFrom).is_none() {
            s.set(CtxKey::PlayFrom, int_or(s, CtxKey::Frame, FACTORY_START).to_string());
        } else if value == V_FALSE {
            s.remove(CtxKey::PlayFrom);
        }
    }
}

/// One tick of playback, `frames` on from the current frame (negative in
/// reverse) — `screen_animation_step` at the end of the range: Infinite
/// jumps back to the other end, Stop at End / Start Frame stop there,
/// Restore Frame stops where playback began, Bounce turns round.
pub fn tick(s: &mut BState, frames: i32) {
    let (start, end) = play_range(s);
    let cur = int_or(s, CtxKey::Frame, start);
    let reverse = s.get(CtxKey::Playing) == Some(V_REVERSE);
    let next = cur + frames;
    let over = if reverse { next < start } else { next > end };
    if !over {
        s.set(CtxKey::Frame, next.to_string());
        return;
    }
    let (near, far) = if reverse { (start, end) } else { (end, start) };
    let frame = match settings::loop_mode(s) {
        BlendLoopMode::Infinite => far,
        BlendLoopMode::StopEndFrame => near,
        BlendLoopMode::StopStartFrame => far,
        BlendLoopMode::Restore => int_or(s, CtxKey::PlayFrom, far),
        BlendLoopMode::Bounce => near,
    };
    match settings::loop_mode(s) {
        BlendLoopMode::Infinite => {}
        BlendLoopMode::Bounce => s.set(CtxKey::Playing, if reverse { V_TRUE } else { V_REVERSE }),
        _ => {
            s.set(CtxKey::Playing, V_FALSE);
            s.remove(CtxKey::PlayFrom);
        }
    }
    s.set(CtxKey::Frame, frame.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::state::val::BValue;
    use crate::v2::view::prim::BAction;

    fn playing(s: &mut BState, value: &str) {
        apply_action(s, &BAction::Set(CtxKey::Playing.path(), value.to_string()));
    }

    fn set_loop(s: &mut BState, mode: &str) {
        s.set(&settings::key(settings::LOOP_KEY), BValue::Enum(mode.to_string()).encode());
    }

    #[test]
    fn the_loop_modes_decide_what_the_end_of_the_range_does() {
        let mut s = BState::default();
        s.set(CtxKey::FrameStart, "1");
        s.set(CtxKey::FrameEnd, "10");
        s.set(CtxKey::Frame, "9");
        playing(&mut s, V_TRUE);
        assert_eq!(s.get(CtxKey::PlayFrom), Some("9"), "where playback began");
        tick(&mut s, 1);
        assert_eq!(s.get(CtxKey::Frame), Some("10"));
        tick(&mut s, 1);
        assert_eq!((s.get(CtxKey::Frame), s.get(CtxKey::Playing)), (Some("1"), Some(V_TRUE)), "Infinite: round to the start, still playing");
        set_loop(&mut s, "STOP_END_FRAME");
        s.set(CtxKey::Frame, "9");
        tick(&mut s, 3);
        assert_eq!((s.get(CtxKey::Frame), s.get(CtxKey::Playing)), (Some("10"), Some(V_FALSE)), "stops at the end");
        set_loop(&mut s, "STOP_START_FRAME");
        s.set(CtxKey::Frame, "9");
        playing(&mut s, V_TRUE);
        tick(&mut s, 3);
        assert_eq!((s.get(CtxKey::Frame), s.get(CtxKey::Playing)), (Some("1"), Some(V_FALSE)), "back to the start, stopped");
        set_loop(&mut s, "RESTORE");
        s.set(CtxKey::Frame, "6");
        playing(&mut s, V_TRUE);
        s.set(CtxKey::Frame, "9");
        tick(&mut s, 3);
        assert_eq!((s.get(CtxKey::Frame), s.get(CtxKey::Playing)), (Some("6"), Some(V_FALSE)), "where it began, stopped");
        assert_eq!(s.get(CtxKey::PlayFrom), None);
        set_loop(&mut s, "BOUNCE");
        s.set(CtxKey::Frame, "9");
        playing(&mut s, V_TRUE);
        tick(&mut s, 3);
        assert_eq!((s.get(CtxKey::Frame), s.get(CtxKey::Playing)), (Some("10"), Some(V_REVERSE)), "turns round at the end");
        tick(&mut s, -1);
        assert_eq!(s.get(CtxKey::Frame), Some("9"));
        s.set(CtxKey::Frame, "1");
        tick(&mut s, -1);
        assert_eq!((s.get(CtxKey::Frame), s.get(CtxKey::Playing)), (Some("1"), Some(V_TRUE)), "and at the start");
        // The preview range is the range while it is on.
        s.set(CtxKey::PreviewRange, V_TRUE);
        s.set(CtxKey::PreviewStart, "3");
        s.set(CtxKey::PreviewEnd, "5");
        assert_eq!(play_range(&s), (3, 5));
    }

    #[test]
    fn limit_to_frame_range_keeps_a_scrub_inside_the_range() {
        let mut s = BState::default();
        s.set(CtxKey::FrameStart, "10");
        s.set(CtxKey::FrameEnd, "20");
        apply_action(&mut s, &BAction::Set(CtxKey::Frame.path(), "40".into()));
        assert_eq!(s.get(CtxKey::Frame), Some("40"), "unlocked: anywhere");
        s.set(&settings::key("lock_frame_selection_to_range"), BValue::Bool(true).encode());
        apply_action(&mut s, &BAction::Set(CtxKey::Frame.path(), "40".into()));
        assert_eq!(s.get(CtxKey::Frame), Some("20"));
        apply_action(&mut s, &BAction::Set(CtxKey::Frame.path(), "2".into()));
        assert_eq!(s.get(CtxKey::Frame), Some("10"));
        apply_action(&mut s, &BAction::Set(CtxKey::Frame.path(), "15".into()));
        assert_eq!(s.get(CtxKey::Frame), Some("15"));
    }

    #[test]
    fn a_keyframe_jump_lands_on_the_next_column_and_stops_at_the_last() {
        let keys = [0.0, 10.0, 20.0, 132.0];
        assert_eq!(stepped(1, 1, 140, "key+", &keys), 10);
        assert_eq!(stepped(10, 1, 140, "key+", &keys), 20);
        assert_eq!(stepped(132, 1, 140, "key+", &keys), 132, "no key after the last: stays");
        assert_eq!(stepped(15, 1, 140, "key-", &keys), 10);
        assert_eq!(stepped(0, 1, 140, "key-", &keys), 0, "no key before the first: stays");
        assert_eq!(stepped(5, 1, 140, "key+", &[]), 5, "nothing keyed: stays");
    }
}
