//! `BTimeItem` — the frame ruler + transport toolbar + channel body.
//!
//! Replaces the placeholder `BView::canvas(BHostType::Time)` with a real
//! declarative layout:
//!
//!   ┌─ ruler toolbar ────────────────────────────────────────────────┐
//!   │  |◀  ◀  ▶/⏸  ▶  ▶|   frame: 1                               │
//!   └────────────────────────────────────────────────────────────────┘
//!   ┌─ channel body ─────────────────────────────────────────────────┐
//!   │  Cube.location                                                  │
//!   │  Cube.rotation                                                  │
//!   │  Camera.location                                                │
//!   └────────────────────────────────────────────────────────────────┘
//!
//! State keys:
//!   `ctx.time.playing` — `"true"` while the animation is playing.
//!   `ctx.time.frame` — current frame number as a decimal string; default "1".
//!   `ctx.time.frame_step` — pending relative frame step.

use crate::v2::host::BHostType;
use crate::v2::part::channels::set::channels_for;
use crate::v2::state::{BPath, BState, val::V_TRUE};
use crate::v2::view::{BAction, BPanelRole, BView, BViewItem};

fn k_playing() -> BPath {
    BPath::from_static(&["time", "playing"])
}

fn k_frame() -> BPath {
    BPath::from_static(&["time", "frame"])
}

fn k_frame_step() -> BPath {
    BPath::from_static(&["time", "frame_step"])
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BTimeItem;

impl BTimeItem {
    pub fn new() -> Self {
        Self
    }

    fn ruler_toolbar(s: &BState) -> BView {
        let playing = s.get(&k_playing()) == Some(V_TRUE);
        let frame_text = s.get(&k_frame()).unwrap_or("1").to_string();

        // Transport buttons: skip-back, step-back, play/pause, step-fwd, skip-fwd.
        let skip_back = BView::button_action("skip_back", "", BAction::Set(k_frame(), "1".into()));
        let step_back =
            BView::button_action("arrow_left", "", BAction::Set(k_frame_step(), "-1".into()));
        let play_pause = BView::button_action(
            if playing { "pause" } else { "play" },
            "",
            BAction::Toggle(k_playing()),
        );
        let step_fwd =
            BView::button_action("arrow_right", "", BAction::Set(k_frame_step(), "+1".into()));
        let skip_fwd =
            BView::button_action("skip_forward", "", BAction::Set(k_frame(), "999999".into()));
        let frame_label = BView::label(frame_text);

        BView::panel(
            BPanelRole::Toolbar,
            vec![BView::row(vec![
                skip_back,
                step_back,
                play_pause,
                step_fwd,
                skip_fwd,
                frame_label,
            ])],
        )
    }

    fn channel_body(_s: &BState) -> BView {
        let channels = channels_for(BHostType::Time);
        let rows: Vec<BView> = channels.iter().map(|ch| BView::label(ch.label)).collect();
        BView::panel(BPanelRole::Body, rows)
    }
}

impl BViewItem for BTimeItem {
    fn build(&self, s: &BState) -> BView {
        BView::column(vec![Self::ruler_toolbar(s), Self::channel_body(s)])
    }
}

#[cfg(test)]
mod tests {
    use super::{k_frame, k_frame_step, k_playing};

    #[test]
    fn timeline_keys_encode_to_scoped_paths() {
        assert_eq!(k_playing().encode(), "ctx.time.playing");
        assert_eq!(k_frame().encode(), "ctx.time.frame");
        assert_eq!(k_frame_step().encode(), "ctx.time.frame_step");
    }
}
