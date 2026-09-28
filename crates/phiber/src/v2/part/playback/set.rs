//! The transport buttons, in Blender's order — `playback_controls`:
//! auto-key, jump to start, previous keyframe, play reverse, play (or
//! pause while playing), next keyframe, jump to end, then the time jump
//! back / forward by `time_jump_delta` in `time_jump_unit`, whose defaults
//! are one second (`DNA_scene_types.h`: `time_jump_delta = 1.0`,
//! `time_jump_unit = SCE_TIME_JUMP_SECOND`).

use crate::v2::state::key::float_or;
use crate::v2::state::{BKey as _, BState, CtxKey, val::{V_FALSE, V_REVERSE, V_TRUE}};
use crate::v2::view::prim::{BAction, BControl};
use crate::v2::view::BView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BTransportButton {
    /// `tool_settings.use_keyframe_insert_auto` — the record toggle.
    AutoKey,
    /// `screen.frame_jump` (end = False).
    JumpStart,
    /// `screen.keyframe_jump` (next = False).
    PrevKey,
    /// `screen.animation_play` (reverse = True).
    PlayReverse,
    /// `screen.animation_play`.
    Play,
    /// `screen.animation_pause`: while playing, one button two slots wide
    /// stands where the two play buttons were (`playback_controls`,
    /// `row.scale_x = 2`).
    Pause,
    /// `screen.keyframe_jump` (next = True).
    NextKey,
    /// `screen.frame_jump` (end = True).
    JumpEnd,
    /// `screen.time_jump` (backward = True).
    TimeBack,
    /// `screen.time_jump` (backward = False).
    TimeForward,
}

impl BTransportButton {
    pub const ALL: [Self; 10] = [
        Self::AutoKey,
        Self::JumpStart,
        Self::PrevKey,
        Self::PlayReverse,
        Self::Play,
        Self::Pause,
        Self::NextKey,
        Self::JumpEnd,
        Self::TimeBack,
        Self::TimeForward,
    ];

    /// The buttons the header shows: the two play buttons while stopped,
    /// the one pause button while playing (`playback_controls`).
    pub fn shown(s: &BState) -> impl Iterator<Item = Self> {
        let playing = Self::playing(s);
        Self::ALL.into_iter().filter(move |button| match button {
            Self::PlayReverse | Self::Play => !playing,
            Self::Pause => playing,
            _ => true,
        })
    }

    /// The icon Blender gives the button (`REW`, `PREV_KEYFRAME`, …).
    pub fn icon(self, _s: &BState) -> &'static str {
        match self {
            Self::AutoKey => "record",
            Self::JumpStart => "skip_start",
            Self::PrevKey => "keyframe_prev",
            Self::PlayReverse => "play_back",
            Self::Play => "play",
            Self::Pause => "pause",
            Self::NextKey => "keyframe_next",
            Self::JumpEnd => "skip_end",
            Self::TimeBack => "chevron_left",
            Self::TimeForward => "chevron_right",
        }
    }

    pub fn tooltip(self) -> &'static str {
        match self {
            Self::AutoKey => "Auto Keying",
            Self::JumpStart => "Jump to Endpoint",
            Self::PrevKey => "Jump to Keyframe",
            Self::PlayReverse => "Play Animation (reverse)",
            Self::Pause => "Pause Animation",
            Self::Play => "Play Animation",
            Self::NextKey => "Jump to Keyframe",
            Self::JumpEnd => "Jump to Endpoint",
            Self::TimeBack => "Jump Back in Time",
            Self::TimeForward => "Jump Forward in Time",
        }
    }

    /// Lit while its state is on: the record toggle, and pause while playing.
    pub fn active(self, s: &BState) -> bool {
        match self {
            Self::AutoKey => s.get(CtxKey::AutoKey) == Some(V_TRUE),
            Self::Pause => Self::playing(s),
            _ => false,
        }
    }

    /// What pressing it writes.
    pub fn action(self, s: &BState) -> BAction {
        let step = |intent: String| BAction::Set(CtxKey::FrameStep.path(), intent);
        let second = float_or(s, CtxKey::Fps, 24.0).round().max(1.0) as i32;
        match self {
            Self::AutoKey => BAction::Toggle(CtxKey::AutoKey.path()),
            Self::JumpStart => step("start".into()),
            Self::PrevKey => step("key-".into()),
            // `screen.animation_play`: the ticker runs forwards or, with
            // `reverse`, backwards (`app/playback`); pause stops it.
            Self::PlayReverse => BAction::Set(CtxKey::Playing.path(), V_REVERSE.to_string()),
            Self::Play => BAction::Set(CtxKey::Playing.path(), V_TRUE.to_string()),
            Self::Pause => BAction::Set(CtxKey::Playing.path(), V_FALSE.to_string()),
            Self::NextKey => step("key+".into()),
            Self::JumpEnd => step("end".into()),
            Self::TimeBack => step(format!("-{second}")),
            Self::TimeForward => step(format!("+{second}")),
        }
    }

    pub fn view(self, s: &BState) -> BView {
        BView::Button(
            BControl::new()
                .with_icon(self.icon(s))
                .with_tooltip(self.tooltip())
                .with_active(self.active(s))
                .with_action(self.action(s))
                .with_span(if self == Self::Pause { 2 } else { 1 }),
        )
    }

    /// Playing either way (`ctx.playing` is `true` or `reverse`).
    pub fn playing(s: &BState) -> bool {
        matches!(s.get(CtxKey::Playing), Some(V_TRUE) | Some(V_REVERSE))
    }

    /// The frame step the ticker applies each tick: forwards, or backwards
    /// while playing in reverse (`screen_animation_step`, `ANIMPLAY_FLAG_REVERSE`).
    pub fn tick_step(s: &BState) -> Option<&'static str> {
        match s.get(CtxKey::Playing) {
            Some(V_TRUE) => Some("+1"),
            Some(V_REVERSE) => Some("-1"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;

    #[test]
    fn the_time_jump_is_one_second_of_the_scene_fps() {
        let mut s = BState::default();
        s.set(CtxKey::Fps, "30");
        s.set(CtxKey::FrameStart, "1");
        s.set(CtxKey::FrameEnd, "250");
        s.set(CtxKey::Frame, "100");
        let forward = BTransportButton::TimeForward.action(&s);
        apply_action(&mut s, &forward);
        assert_eq!(s.get(CtxKey::Frame), Some("130"));
        let back = BTransportButton::TimeBack.action(&s);
        apply_action(&mut s, &back);
        assert_eq!(s.get(CtxKey::Frame), Some("100"));
    }

    #[test]
    fn playing_swaps_the_two_play_buttons_for_one_wide_pause_and_record_lights_when_on() {
        let mut s = BState::default();
        let shown = |s: &BState| BTransportButton::shown(s).collect::<Vec<_>>();
        assert!(shown(&s).contains(&BTransportButton::Play) && shown(&s).contains(&BTransportButton::PlayReverse));
        assert!(!shown(&s).contains(&BTransportButton::Pause));
        let pressed = BTransportButton::Play.action(&s);
        apply_action(&mut s, &pressed);
        assert_eq!(BTransportButton::tick_step(&s), Some("+1"));
        assert_eq!(shown(&s).iter().filter(|b| matches!(b, BTransportButton::Play | BTransportButton::PlayReverse)).count(), 0);
        assert!(shown(&s).contains(&BTransportButton::Pause) && BTransportButton::Pause.active(&s));
        let BView::Button(pause) = BTransportButton::Pause.view(&s) else { panic!() };
        assert_eq!(pause.span, 2, "the pause stands in for both play buttons");
        let pressed = BTransportButton::Pause.action(&s);
        apply_action(&mut s, &pressed);
        assert_eq!(BTransportButton::tick_step(&s), None);
        let pressed = BTransportButton::PlayReverse.action(&s);
        apply_action(&mut s, &pressed);
        assert_eq!(BTransportButton::tick_step(&s), Some("-1"), "reverse runs the ticker backwards");
        assert!(BTransportButton::playing(&s));
        assert!(!BTransportButton::AutoKey.active(&s));
        let record = BTransportButton::AutoKey.action(&s);
        apply_action(&mut s, &record);
        assert!(BTransportButton::AutoKey.active(&s));
    }
}
