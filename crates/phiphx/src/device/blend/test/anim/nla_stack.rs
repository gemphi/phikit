use super::*;

/// As Blender 5.0 lays them (`bpy`, `action.push_down` / `action.stash`
/// on the Action Editor): the Cube's CubeAction (keys 1–20, Manual
/// Frame Range 5–30, Cyclic) pushed down, the Camera's CameraAction
/// (keys 1–40) stashed.
#[test]
fn push_down_and_stash_lay_strips_as_blender_does() {
    let mut cube = BlendNla { action: Some("CubeAction".into()), influence: 1.0, ..Default::default() };
    let placed = cube.push_down(BNlaStrip::clip("CubeAction", (7, "OBCube"), (5.0, 30.0), true, true));
    assert_eq!(placed, BNlaPlaced { track: 0, new_track: true });
    let (track, strip) = (&cube.tracks[0], &cube.tracks[0].strips[0]);
    assert_eq!((track.name.as_str(), track.flag), ("CubeAction", 0x10003));
    assert_eq!((strip.name.as_str(), strip.flag, strip.start, strip.end, strip.influence), ("CubeAction", 0x83, 5.0, 30.0, 1.0));
    assert_eq!(cube.action, None);
    // A second push that fits goes into the same track, named apart.
    cube.push_down(BNlaStrip::clip("CubeAction", (7, "OBCube"), (40.0, 50.0), false, false));
    assert_eq!(cube.tracks.len(), 1);
    assert_eq!(cube.tracks[0].strips[1].name, "CubeAction.001");
    assert_eq!(cube.tracks[0].strips[0].flag & STRIP_ACTIVE, 0, "one active strip");

    let mut camera = BlendNla { action: Some("CameraAction".into()), influence: 1.0, ..Default::default() };
    camera.tracks.push(BNlaTrack { name: "Walk".into(), flag: TRACK_ACTIVE, ..Default::default() });
    let placed = camera.stash(BNlaStrip::clip("CameraAction", (7, "OBCamera"), (1.0, 40.0), false, false)).unwrap();
    assert_eq!(placed.track, 0, "at the bottom");
    let (track, strip) = (&camera.tracks[0], &camera.tracks[0].strips[0]);
    assert_eq!((track.name.as_str(), track.flag), (STASH_TRACK, 0x10017));
    assert_eq!((strip.flag, strip.influence), (0x200, 0.0));
    assert_eq!(camera.tracks[1].flag & TRACK_ACTIVE, 0);
    assert!(camera.stash(BNlaStrip::clip("CameraAction", (7, "OBCamera"), (1.0, 40.0), false, false)).is_none(), "stashed once");
}
