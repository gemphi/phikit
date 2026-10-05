//! What a track looks like: the groove, how full it is, and its read-out.
//!
//! These parts were `BCtrlPart::Value` and `BCtrlPart::Bar` in `ctrl/mod.rs`,
//! where the slider was their only user. A bar belongs to the track now and the
//! slider is one thing that has one, so there is a single description of what a
//! track looks like and a single place that wires the gesture to it.

use super::{BTrack, draggable};
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::draw::ctx::BRenderCtx;
use crate::v2::view::draw::flex::{
    BAlign, BDim, BEdge, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BRadius,
};
use gpui::prelude::*;

/// The parts of a track.
enum BTrackPart {
    /// The groove the value runs along: a field-shaped well, clipped.
    Groove,
    /// The filled part, `share` of the groove.
    Fill { share: f32 },
}

impl BFlexed for BTrackPart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match self {
            Self::Groove => BFlex::row()
                .fills()
                .min_width(BLen::Zero)
                .align(BAlign::Center)
                .justify(BJustify::Center)
                .height(BLen::Dim(BDim::ButtonHeight))
                .radius(BRadius::Role(BStyleRole::Field))
                .border(BEdge::All)
                .overflow(BOverflow::Clip),
            Self::Fill { share } => BFlex::cell()
                .width(BLen::Share(*share))
                .height(BLen::Full)
                .fill(BFill::Tone(BSurfaceTone::Header)),
        }
    }
}

/// A track, drawn and draggable: the groove, how full it is, and its read-out.
pub fn view(track: &BTrack, share: f32, text: &str, ctx: &BRenderCtx) -> gpui::AnyElement {
    let id = gpui::SharedString::from(format!("track-{}", track.key.encode()));
    let groove = BTrackPart::Groove
        .flex(ctx)
        .div(ctx)
        .id(id.clone())
        .debug_selector({
            let id = id;
            move || id.to_string()
        });
    draggable(groove, track.clone(), ctx)
        .child(BTrackPart::Fill { share }.flex(ctx).div(ctx))
        .child(gpui::SharedString::from(text.to_string()))
        .into_any_element()
}

/// How full a track reading `value` is, 0..1. Flat when the range is empty, so a
/// degenerate slider draws as empty rather than dividing by nothing.
pub fn share_of(value: f32, min: f32, max: f32) -> f32 {
    if max > min { ((value - min) / (max - min)).clamp(0.0, 1.0) } else { 0.0 }
}

