//! The NLA editor's strips — `nla_draw.cc`: where a strip's box, its
//! extend backdrops and its repeat lines fall on the track, in frames.
//! Structure: the geometry of one track's strips ([`BStripBox`],
//! [`BTrackStrips`]). Painting them, and the action line, is `set.rs`.

pub mod set;

use crate::phiphx::blend::{BNlaStrip, BNlaTrack, EXTEND_HOLD, EXTEND_HOLD_FORWARD, STRIP_CLIP};

/// `nla_draw_strip`: the faint backdrop before the first strip and the
/// less faint one after each strip, when it holds.
pub const HOLD_BEFORE_ALPHA: f32 = 0.15;
pub const HOLD_AFTER_ALPHA: f32 = 0.3;
/// A strip on a muted track, or a muted strip: its inside at a tenth.
pub const MUTED_ALPHA: f32 = 0.1;
/// `draw_nla_main_data`: the name keeps 8 px off the strip's ends.
pub const TEXT_MARGIN: f32 = 8.0;
/// Repeat lines stop 4 px short of the strip's top and bottom.
pub const REPEAT_INSET: f32 = 4.0;

/// One strip on a track, with the frames its backdrops reach.
#[derive(Clone, Debug, PartialEq)]
pub struct BStripBox<'a> {
    pub strip: &'a BNlaStrip,
    /// `NLASTRIP_EXTEND_HOLD` on the first strip: held back to this frame.
    pub held_before: Option<f32>,
    /// Held forward to this frame when nothing follows at once.
    pub held_after: Option<f32>,
    /// Whole repeats inside the strip, as frames of a dividing line.
    pub repeats: Vec<f32>,
}

/// A track's strips laid along the view `first ..= last`.
pub struct BTrackStrips<'a> {
    pub boxes: Vec<BStripBox<'a>>,
}

impl<'a> BTrackStrips<'a> {
    pub fn of(track: &'a BNlaTrack, first: f32, last: f32) -> Self {
        let strips = &track.strips;
        let boxes = strips
            .iter()
            .enumerate()
            .map(|(index, strip)| {
                let next = strips.get(index + 1);
                let holds = matches!(strip.extend, EXTEND_HOLD | EXTEND_HOLD_FORWARD);
                let held_before = (strip.extend == EXTEND_HOLD && index == 0).then_some(first);
                let held_after = match next {
                    _ if !holds => None,
                    Some(next) if (next.start - strip.end).abs() < f32::EPSILON => None,
                    Some(next) => Some(next.start),
                    None => Some(last),
                };
                BStripBox { strip, held_before, held_after, repeats: Self::repeats(strip) }
            })
            .collect();
        Self { boxes }
    }

    /// `repeatLen = (actend − actstart) × scale`: a line at each whole
    /// repeat that falls inside the strip.
    fn repeats(strip: &BNlaStrip) -> Vec<f32> {
        if strip.kind != STRIP_CLIP || strip.repeat <= 1.0 {
            return Vec::new();
        }
        let length = (strip.actend - strip.actstart) * strip.scale;
        (1..strip.repeat.floor() as i32 + 1)
            .map(|i| strip.start + length * i as f32)
            .filter(|frame| *frame < strip.end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::EXTEND_NOTHING;

    fn strip(start: f32, end: f32, repeat: f32, extend: u32) -> BNlaStrip {
        BNlaStrip { name: "Walk".into(), start, end, actstart: 1.0, actend: 49.0, repeat, scale: 1.0, kind: STRIP_CLIP, extend, ..Default::default() }
    }

    #[test]
    fn strips_hold_to_the_view_edges_and_mark_their_repeats() {
        let track = BNlaTrack { name: "Base".into(), flag: 0, strips: vec![strip(1.0, 97.0, 2.0, EXTEND_HOLD), strip(97.0, 145.0, 1.0, EXTEND_HOLD), strip(200.0, 248.0, 1.0, EXTEND_NOTHING)] };
        let laid = BTrackStrips::of(&track, -9.0, 260.0);
        assert_eq!(laid.boxes[0].held_before, Some(-9.0), "the first strip holds back to the view's start");
        assert_eq!(laid.boxes[0].held_after, None, "the next strip starts where this ends");
        assert_eq!(laid.boxes[0].repeats, vec![49.0], "one whole repeat inside 1 .. 97");
        assert_eq!(laid.boxes[1].held_before, None);
        assert_eq!(laid.boxes[1].held_after, Some(200.0), "held forward to the next strip");
        assert_eq!(laid.boxes[2].held_after, None, "`EXTEND_NOTHING` holds nowhere");
        let lone = BNlaTrack { name: "Lone".into(), flag: 0, strips: vec![strip(10.0, 58.0, 1.0, EXTEND_HOLD_FORWARD)] };
        let laid = BTrackStrips::of(&lone, 0.0, 250.0);
        assert_eq!((laid.boxes[0].held_before, laid.boxes[0].held_after), (None, Some(250.0)));
    }
}
