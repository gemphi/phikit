use super::*;

/// `nla_cube.blend` as read: Base Layer (Walk 1–97) under Overlay
/// (Walk Again 120–168, muted), both tracks and both strips selected,
/// Overlay active.
fn cube() -> BNlaStackList {
    let strip = |name: &str, start: f32, end: f32, flag: u32| BNlaStrip { name: name.into(), start, end, flag, ..Default::default() };
    let base = BNlaTrack { name: "Base Layer".into(), flag: TRACK_SELECTED, strips: vec![strip("Walk", 1.0, 97.0, STRIP_SELECT)], address: 0 };
    let overlay = BNlaTrack { name: "Overlay".into(), flag: TRACK_SELECTED | TRACK_ACTIVE | TRACK_MUTED, strips: vec![strip("Walk Again", 120.0, 168.0, STRIP_SELECT | STRIP_MUTED)], address: 0 };
    BNlaStackList(vec![BlendNla { object: "Cube".into(), tracks: vec![base, overlay], ..Default::default() }])
}

fn picks(s: &BNlaStackList) -> Vec<(String, u32, u32, u32)> {
    let n = &s.0[0];
    n.tracks.iter().flat_map(|t| t.strips.iter().map(|x| (x.name.clone(), x.flag & STRIP_SELECT, x.flag & STRIP_ACTIVE, x.flag & STRIP_MUTED))).collect()
}

fn tracks(s: &BNlaStackList) -> Vec<(String, bool, bool)> {
    s.0[0].tracks.iter().map(|t| (t.name.clone(), t.flag & TRACK_SELECTED != 0, t.flag & TRACK_ACTIVE != 0)).collect()
}

/// Step for step as bpy 5.0 made them on `nla_cube.blend` in the NLA
/// editor (`nla.select_all`, `mute_toggle`, `delete`, `tracks_add`,
/// `tracks_delete`).
#[test]
fn the_operators_edit_the_stack_as_blender_does() {
    let mut s = cube();
    s.pick_strips(BNlaPick::Clear, false);
    s.0[0].tracks[0].strips[0].flag |= STRIP_SELECT;
    assert_eq!(s.mute_strips(), 1);
    assert_eq!(picks(&s), [("Walk".into(), STRIP_SELECT, 0, STRIP_MUTED), ("Walk Again".into(), 0, 0, STRIP_MUTED)]);
    s.pick_strips(BNlaPick::Invert, false);
    assert_eq!(picks(&s), [("Walk".into(), 0, 0, STRIP_MUTED), ("Walk Again".into(), STRIP_SELECT, 0, STRIP_MUTED)]);
    s.pick_strips(BNlaPick::Add, false);
    s.mute_strips();
    assert_eq!(picks(&s), [("Walk".into(), STRIP_SELECT, 0, 0), ("Walk Again".into(), STRIP_SELECT, 0, 0)]);
    s.0[0].tracks[1].strips[0].flag &= !STRIP_SELECT;
    assert_eq!(s.delete_strips(), 1);
    assert_eq!(picks(&s), [("Walk Again".into(), 0, 0, 0)], "Walk gone");

    assert!(s.add_tracks(false));
    let t = |name: &str, active: bool| (name.to_string(), true, active);
    assert_eq!(tracks(&s), [t("Base Layer", false), t("Overlay", false), t("NlaTrack", true)]);
    assert!(s.add_tracks(true));
    assert_eq!(tracks(&s), [t("Base Layer", false), t("NlaTrack.001", false), t("Overlay", false), t("NlaTrack.002", false), t("NlaTrack", false), t("NlaTrack.003", true)]);
    s.0[0].tracks[0].flag &= !TRACK_SELECTED;
    assert_eq!(s.delete_tracks(), 5);
    assert_eq!(tracks(&s), [("Base Layer".to_string(), false, false)]);
    assert!(!s.add_tracks(false), "nothing picked: 'Select an existing NLA Track or an empty action line first'");
}

/// As bpy 5.0's `nla.actionclip_add(action='Idle')` at frame 100 on
/// `nla_cube.blend` (Overlay active): Idle (keys 10–30) goes into
/// Overlay before Walk Again; again, it has no room there and goes in a
/// new track on top as Idle.001; with no active track, or a locked one,
/// nothing goes in.
#[test]
fn an_action_clip_goes_in_the_active_track_or_a_new_one_on_top() {
    let mut s = cube();
    let idle = || BNlaStrip::clip("Idle", (1, "OBCube"), (10.0, 30.0), false, false);
    let cube = &mut s.0[0];
    assert!(cube.takes_clips() && cube.add_clip(idle(), 100.0));
    let overlay: Vec<(&str, f32, f32, f32, f32)> = cube.tracks[1].strips.iter().map(|x| (x.name.as_str(), x.start, x.end, x.actstart, x.actend)).collect();
    assert_eq!(overlay, [("Idle", 100.0, 120.0, 10.0, 30.0), ("Walk Again", 120.0, 168.0, 0.0, 0.0)]);
    assert_eq!(cube.tracks[1].strips[0].flag, STRIP_SELECT | STRIP_SYNC_LENGTH, "selected, not active");
    assert!(cube.add_clip(idle(), 100.0));
    assert_eq!((cube.tracks.len(), cube.tracks[2].name.as_str(), cube.tracks[2].strips[0].name.as_str()), (3, NEW_TRACK, "Idle.001"));
    assert_eq!(cube.tracks.iter().map(|t| t.flag & TRACK_ACTIVE != 0).collect::<Vec<_>>(), [false, false, true]);
    cube.tracks.iter_mut().for_each(|t| t.flag &= !TRACK_ACTIVE);
    assert!(!cube.takes_clips() && !cube.add_clip(idle(), 100.0), "no active track");
    cube.tracks[0].flag |= TRACK_ACTIVE | TRACK_PROTECTED;
    assert!(!cube.add_clip(idle(), 100.0), "a locked one");
}

#[test]
fn a_click_picks_one_strip_and_its_track_and_a_locked_track_keeps_its_strips() {
    let mut s = cube();
    s.click(Some((0, 0, 0)), false);
    assert_eq!(picks(&s), [("Walk".into(), STRIP_SELECT, STRIP_ACTIVE, 0), ("Walk Again".into(), 0, 0, STRIP_MUTED)]);
    assert_eq!(tracks(&s), [("Base Layer".to_string(), true, true), ("Overlay".to_string(), false, false)]);
    s.click(Some((0, 1, 0)), true);
    assert_eq!(picks(&s)[1], ("Walk Again".into(), STRIP_SELECT, STRIP_ACTIVE, STRIP_MUTED), "Shift adds it, active now");
    assert_eq!(picks(&s)[0].2, 0);
    s.click(Some((0, 1, 0)), true);
    assert_eq!(picks(&s)[1].1, 0, "Shift again takes it off");
    s.click(None, false);
    assert!(picks(&s).iter().all(|p| p.1 == 0) && tracks(&s).iter().all(|t| !t.1 && !t.2), "a click on nothing clears all");
    s.click_track(0, 1, false);
    assert_eq!(tracks(&s), [("Base Layer".to_string(), false, false), ("Overlay".to_string(), true, true)]);
    s.0[0].tracks[1].flag |= TRACK_PROTECTED;
    s.pick_strips(BNlaPick::Add, false);
    let muted = s.mute_strips();
    assert_eq!((muted, s.delete_strips()), (1, 1), "only Base Layer's strip");
    assert_eq!(s.0[0].tracks[1].strips.len(), 1);
    // Where a click lands: inside a strip, or within reach of its end.
    let overlay = &s.0[0].tracks[1];
    assert_eq!(overlay.strip_at(150.0, (148.0, 152.0)), Some(0));
    assert_eq!(overlay.strip_at(169.0, (167.9, 170.1)), Some(0), "just past its end");
    assert_eq!(overlay.strip_at(180.0, (178.0, 182.0)), None);
}
