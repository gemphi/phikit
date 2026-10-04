//! The body of a colour control's picker: one row per channel.
//!
//! A colour control was a swatch and a text field, and the swatch was inert: the
//! only way to change a theme colour was to type four numbers into the field
//! beside it, in the exact spelling the descriptor accepts. Clicking the colour
//! did nothing, which is not what a swatch looks like it does.
//!
//! This is the picker that opens instead: hue, saturation, lightness and alpha,
//! each a bar you sweep (`BView::Track`) with a step either side for the fine
//! adjustment a drag is bad at — the pair Blender's own picker shows. Every write
//! carries the whole colour with that one channel moved, in the spelling the
//! descriptor parses back.
//!
//! It is `BView` and `BControl` throughout rather than renderer internals, so it
//! composes the same atoms every other control does. The saturation/lightness
//! square leads the rows (`BPlane::saturation_lightness`, Blender's `GRAD_SV`).
//! Blender's own default is a hue/saturation *ring* instead, and not having one is
//! a decision rather than a gap — `BPlane::saturation_lightness` records why.

mod channel;
mod hex;

use self::channel::rows as channel_rows;
use self::hex::row as hex_row;
use super::swatch::BCtrlSwatch;
use crate::v2::state::{BPath, BState};
use crate::v2::theme::BColor;
use crate::v2::view::prim::{BAction, BControl, BCtrl, BPopoverSide};
use crate::v2::view::BView;

/// The whole picker for `ctrl`: the swatch that opens it, and the channels
/// inside. `None` when there is nothing to open — a colour that could not be
/// read, or a control with nowhere to record being open.
pub fn popover(ctrl: &BCtrl, swatch: BCtrlSwatch, s: &BState) -> Option<BView> {
    let (Some(open_key), BCtrlSwatch::Hsl(color)) = (ctrl.open_key.clone(), swatch) else {
        return None;
    };
    let anchor = BView::Button(
        BControl::new()
            .with_active(ctrl.open)
            .with_action(BAction::Toggle(open_key.clone()))
            .with_swatch(color),
    );
    Some(BView::popover(
        anchor,
        rows(&ctrl.key, color, s),
        ctrl.open,
        BAction::Reset(open_key),
        BPopoverSide::Below,
    ))
}

/// The picker's rows for `color`, each step writing `key`.
fn rows(key: &BPath, color: BColor, s: &BState) -> Vec<BView> {
    // The square first: saturation across, lightness down — Blender's `GRAD_SV` —
    // with hue on a bar below rather than inside it, because `colorpicker_square`
    // pairs a `GRAD_SV` square with `GRAD_H` (`gradient_type = type + 3`) and draws
    // that bar under it. The channel rows stay: a square is how you find a colour
    // and a number is how you say which one, and Blender shows both.
    let mut out = vec![BView::Plane {
        plane: Box::new(crate::v2::view::draw::BPlane::saturation_lightness(key.clone(), color)),
    }];
    out.extend(channel_rows(key, color));
    out.push(hex_row(key, color, s));
    out
}



#[cfg(test)]
mod tests {
    use super::*;

    fn actions(views: &[BView]) -> Vec<String> {
        fn walk(v: &BView, out: &mut Vec<String>) {
            match v {
                BView::Button(ctl) => {
                    if let Some(BAction::Adjust(_, value)) = &ctl.action {
                        out.push(value.clone());
                    }
                }
                BView::Row { list } => list.iter().for_each(|c| walk(c, out)),
                _ => {}
            }
        }
        let mut out = Vec::new();
        views.iter().for_each(|v| walk(v, &mut out));
        out
    }

    #[test]
    fn a_step_writes_the_whole_colour_with_one_channel_moved() {
        let key = BPath::owned(vec!["ui_style", "colors", "bg_app"]);
        let views = rows(&key, (200.0, 50.0, 40.0, 1.0), &BState::new());
        let written = actions(&views);
        // Four channels, a step either side of each.
        assert_eq!(written.len(), 8);
        // Hue down and up; the other three channels ride along unchanged.
        assert!(written.contains(&"199, 50, 40, 1".to_string()), "{written:?}");
        assert!(written.contains(&"201, 50, 40, 1".to_string()), "{written:?}");
        // And alpha moves by its own step, not hue's.
        assert!(written.contains(&"200, 50, 40, 0.95".to_string()), "{written:?}");
    }

    #[test]
    fn every_write_is_a_colour_the_descriptor_would_accept() {
        let key = BPath::owned(vec!["ui_style", "colors", "bg_app"]);
        for color in [(0.0, 0.0, 0.0, 0.0), (360.0, 100.0, 100.0, 1.0), (228.94, 24.36, 6.12, 1.0)] {
            for written in actions(&rows(&key, color, &BState::new())) {
                assert!(
                    crate::v2::theme::parse_hsla(&written).is_ok(),
                    "the picker wrote {written:?}, which the field beside it would reject"
                );
            }
        }
    }

    #[test]
    fn a_channel_at_its_end_does_not_offer_to_write_what_it_holds() {
        let key = BPath::owned(vec!["ui_style", "colors", "bg_app"]);
        // Hue at 0 and alpha at 0: stepping down is a no-op either way, and a
        // button that writes the current value looks live and does nothing.
        let views = rows(&key, (0.0, 50.0, 40.0, 0.0), &BState::new());
        let mut muted = 0;
        fn walk(v: &BView, muted: &mut usize) {
            match v {
                BView::Button(ctl) if ctl.muted => *muted += 1,
                BView::Row { list } => list.iter().for_each(|c| walk(c, muted)),
                _ => {}
            }
        }
        views.iter().for_each(|v| walk(v, &mut muted));
        assert_eq!(muted, 2, "hue's and alpha's down-steps are the two with nowhere to go");
    }

    #[test]
    fn each_channel_is_a_track_over_its_own_range() {
        // What `BView::Track` was for. A popover describes its body as `BView` and
        // a gesture needs an element, so until the walker could render a track
        // from the tree these rows could only ever be buttons.
        use crate::v2::view::draw::BTrackWrite;

        let key = BPath::owned(vec!["ui_style", "colors", "bg_app"]);
        let views = rows(&key, (180.0, 50.0, 25.0, 0.5), &BState::new());
        let mut tracks = Vec::new();
        fn walk(v: &BView, out: &mut Vec<(Box<crate::v2::view::draw::BTrack>, f32)>) {
            match v {
                BView::Track { track, share, .. } => out.push((track.clone(), *share)),
                BView::Row { list } => list.iter().for_each(|c| walk(c, out)),
                _ => {}
            }
        }
        views.iter().for_each(|v| walk(v, &mut tracks));
        assert_eq!(tracks.len(), 4, "hue, saturation, lightness, alpha");

        // Over the bounds each channel actually has: degrees are not percent, and
        // neither is an alpha.
        let ranges: Vec<(f32, f32)> = tracks.iter().map(|(t, _)| (t.min, t.max)).collect();
        assert_eq!(ranges, vec![(0.0, 360.0), (0.0, 100.0), (0.0, 100.0), (0.0, 1.0)]);

        for (index, (track, share)) in tracks.iter().enumerate() {
            assert_eq!(track.key, key, "every channel writes the colour's own one key");
            assert!(
                matches!(&track.write, BTrackWrite::Channel { index: i, .. } if *i == index),
                "channel {index} writes channel {index}, got {:?}",
                track.write
            );
            assert!((0.0..=1.0).contains(share), "a share is a fraction, got {share}");
        }
        // And the shares are where those values sit: half of 360, half of 100,
        // a quarter of 100, half of 1.
        let shares: Vec<f32> = tracks.iter().map(|(_, s)| (s * 100.0).round() / 100.0).collect();
        assert_eq!(shares, vec![0.5, 0.5, 0.25, 0.5]);
    }

    #[test]
    fn the_picker_leads_with_a_square_over_the_two_channels_a_square_is_for() {
        use crate::v2::view::draw::BPlane;

        let key = BPath::owned(vec!["ui_style", "colors", "bg_app"]);
        let color = (210.0, 40.0, 60.0, 1.0);
        let views = rows(&key, color, &BState::new());
        let BView::Plane { plane } = &views[0] else {
            panic!("the square comes first, got {:?}", &views[0])
        };
        assert_eq!(plane.key, key, "it writes the colour's own key");
        // Blender's `GRAD_SV`: saturation across, lightness down, hue on the bar
        // under it (`GRAD_H`, which is `GRAD_SV + 3`).
        assert_eq!((plane.x.channel, plane.y.channel), (1, 2), "saturation across, lightness down");
        assert_eq!((plane.x.max, plane.y.max), (100.0, 100.0), "both are percent");
        // The pairing Blender does not ship: a square whose axes are hue and
        // saturation has a VALUE bar (`GRAD_HS + 3 == GRAD_V`), never a lightness
        // one — `GRAD_L_ALT` belongs to the circle alone. This square took those
        // axes for a round, so the guard names what it must not go back to.
        assert_ne!(plane.x.channel, 0, "hue is not an axis of this square");
        assert_ne!(plane.y.channel, 0, "hue is not an axis of this square");
        // Hue and alpha keep their own rows, because a square cannot show four
        // channels and pretending otherwise is how a picker loses one.
        assert_eq!(views.len(), 6, "the square, four channel rows and Hex");
        assert_eq!(**plane, BPlane::saturation_lightness(key, color));
    }

    /// What the square shows is what dropping the pointer there writes.
    ///
    /// The reason it is a grid of cells rather than stacked gradients: a 2-stop
    /// linear gradient cannot express a saturation sweep crossed with a lightness
    /// one, so an approximated square would show one colour and set another.
    #[test]
    fn a_cell_is_the_colour_that_cell_sets() {
        let key = BPath::owned(vec!["c"]);
        let BView::Plane { plane } = &rows(&key, (120.0, 50.0, 50.0, 1.0), &BState::new())[0] else { panic!() };
        // A cell's fill is `color_at` of its own centre, and so is the write for a
        // pointer dropped there — same function, so they cannot disagree.
        for (across, down) in [(0.025, 0.025), (0.5, 0.5), (0.975, 0.975)] {
            let shown = plane.color_at(across, down);
            assert!(
                crate::v2::theme::parse_hsla(&crate::v2::view::draw::track::encode_hsla(shown)).is_ok(),
                "{shown:?} is a colour the field would accept"
            );
        }
    }


}
