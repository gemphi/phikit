//! Scrubbing view — timeline transport cluster (Blender space_action header).
//!
//! Renders the store's static `TRANSPORT` buttons and frame range, reading
//! `CtxFrame` / `CtxPlaying` state.

use super::super::bview::BAction;
use super::super::theme::BPalette;
use super::super::{BKey, BState, BView};
use crate::store::hosts::BHostType;
use crate::store::parts::scrubbing::set;

pub struct ScrubbingView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> ScrubbingView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let state = self.state;
        let p = BPalette::from_state(state);
        let frame = state.get(&BKey::CtxFrame).unwrap_or(set::FRAME_START);
        let playing = state.get(&BKey::CtxPlaying) == Some("true");

        let mut items: Vec<BView> = Vec::new();

        // Current frame readout (Blender: frame field left of transport).
        items.push(BView::label(frame));

        // Transport buttons from static store data.
        for t in set::TRANSPORT {
            let (action, active) = match t.idx {
                "jump_start" => (
                    BAction::Set(BKey::CtxFrame, set::FRAME_START.to_string()),
                    false,
                ),
                "jump_end" => (
                    BAction::Set(BKey::CtxFrame, set::FRAME_END.to_string()),
                    false,
                ),
                "play" | "play_reverse" => (
                    BAction::Toggle(BKey::CtxPlaying),
                    playing && t.idx == "play",
                ),
                _ => continue,
            };
            items.push(
                BView::button()
                    .with_icon(t.icon)
                    .with_tooltip(t.tip)
                    .with_active(active)
                    .with_action(action),
            );
        }

        // Frame range (Blender: Start / End fields right of transport).
        items.push(BView::vseparator());
        items.push(BView::label("Start"));
        items.push(BView::label(set::FRAME_START));
        items.push(BView::label("End"));
        items.push(BView::label(set::FRAME_END));

        BView::row(Some(p.gap_xs), items)
    }
}
