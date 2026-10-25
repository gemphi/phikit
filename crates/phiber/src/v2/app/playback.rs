//! Playback ticker (Blender `screen.animation_play`): while `CtxKey::Playing.path()` is
//! `true` or `reverse`, step `CtxKey::Frame.path()` forwards or backwards
//! at the scene's fps (looping through the reducer's frame-step rule,
//! `screen_animation_step` with `ANIMPLAY_FLAG_REVERSE`) and redraw. One ticker at a time.
//!
//! The ticker keeps time, not ticks: each frame is due `1 / fps` after the
//! last, and a tick that comes late — the viewport was busy re-posing the
//! scene — steps as many frames as fell due, so a second of playback is a
//! second of animation (Blender's default sync, `AUDIO_SYNC`, drops frames
//! the same way). The fps is read every tick, so changing it mid-play takes.

use crate::v2::state::CtxKey;
use super::BPhiberApp;
use crate::v2::state::key::{float_or};
use crate::v2::part::playback::set::BTransportButton;
use gpui::Context;
use std::time::Duration;
use web_time::Instant;

/// The most frames one late tick may catch up, so a long stall does not
/// spin the playhead round the range.
const MAX_CATCH_UP: i32 = 12;

/// How many frames fall due by `now`, and when the next one is due after
/// them, given the frame `due` was owed at and the frame period.
pub fn frames_due(due: Instant, now: Instant, period: Duration) -> (i32, Instant) {
    let late = now.saturating_duration_since(due).as_secs_f32();
    let whole = (late / period.as_secs_f32()).floor() as i32;
    if whole >= MAX_CATCH_UP - 1 {
        // A stall: catch up the cap and start the grid again from now.
        return (MAX_CATCH_UP, now + period);
    }
    (1 + whole, due + period * (1 + whole) as u32)
}

impl BPhiberApp {
    pub(crate) fn ensure_playback(&mut self, cx: &mut Context<Self>) {
        if self.playing_ticker || !BTransportButton::playing(&self.state) {
            return;
        }
        self.playing_ticker = true;
        let mut due = Instant::now() + period_of(float_or(&self.state, CtxKey::Fps, 24.0));
        cx.spawn(async move |this, cx| loop {
            let now = Instant::now();
            if due > now {
                cx.background_executor().timer(due - now).await;
            }
            let go_on = this
                .update(cx, |app, cx| {
                    // Forwards, or backwards while playing in reverse.
                    let Some(step) = BTransportButton::tick_step(&app.state) else {
                        app.playing_ticker = false;
                        return false;
                    };
                    let period = period_of(float_or(&app.state, CtxKey::Fps, 24.0));
                    let (frames, next) = frames_due(due, Instant::now(), period);
                    due = next;
                    let signed = if step.starts_with('-') { -frames } else { frames };
                    // `screen_animation_step`: the loop mode decides what
                    // the end of the range does (`part/time/step.rs`).
                    crate::v2::part::time::step::tick(&mut app.state, signed);
                    cx.notify();
                    true
                })
                .unwrap_or(false);
            if !go_on {
                break;
            }
        })
        .detach();
    }
}

/// One frame's duration at `fps`.
fn period_of(fps: f32) -> Duration {
    Duration::from_secs_f32(1.0 / fps.clamp(1.0, 240.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tick_on_time_is_one_frame_and_a_late_one_catches_up() {
        let period = Duration::from_millis(40);
        let due = Instant::now();
        assert_eq!(frames_due(due, due, period).0, 1);
        assert_eq!(frames_due(due, due + Duration::from_millis(39), period).0, 1);
        let (frames, next) = frames_due(due, due + Duration::from_millis(100), period);
        assert_eq!(frames, 3, "two whole frames fell due late, plus the one owed");
        assert_eq!(next, due + Duration::from_millis(120), "the next frame is due on the grid, not from now");
        let stalled = due + Duration::from_secs(10);
        assert_eq!(frames_due(due, stalled, period), (MAX_CATCH_UP, stalled + period), "a stall is capped and the grid restarts");
    }
}

/// How often the refining viewport is asked whether a frame landed.
const RAY_POLL: Duration = Duration::from_millis(8);

impl BPhiberApp {
    /// Whether the window is being drawn by that task (a test reads it).
    pub fn draws_on_frames(&self) -> bool {
        self.ray_ticker
    }

    /// While the viewport's image refines, or a final render runs, a task
    /// draws the window when a frame lands (`rendered::take_fresh`) and ends
    /// when the image has settled — in place of a draw every vsync while
    /// waiting.
    pub(crate) fn ensure_ray_ticker(&mut self, cx: &mut Context<Self>) {
        use crate::v2::view::draw::canvas::view3d::rendered::{refining, take_fresh};
        let busy = || refining() || crate::v2::scene::render_job::rendering();
        if self.ray_ticker || !busy() {
            return;
        }
        self.ray_ticker = true;
        crate::v2::view::draw::canvas::view3d::rendered::set_ticking(true);
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(RAY_POLL).await;
            let go_on = this.update(cx, |app, cx| {
                let fresh = take_fresh();
                if fresh {
                    cx.notify();
                }
                let on = fresh || busy();
                app.ray_ticker = on;
                crate::v2::view::draw::canvas::view3d::rendered::set_ticking(on);
                on
            });
            if !go_on.unwrap_or(false) {
                crate::v2::view::draw::canvas::view3d::rendered::set_ticking(false);
                break;
            }
        })
        .detach();
    }
}
