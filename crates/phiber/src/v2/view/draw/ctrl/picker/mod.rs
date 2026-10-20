//! The body of a colour control's picker: Blender's wheel, then one row per
//! channel.
//!
//! A colour control was a swatch and a text field, and the swatch was inert: the
//! only way to change a theme colour was to type four numbers into the field
//! beside it, in the exact spelling the descriptor accepts. Clicking the colour
//! did nothing, which is not what a swatch looks like it does.
//!
//! This is the picker that opens instead, laid out as `block_colorpicker` lays
//! Blender's out: the hue/saturation wheel with its value bar
//! (`USER_CP_CIRCLE_HSV`, Blender's default — `track/wheel`), then a bar per
//! channel, then Hex. A theme colour's bars are hue, saturation, lightness and
//! alpha, each with a step either side for the fine adjustment a drag is bad at;
//! a scene colour's are red, green, blue and alpha, the RGB tab Blender opens on.
//! Every write carries the whole colour in the spelling its key was read in.
//!
//! It is `BView` and `BControl` throughout rather than renderer internals, so it
//! composes the same atoms every other control does.

mod channel;
mod hex;

use self::channel::rows as channel_rows;
use self::hex::row as hex_row;
use super::swatch::BCtrlSwatch;
use crate::v2::state::{BPath, BState, BValue};
use crate::v2::theme::BColor;
use crate::v2::view::draw::{BTrack, BWheel};
use crate::v2::view::prim::{BAction, BControl, BCtrl, BPopoverSide};
use crate::v2::view::BView;

/// The whole picker for `ctrl`: the swatch that opens it, and what is inside.
/// `None` when there is nothing to open — a colour that could not be read, or
/// a control with nowhere to record being open.
pub fn popover(ctrl: &BCtrl, swatch: BCtrlSwatch, s: &BState) -> Option<BView> {
    let open_key = ctrl.open_key.clone()?;
    let (face, body) = match (swatch, &ctrl.value) {
        (BCtrlSwatch::Hsl(color), _) => (color, rows(&ctrl.key, color, s)),
        // Only a value that IS a colour: the half-black "unreadable" swatch is
        // RGBA too, and a picker over it would write RGBA into a theme key.
        (BCtrlSwatch::Rgba(rgba), BValue::Color(_)) => {
            (crate::v2::view::draw::track::rgb_to_hsla([rgba[0], rgba[1], rgba[2]], rgba[3]), rgba_rows(&ctrl.key, rgba))
        }
        _ => return None,
    };
    let anchor = BView::Button(
        BControl::new()
            .with_active(ctrl.open)
            .with_action(BAction::Toggle(open_key.clone()))
            .with_swatch(face),
    );
    Some(BView::popover(anchor, body, ctrl.open, BAction::Reset(open_key), BPopoverSide::Below))
}

/// A theme colour's picker: the wheel, its four HSLA channels, and Hex.
fn rows(key: &BPath, color: BColor, s: &BState) -> Vec<BView> {
    let mut out = vec![BView::Wheel { wheel: Box::new(BWheel::of_hsl(key.clone(), color)) }];
    out.extend(channel_rows(key, color));
    out.push(hex_row(key, color, s));
    out
}

/// A scene colour's picker: the wheel, then red, green, blue and alpha.
fn rgba_rows(key: &BPath, rgba: [f32; 4]) -> Vec<BView> {
    let mut out = vec![BView::Wheel { wheel: Box::new(BWheel::of_rgba(key.clone(), rgba)) }];
    out.extend(["R", "G", "B", "A"].iter().enumerate().map(|(index, label)| {
        let now = rgba[index];
        BView::row(vec![
            BView::Label { text: label.to_string() },
            BView::Track {
                track: Box::new(BTrack::rgba(key.clone(), index, rgba)),
                share: crate::v2::view::draw::track::share_of(now, 0.0, 1.0),
                text: format!("{now:.3}"),
            },
        ])
    }));
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
    fn the_picker_leads_with_blenders_wheel() {
        let key = BPath::owned(vec!["ui_style", "colors", "bg_app"]);
        let views = rows(&key, (210.0, 40.0, 60.0, 1.0), &BState::new());
        let BView::Wheel { wheel } = &views[0] else { panic!("the wheel comes first, got {:?}", &views[0]) };
        assert_eq!(wheel.key, key, "it writes the colour's own key");
        assert_eq!(wheel.space, crate::v2::view::draw::BPickSpace::Hsl);
        assert_eq!(views.len(), 6, "the wheel, four channel rows and Hex");
    }

    #[test]
    fn a_scene_colour_opens_a_picker_that_writes_rgba() {
        use crate::v2::state::BValue;
        use crate::v2::view::prim::BCtrlKind;
        let key = BPath::owned(vec!["buts", "value", "color"]);
        let s = BState::new();
        let ctrl = BCtrl::read(BCtrlKind::Color, key.clone(), BValue::Color([1.0, 0.5, 0.25, 1.0]), &s);
        let Some(BView::Popover { body, .. }) = popover(&ctrl, BCtrlSwatch::of(&ctrl.value), &s) else {
            panic!("a scene colour opens a picker")
        };
        let BView::Wheel { wheel } = &body[0] else { panic!("{:?}", body[0]) };
        assert_eq!(wheel.space, crate::v2::view::draw::BPickSpace::Rgba);
        assert_eq!(body.len(), 5, "the wheel, then R, G, B and A");
    }

    #[test]
    fn every_colour_control_has_somewhere_to_record_its_picker_open() {
        // `BCtrl::read` gave none of them an open key, so the picker above was
        // never reached: clicking a swatch in Preferences, Properties, the
        // Shading popover or New Image did nothing.
        use crate::v2::state::BValue;
        use crate::v2::view::prim::BCtrlKind;
        let s = BState::new();
        for value in [BValue::Color([0.2, 0.2, 0.2, 1.0]), BValue::Text("0, 0, 24, 1".to_string())] {
            let ctrl = BCtrl::read(BCtrlKind::Color, BPath::owned(vec!["k"]), value, &s);
            assert!(ctrl.open_key.is_some(), "{:?}", ctrl.value);
            assert!(popover(&ctrl, BCtrlSwatch::of(&ctrl.value), &s).is_some());
        }
        // An unreadable theme colour still has no picker to open onto.
        let ctrl = BCtrl::read(BCtrlKind::Color, BPath::owned(vec!["k"]), BValue::Text("nope".to_string()), &s);
        assert!(popover(&ctrl, BCtrlSwatch::of(&ctrl.value), &s).is_none());
    }
}
