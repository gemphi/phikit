//! The four channels of an HSLA colour: what each is called, the bounds it has,
//! and how a colour is taken apart and put back together.
//!
//! Degrees are not percent and neither is an alpha, so a picker that treated them
//! alike would step hue by a hundredth of a degree or alpha by a whole unit. Each
//! channel states its own — and builds its own row, since what a channel is and
//! what its row looks like are the same subject.

use crate::v2::state::BPath;
use crate::v2::theme::BColor;
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

/// One channel of an HSLA colour: what it is called, its bounds, and the step a
/// press moves it by.
pub(super) struct BChannel {
    pub(super) label: &'static str,
    pub(super) min: f32,
    pub(super) max: f32,
    pub(super) step: f32,
    /// How the read-out is written, since degrees, percent and a 0..1 alpha do
    /// not want the same precision.
    pub(super) decimals: usize,
}

pub(super) const CHANNELS: [BChannel; 4] = [
    BChannel { label: "Hue", min: 0.0, max: 360.0, step: 1.0, decimals: 0 },
    BChannel { label: "Saturation", min: 0.0, max: 100.0, step: 1.0, decimals: 1 },
    BChannel { label: "Lightness", min: 0.0, max: 100.0, step: 1.0, decimals: 1 },
    BChannel { label: "Alpha", min: 0.0, max: 1.0, step: 0.05, decimals: 2 },
];

/// `color` written the way a `BStyleDescriptor` formats and parses it back.
///
/// The picker goes through this for every write, so a colour it sets is a colour
/// the field beside it would accept — the spelling is not re-invented here.
pub(super) fn encode(color: BColor) -> String {
    let (h, s, l, a) = color;
    format!("{h}, {s}, {l}, {a}")
}

/// `color` with channel `index` set to `value`.
pub(super) fn with_channel(color: BColor, index: usize, value: f32) -> BColor {
    let (h, s, l, a) = color;
    match index {
        0 => (value, s, l, a),
        1 => (h, value, l, a),
        2 => (h, s, value, a),
        _ => (h, s, l, value),
    }
}

pub(super) fn channel_of(color: BColor, index: usize) -> f32 {
    let (h, s, l, a) = color;
    [h, s, l, a][index.min(3)]
}

/// One row per channel: the label, a step either side, and the bar between them.
pub(super) fn rows(key: &BPath, color: BColor) -> Vec<BView> {
    CHANNELS
        .iter()
        .enumerate()
        .map(|(index, channel)| {
            let now = channel_of(color, index);
            let step = |icon: &str, by: f32| {
                let next = (now + by * channel.step).clamp(channel.min, channel.max);
                BView::Button(
                    BControl::new()
                        .with_icon(icon)
                        // A channel already at its end has nothing to write, so
                        // the button is there and not in force rather than
                        // silently writing the value it already holds.
                        .with_muted((next - now).abs() < f32::EPSILON)
                        // An adjustment, not a decision: the picker stays up for
                        // the next nudge instead of shutting after every one.
                        .with_action(BAction::Adjust(key.clone(), encode(with_channel(color, index, next)))),
                )
            };
            // The channel itself is a track you sweep, with a step either side for
            // the fine adjustment a drag is bad at — which is the pair Blender's
            // own colour picker shows. Until `BView::Track` existed this row was
            // steps and a read-out only, because a popover describes its body as
            // `BView` and a gesture needs an element.
            BView::row(vec![
                BView::Label { text: channel.label.to_string() },
                step("chevron_left", -1.0),
                BView::Track {
                    track: Box::new(crate::v2::view::draw::BTrack::channel(
                        key.clone(),
                        index,
                        color,
                        channel.min,
                        channel.max,
                    )),
                    share: crate::v2::view::draw::track::share_of(now, channel.min, channel.max),
                    text: format!("{now:.*}", channel.decimals),
                },
                step("chevron_right", 1.0),
            ])
        })
        .collect()
}
