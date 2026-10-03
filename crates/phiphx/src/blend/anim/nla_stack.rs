//! Push Down and Stash (`BKE_nla_action_pushdown`, `BKE_nla_action_stash`,
//! `nla.cc`): the action an object plays, laid into its NLA as the clip
//! `BKE_nlastrip_new_for_slot` makes, named uniquely among the object's
//! strips (`BKE_nlastrip_validate_name`). Push Down puts it in the top
//! track when that has room, else in a new track named after the action,
//! and makes it the active strip, mixing as the action line did. Stash puts
//! it, unselected, in a muted and locked "[Action Stash]" track kept at the
//! bottom of the stack — once per action and slot. Either way the object
//! plays no action after. The session (phiber) and the writer
//! (`write/nla.rs`) both lay strips with these, so what shows is what saves.

use super::nla::*;

/// `STASH_TRACK_NAME`.
pub const STASH_TRACK: &str = "[Action Stash]";

/// Where a strip went: the track's index in the stack, bottom first, and
/// whether the track is new.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BNlaPlaced {
    pub track: usize,
    pub new_track: bool,
}

impl BNlaStrip {
    /// `BKE_nlastrip_new_for_slot`: a selected clip of `action` through the
    /// slot, over `range` — the slot's frame range, the manual one when the
    /// action has it (then no syncing its length) — cyclic when the action
    /// is.
    pub fn clip(action: &str, slot: (i32, &str), range: (f32, f32), manual: bool, cyclic: bool) -> Self {
        let (start, end) = range;
        // `BKE_nla_clip_length_ensure_nonzero`.
        let end = if end <= start { start + 1.0 } else { end };
        let mut flag = STRIP_SELECT | STRIP_SYNC_LENGTH;
        if manual {
            flag &= !STRIP_SYNC_LENGTH;
        }
        if cyclic {
            flag |= STRIP_USR_TIME_CYCLIC;
        }
        Self {
            start,
            end,
            actstart: start,
            actend: end,
            repeat: 1.0,
            scale: 1.0,
            flag,
            kind: STRIP_CLIP,
            extend: EXTEND_HOLD,
            action: Some(action.to_string()),
            slot: slot.0,
            slot_name: slot.1.to_string(),
            ..Default::default()
        }
    }
}

impl BlendNla {
    /// `BLI_uniquename_cb` with `.`: `name`, else `name.001`, `.002`, …
    pub(crate) fn unique(name: &str, taken: &[&str]) -> String {
        if !taken.contains(&name) {
            return name.to_string();
        }
        let base = match name.rsplit_once('.') {
            Some((base, n)) if n.len() == 3 && n.bytes().all(|c| c.is_ascii_digit()) => base,
            _ => name,
        };
        (1..10_000).map(|k| format!("{base}.{k:03}")).find(|n| !taken.contains(&n.as_str())).unwrap_or_else(|| base.to_string())
    }

    /// `BKE_nlastrips_has_space`.
    pub(crate) fn has_space(strips: &[BNlaStrip], start: f32, end: f32) -> bool {
        if (start - end).abs() < f32::EPSILON {
            return false;
        }
        for strip in strips {
            if strip.start >= end {
                return true;
            }
            if strip.end > start || strip.end > end {
                return false;
            }
        }
        true
    }

    /// `BKE_nlastrips_add_strip_unsafe`: in start order.
    pub(crate) fn insert(strips: &mut Vec<BNlaStrip>, strip: BNlaStrip) {
        let at = strips.iter().position(|s| s.start > strip.start).unwrap_or(strips.len());
        strips.insert(at, strip);
    }

    /// `BKE_nlastrip_validate_name`: the strip's own name, else its
    /// action's, unique among every strip here.
    pub(crate) fn named(&self, strip: &mut BNlaStrip) {
        let taken: Vec<&str> = self.tracks.iter().flat_map(|t| t.strips.iter().map(|s| s.name.as_str())).collect();
        let own = if strip.name.is_empty() { strip.action.clone().unwrap_or_else(|| "<No Action>".into()) } else { strip.name.clone() };
        strip.name = Self::unique(&own, &taken);
    }

    /// `BKE_nlatrack_set_active`.
    fn activate(&mut self, track: usize) {
        for (k, t) in self.tracks.iter_mut().enumerate() {
            if k == track { t.flag |= TRACK_ACTIVE } else { t.flag &= !TRACK_ACTIVE }
        }
    }

    /// `BKE_nla_action_pushdown`.
    pub fn push_down(&mut self, mut strip: BNlaStrip) -> BNlaPlaced {
        self.named(&mut strip);
        (strip.blend, strip.influence, strip.extend) = (self.blend, self.influence, self.extend);
        if self.influence < 1.0 {
            strip.flag |= STRIP_USR_INFLUENCE;
        }
        // `BKE_nlastrip_set_active`.
        for s in self.tracks.iter_mut().flat_map(|t| t.strips.iter_mut()) {
            s.flag &= !STRIP_ACTIVE;
        }
        strip.flag |= STRIP_ACTIVE;
        let room = self.tracks.last().is_some_and(|t| t.flag & TRACK_PROTECTED == 0 && Self::has_space(&t.strips, strip.start, strip.end));
        self.action = None;
        if room {
            let track = self.tracks.len() - 1;
            Self::insert(&mut self.tracks[track].strips, strip);
            return BNlaPlaced { track, new_track: false };
        }
        let name = strip.action.clone().unwrap_or_default();
        self.tracks.push(BNlaTrack { name, flag: TRACK_SELECTED | TRACK_OVERRIDELIBRARY_LOCAL, strips: vec![strip], address: 0 });
        let track = self.tracks.len() - 1;
        self.activate(track);
        BNlaPlaced { track, new_track: true }
    }

    /// `BKE_nla_action_stash`: `None` when this action and slot are already
    /// stashed (the slot by its identifier, which the file and the session
    /// share). The object plays no action after, either way.
    pub fn stash(&mut self, mut strip: BNlaStrip) -> Option<BNlaPlaced> {
        self.action = None;
        let stashed = |t: &BNlaTrack| t.name.contains(STASH_TRACK);
        if self.tracks.iter().filter(|t| stashed(t)).flat_map(|t| &t.strips).any(|s| s.action == strip.action && s.slot_name == strip.slot_name) {
            return None;
        }
        self.named(&mut strip);
        strip.flag = (strip.flag & !(STRIP_SELECT | STRIP_ACTIVE)) | STRIP_SYNC_LENGTH;
        let taken: Vec<&str> = self.tracks.iter().map(|t| t.name.as_str()).collect();
        let name = Self::unique(STASH_TRACK, &taken);
        // Above the last stash track, else at the bottom of the stack.
        let at = self.tracks.iter().rposition(stashed).map_or(0, |k| k + 1);
        let flag = TRACK_SELECTED | TRACK_OVERRIDELIBRARY_LOCAL | TRACK_MUTED | TRACK_PROTECTED;
        self.tracks.insert(at, BNlaTrack { name, flag, strips: vec![strip], address: 0 });
        self.activate(at);
        Some(BNlaPlaced { track: at, new_track: true })
    }
}

#[cfg(test)]
mod tests {
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
}
