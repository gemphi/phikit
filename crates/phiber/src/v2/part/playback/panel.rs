//! The Playback popover — `TIME_PT_playback` as `space_time.py` lays it
//! out with `use_property_split`: each row a heading (or nothing) on the
//! left and its control on the right. Sync; Audio: Scrubbing, Play Audio;
//! Playback: Limit to Frame Range, Allow Preroll (greyed while limited),
//! Follow Current Frame, Loop; Play In: the nine editors; Show: Subframes;
//! then Set Start Frame / Set End Frame (`anim.start_frame_set`,
//! `anim.end_frame_set`). A panel, not a menu: it stays open while its
//! boxes are ticked, and closes on a click outside.

use super::settings::{self, BPlayToggle, LOOP_KEY, LOOP_MODES, SYNC_KEY, SYNC_MODES};
use crate::v2::state::{BKey as _, BState, OpKey, val::BValue};
use crate::v2::view::prim::{BAction, BControl, BCtrl, BCtrlKind};
use crate::v2::view::BView;

fn check_row(s: &BState, heading: &str, toggle: &BPlayToggle) -> BView {
    BView::check_row(s, heading, settings::key(toggle.key), settings::default_of(toggle.key), toggle.label)
}

fn select_row(s: &BState, heading: &str, rna: &'static str, items: &[(&str, &str)], default: &str) -> BView {
    BView::select_row(s, heading, settings::key(rna), rna, items, default)
}

fn group(s: &BState, heading: &str, toggles: &[BPlayToggle]) -> Vec<BView> {
    toggles.iter().enumerate().map(|(n, toggle)| check_row(s, if n == 0 { heading } else { "" }, toggle)).collect()
}

/// The popover's rows.
pub fn body(s: &BState) -> Vec<BView> {
    let sync: Vec<(&str, &str)> = SYNC_MODES.iter().map(|(k, l, _)| (*k, *l)).collect();
    let loops: Vec<(&str, &str)> = LOOP_MODES.iter().map(|(k, l, _)| (*k, *l)).collect();
    let mut rows = vec![select_row(s, "Sync", SYNC_KEY, &sync, "AUDIO_SYNC")];
    rows.extend(group(s, "Audio", &settings::AUDIO));
    rows.extend(group(s, "Playback", &settings::PLAYBACK));
    rows.push(select_row(s, "Loop", LOOP_KEY, &loops, "INFINITE"));
    rows.extend(group(s, "Play In", &settings::PLAY_IN));
    rows.extend(group(s, "Show", &settings::SHOW));
    rows.push(BView::menu_sep());
    rows.push(BView::row(vec![
        BView::Button(BControl::new().with_label("Set Start Frame").with_action(BAction::Set(OpKey::FrameRange.path(), "start".into()))),
        BView::Button(BControl::new().with_label("Set End Frame").with_action(BAction::Set(OpKey::FrameRange.path(), "end".into()))),
    ]));
    rows
}

/// `TIME_PT_jump`: the unit and the delta a time jump moves by.
pub fn jump_body(s: &BState) -> Vec<BView> {
    let delta = BCtrl::read(BCtrlKind::Number { min: 0.0, max: 1.0e4, step: 1.0 }, settings::key(settings::JUMP_DELTA_KEY), BValue::Float(1.0), s);
    vec![
        select_row(s, "Jump Unit", settings::JUMP_UNIT_KEY, &settings::JUMP_UNITS, "SECOND"),
        BView::prop_row("Delta", delta),
    ]
}

/// `TIME_PT_playhead_snapping`: how near a key or marker must be, what the
/// playhead snaps to, and — for frames and seconds — the step.
pub fn snap_body(s: &BState) -> Vec<BView> {
    use super::snap::{self, BPlayheadSnap, TARGETS};
    let int = |rna: &'static str, default: u32, max: f32| {
        BCtrl::read(BCtrlKind::Number { min: 1.0, max, step: 1.0 }, settings::key(rna), BValue::Int(i64::from(default)), s)
    };
    let now = BPlayheadSnap::now(s);
    let mut rows = vec![BView::prop_row("Snap Distance", int(snap::DISTANCE_KEY, now.distance, 100.0))];
    rows.extend(TARGETS.iter().enumerate().map(|(n, t)| {
        BView::check_row(s, if n == 0 { "Snap Target" } else { "" }, settings::key(t.key), now.to & t.bit != 0, t.label)
    }));
    if now.to & TARGETS[0].bit != 0 {
        rows.push(BView::prop_row("Frame Step", int(snap::FRAME_STEP_KEY, now.frames, 32768.0)));
    }
    if now.to & TARGETS[1].bit != 0 {
        rows.push(BView::prop_row("Second Step", int(snap::SECOND_STEP_KEY, now.seconds, 32768.0)));
    }
    rows
}

/// `TIME_PT_auto_keyframing`: how auto keying keys, dimmed in Blender while
/// it is off.
pub fn autokey_body(s: &BState) -> Vec<BView> {
    vec![
        select_row(s, "Mode", settings::AUTO_KEY_MODE_KEY, &settings::AUTO_KEY_MODES, "ADD_REPLACE_KEYS"),
        check_row(s, "", &settings::AUTO_KEY_SET),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_lays_out_blenders_rows_with_their_headings() {
        let s = BState::new();
        let rows = body(&s);
        let headings: Vec<String> = rows
            .iter()
            .filter_map(|row| match row {
                // A property row's heading is its label column; a switch
                // group's heading is the line above its first switch.
                BView::Split { list } | BView::Column { list } => match list.first() {
                    Some(BView::Label { text }) if !text.is_empty() => Some(text.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert_eq!(headings, ["Sync", "Audio", "Playback", "Loop", "Play In", "Show"]);
        assert_eq!(rows.len(), 1 + 2 + 3 + 1 + 9 + 1 + 2, "every row, the separator and the frame buttons");
        // The factory file: 3D Viewport and Animation Editors play, the rest do not.
        let on = |rna: &'static str| settings::is_on(&s, rna);
        assert!(on("use_play_3d_editors") && on("use_play_animation_editors") && !on("use_play_image_editors"));
        assert!(on("use_audio") && !on("use_audio_scrub"));
    }
}
