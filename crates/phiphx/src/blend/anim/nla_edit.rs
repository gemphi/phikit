//! The NLA editor's operators over the stacks it shows, one per object
//! (`ANIM_animdata_filter` in the NLA editor), as Blender 5.0 makes them:
//! click select (`nla.click_select`, `mouse_nla_strips`), a track picked in
//! the channel list (`anim.channels_click`, `mouse_nla_channels`), Select
//! All / None / Invert (`nla.select_all`, `deselect_nla_strips`), Delete
//! (`nla.delete`), Toggle Muting (`nla.mute_toggle`), and Add / Add Above
//! Selected / Delete tracks (`nla.tracks_add`, `nla.tracks_delete`), and
//! Add ▸ Action (`nla.actionclip_add`). Strips on a locked track are not
//! deleted or muted, nor added to (`ANIMFILTER_FOREDIT`).

use super::nla::*;

/// The name a new track starts from (`BKE_nlatrack_insert_before/after`).
pub const NEW_TRACK: &str = "NlaTrack";

/// How a pick sets a selection bit (`ACHANNEL_SETFLAG_ADD`, `_CLEAR`,
/// `_INVERT`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BNlaPick {
    Add,
    Clear,
    Invert,
}

impl BNlaPick {
    fn set(self, flag: &mut u32, bit: u32) {
        match self {
            Self::Add => *flag |= bit,
            Self::Clear => *flag &= !bit,
            Self::Invert => *flag ^= bit,
        }
    }
}

/// A strip among the stacks: `(stack, track, strip)`, the track counted
/// from the bottom of its stack.
pub type BNlaAt = (usize, usize, usize);

/// The stacks the NLA editor shows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BNlaStacks(pub Vec<BlendNla>);

impl BlendNla {
    /// Has a track new strips can go in (`nlaedit_get_editable_tracks`:
    /// the active one, unlocked)?
    pub fn takes_clips(&self) -> bool {
        self.tracks.iter().any(|t| t.flag & TRACK_ACTIVE != 0 && t.flag & TRACK_PROTECTED == 0)
    }

    /// `nla.actionclip_add` on this stack: an empty stack whose action line
    /// is picked gets a track (`nlaedit_add_tracks_empty`); then `strip`,
    /// moved to start at `frame`, goes in the active track when it has room
    /// — else in a new one on top, made active — named uniquely after its
    /// action. Whether it went in.
    pub fn add_clip(&mut self, mut strip: BNlaStrip, frame: f32) -> bool {
        if self.tracks.is_empty() && self.flag & ADT_UI_SELECTED != 0 {
            BNlaStacks::add_tracks_to(self, &[0]);
        }
        let Some(at) = self.tracks.iter().position(|t| t.flag & TRACK_ACTIVE != 0 && t.flag & TRACK_PROTECTED == 0) else { return false };
        (strip.end, strip.start) = (strip.end + frame - strip.start, frame);
        self.named(&mut strip);
        let at = match Self::has_space(&self.tracks[at].strips, strip.start, strip.end) {
            true => at,
            false => {
                BNlaStacks::add_tracks_to(self, &[self.tracks.len()]);
                self.tracks.len() - 1
            }
        };
        Self::insert(&mut self.tracks[at].strips, strip);
        true
    }
}

impl BNlaTrack {
    /// `nlaedit_strip_at_region_position`: the strip a click at `frame`
    /// lands on — the nearest overlapping `reach`, the click's frame ± 7 px
    /// (`BKE_nlastrip_within_bounds`, `BKE_nlastrip_distance_to_frame`).
    pub fn strip_at(&self, frame: f32, reach: (f32, f32)) -> Option<usize> {
        let (min, max) = reach;
        let inside = |x: f32, a: f32, b: f32| a < x && x < b;
        let mut best: Option<(usize, f32)> = None;
        for (k, s) in self.strips.iter().enumerate() {
            let (length, bounds) = (s.end - s.start, (max - min).abs());
            if length.abs() < f32::EPSILON || bounds < f32::EPSILON {
                continue;
            }
            if length < bounds && !(inside(s.start, min, max) || inside(s.end, min, max)) {
                continue;
            }
            if length > bounds && !(inside(min, s.start, s.end) || inside(max, s.start, s.end)) {
                continue;
            }
            let distance = if frame < s.start { s.start - frame } else if s.end < frame { frame - s.end } else { 0.0 };
            if best.is_some_and(|(_, d)| distance > d) {
                continue;
            }
            best = Some((k, distance));
            if distance == 0.0 {
                break;
            }
        }
        best.map(|(k, _)| k)
    }
}

impl BNlaStacks {
    fn strips(&mut self) -> impl Iterator<Item = &mut BNlaStrip> {
        self.0.iter_mut().flat_map(|n| n.tracks.iter_mut()).flat_map(|t| t.strips.iter_mut())
    }

    /// Unlocked tracks' strips (`ANIMFILTER_FOREDIT`).
    fn editable(&mut self) -> impl Iterator<Item = &mut BNlaTrack> {
        self.0.iter_mut().flat_map(|n| n.tracks.iter_mut()).filter(|t| t.flag & TRACK_PROTECTED == 0)
    }

    /// `deselect_nla_strips`: every strip's selection set by `pick` (any
    /// selected turns a `test`ed Add into Clear), and none active.
    pub fn pick_strips(&mut self, pick: BNlaPick, test: bool) {
        let pick = if test && self.strips().any(|s| s.flag & STRIP_SELECT != 0) { BNlaPick::Clear } else { pick };
        for s in self.strips() {
            pick.set(&mut s.flag, STRIP_SELECT);
            s.flag &= !STRIP_ACTIVE;
        }
    }

    /// `ANIM_anim_channels_select_set(ACHANNEL_SETFLAG_CLEAR)`: no track
    /// or action line selected or active.
    fn clear_channels(&mut self) {
        for n in &mut self.0 {
            n.flag &= !(ADT_UI_SELECTED | ADT_UI_ACTIVE);
            n.tracks.iter_mut().for_each(|t| t.flag &= !(TRACK_SELECTED | TRACK_ACTIVE));
        }
    }

    /// `ANIM_set_active_channel` on a track: the one active track shown.
    fn activate(&mut self, stack: usize, track: usize) {
        self.0.iter_mut().flat_map(|n| n.tracks.iter_mut()).for_each(|t| t.flag &= !TRACK_ACTIVE);
        if let Some(t) = self.0.get_mut(stack).and_then(|n| n.tracks.get_mut(track)) {
            t.flag |= TRACK_ACTIVE;
        }
    }

    /// `nla.click_select`: `hit` alone selected (with Shift, flipped), made
    /// active with its track; a click on nothing clears everything.
    pub fn click(&mut self, hit: Option<BNlaAt>, extend: bool) {
        if !extend {
            self.pick_strips(BNlaPick::Clear, false);
            self.clear_channels();
        }
        let Some((n, t, k)) = hit else { return };
        let Some(strip) = self.0.get_mut(n).and_then(|s| s.tracks.get_mut(t)).and_then(|t| t.strips.get_mut(k)) else { return };
        (if extend { BNlaPick::Invert } else { BNlaPick::Add }).set(&mut strip.flag, STRIP_SELECT);
        let selected = strip.flag & STRIP_SELECT != 0;
        self.strips().for_each(|s| s.flag &= !STRIP_ACTIVE);
        if selected {
            let track = &mut self.0[n].tracks[t];
            track.strips[k].flag |= STRIP_ACTIVE;
            track.flag |= TRACK_SELECTED;
            self.activate(n, t);
        }
    }

    /// `anim.channels_click` on a track: it alone selected (with Shift,
    /// flipped), and active while selected. Strips keep theirs.
    pub fn click_track(&mut self, stack: usize, track: usize, extend: bool) {
        if !extend {
            self.clear_channels();
        }
        let Some(t) = self.0.get_mut(stack).and_then(|n| n.tracks.get_mut(track)) else { return };
        (if extend { BNlaPick::Invert } else { BNlaPick::Add }).set(&mut t.flag, TRACK_SELECTED);
        if t.flag & TRACK_SELECTED != 0 {
            self.activate(stack, track);
        }
    }

    /// `nla.delete`: every selected strip, with a transition either side;
    /// how many went.
    pub fn delete_strips(&mut self) -> usize {
        let mut gone = 0;
        for track in self.editable() {
            let strips = &mut track.strips;
            let mut k = 0;
            while k < strips.len() {
                if strips[k].flag & STRIP_SELECT == 0 {
                    k += 1;
                    continue;
                }
                let from = if k > 0 && strips[k - 1].kind == STRIP_TRANSITION { k - 1 } else { k };
                let to = if strips.get(k + 1).is_some_and(|s| s.kind == STRIP_TRANSITION) { k + 2 } else { k + 1 };
                strips.drain(from..to);
                gone += 1;
                k = from;
            }
        }
        gone
    }

    /// `nla.mute_toggle`: every selected strip's muting flipped.
    pub fn mute_strips(&mut self) -> usize {
        let mut flipped = 0;
        for s in self.editable().flat_map(|t| t.strips.iter_mut()).filter(|s| s.flag & STRIP_SELECT != 0) {
            s.flag ^= STRIP_MUTED;
            flipped += 1;
        }
        flipped
    }

    /// `nla.tracks_add`: over each selected track (`above`), or once on
    /// top of each stack with one; then a track in each empty stack whose
    /// action line is picked. Each new track is `NlaTrack` made unique,
    /// selected, and its stack's active one. Whether any was added.
    pub fn add_tracks(&mut self, above: bool) -> bool {
        let mut added = false;
        for stack in &mut self.0 {
            let picked: Vec<usize> = (0..stack.tracks.len()).filter(|&k| stack.tracks[k].flag & TRACK_SELECTED != 0).collect();
            let at: Vec<usize> = match (above, picked.last()) {
                (true, _) => picked.iter().enumerate().map(|(n, k)| k + n + 1).collect(),
                (false, Some(_)) => vec![stack.tracks.len()],
                (false, None) if stack.tracks.is_empty() && stack.flag & ADT_UI_SELECTED != 0 => vec![0],
                (false, None) => Vec::new(),
            };
            added |= !at.is_empty();
            Self::add_tracks_to(stack, &at);
        }
        added
    }

    /// New tracks at `at`, in turn (`BKE_nlatrack_new_*`): each `NlaTrack`
    /// made unique, selected, and its stack's active track.
    fn add_tracks_to(stack: &mut BlendNla, at: &[usize]) {
        for &k in at {
            let taken: Vec<&str> = stack.tracks.iter().map(|t| t.name.as_str()).collect();
            let name = BlendNla::unique(NEW_TRACK, &taken);
            stack.tracks.insert(k, BNlaTrack { name, flag: TRACK_SELECTED | TRACK_OVERRIDELIBRARY_LOCAL, ..Default::default() });
            stack.tracks.iter_mut().enumerate().for_each(|(j, t)| if j == k { t.flag |= TRACK_ACTIVE } else { t.flag &= !TRACK_ACTIVE });
        }
    }

    /// `nla.tracks_delete`: every selected track with its strips (a solo
    /// one takes its stack's solo flag with it); how many went.
    pub fn delete_tracks(&mut self) -> usize {
        let mut gone = 0;
        for stack in &mut self.0 {
            if stack.tracks.iter().any(|t| t.flag & TRACK_SELECTED != 0 && t.flag & TRACK_SOLO != 0) {
                stack.flag &= !ADT_SOLO_TRACK;
            }
            let before = stack.tracks.len();
            stack.tracks.retain(|t| t.flag & TRACK_SELECTED == 0);
            gone += before - stack.tracks.len();
        }
        gone
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `nla_cube.blend` as read: Base Layer (Walk 1–97) under Overlay
    /// (Walk Again 120–168, muted), both tracks and both strips selected,
    /// Overlay active.
    fn cube() -> BNlaStacks {
        let strip = |name: &str, start: f32, end: f32, flag: u32| BNlaStrip { name: name.into(), start, end, flag, ..Default::default() };
        let base = BNlaTrack { name: "Base Layer".into(), flag: TRACK_SELECTED, strips: vec![strip("Walk", 1.0, 97.0, STRIP_SELECT)], address: 0 };
        let overlay = BNlaTrack { name: "Overlay".into(), flag: TRACK_SELECTED | TRACK_ACTIVE | TRACK_MUTED, strips: vec![strip("Walk Again", 120.0, 168.0, STRIP_SELECT | STRIP_MUTED)], address: 0 };
        BNlaStacks(vec![BlendNla { object: "Cube".into(), tracks: vec![base, overlay], ..Default::default() }])
    }

    fn picks(s: &BNlaStacks) -> Vec<(String, u32, u32, u32)> {
        let n = &s.0[0];
        n.tracks.iter().flat_map(|t| t.strips.iter().map(|x| (x.name.clone(), x.flag & STRIP_SELECT, x.flag & STRIP_ACTIVE, x.flag & STRIP_MUTED))).collect()
    }

    fn tracks(s: &BNlaStacks) -> Vec<(String, bool, bool)> {
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
}
