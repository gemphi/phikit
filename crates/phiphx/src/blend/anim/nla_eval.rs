//! Playing the NLA (`animsys_evaluate_nla`, `anim_sys.cc`): each track,
//! bottom to top — only the solo one when there is one, none that is muted
//! — plays the strip the frame falls in, or the one a Hold carries past its
//! end (`nlastrips_ctime_get_strip`), at the frame of its action the strip
//! maps to (`nlastrip_get_frame_actionclip`), mixed over what the tracks
//! below left by its blend mode and influence (`nla_blend_value`). The
//! object's own action is then written over it (`motion.rs`).

use super::channels::channel_mut;
use super::fcurve::BFCurve;
use super::nla::*;
use super::read::BAction;
use crate::blend::file::types::*;
use crate::blend::object::transform::ObjectTransformRaw;
use std::collections::HashMap;

/// A clip strip with the curves of its action's slot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BMotionClip {
    pub strip: BNlaStrip,
    pub curves: Vec<BFCurve>,
}

/// One NLA track as motion plays it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BMotionTrack {
    pub flag: u32,
    pub clips: Vec<BMotionClip>,
}

impl BNlaStrip {
    /// `nlastrip_get_frame_actionclip`: the action's frame at scene `frame`
    /// inside the strip.
    pub fn action_frame(&self, frame: f32) -> f32 {
        let scale = if self.scale.abs() == 0.0 { 1.0 } else { self.scale.abs() };
        let length = if (self.actend - self.actstart).abs() < f32::EPSILON { 1.0 } else { self.actend - self.actstart };
        let whole = (self.repeat - self.repeat.floor()).abs() < f32::EPSILON;
        if self.flag & STRIP_REVERSE != 0 {
            if (frame - self.end).abs() < f32::EPSILON && whole {
                return self.actstart;
            }
            return self.actend - (frame - self.start).rem_euclid(length * scale) / scale;
        }
        if (frame - self.end).abs() < f32::EPSILON && whole {
            return self.actend;
        }
        self.actstart + (frame - self.start) % (length * scale) / scale
    }

    /// `nlastrip_evaluate_controls`: the strip's own influence when set by
    /// hand, else full, eased over its blend in and out.
    fn influence_at(&self, frame: f32) -> f32 {
        if self.flag & STRIP_USR_INFLUENCE != 0 {
            return self.influence;
        }
        if self.blendin > 0.0 && frame <= self.start + self.blendin {
            return ((frame - self.start) / self.blendin).abs();
        }
        if self.blendout > 0.0 && frame >= self.end - self.blendout {
            return ((self.end - frame) / self.blendout).abs();
        }
        1.0
    }
}

impl BMotionTrack {
    /// `nlastrips_ctime_get_strip`: the clip that plays at `frame`, and the
    /// scene frame it plays at — clamped to the strip when held.
    fn playing(&self, frame: f32) -> Option<(&BMotionClip, f32)> {
        let clips = &self.clips;
        let mut found = None;
        for (k, clip) in clips.iter().enumerate() {
            let s = &clip.strip;
            if frame >= s.start && frame <= s.end {
                found = Some((clip, frame));
                break;
            }
            if frame < s.start {
                found = match k {
                    0 => (s.extend == EXTEND_HOLD).then_some((clip, s.start)),
                    _ => (clips[k - 1].strip.extend != EXTEND_NOTHING).then_some((&clips[k - 1], clips[k - 1].strip.end)),
                };
                break;
            }
            if frame > s.end && k + 1 == clips.len() {
                found = (s.extend != EXTEND_NOTHING).then_some((clip, s.end));
            }
        }
        found.filter(|(clip, _)| clip.strip.flag & STRIP_MUTED == 0)
    }

    /// `nla_blend_value` / `nla_combine_value`, per channel.
    fn blend(mode: u32, property: &str, lower: f32, value: f32, influence: f32) -> f32 {
        match mode {
            BLEND_ADD => lower + value * influence,
            BLEND_SUBTRACT => lower - value * influence,
            BLEND_MULTIPLY => influence * (lower * value) + (1.0 - influence) * lower,
            // Combine: added over the default for a location or an angle,
            // multiplied for a scale; a quaternion mixes as a replace here.
            BLEND_COMBINE if property == "scale" => lower * value.powf(influence),
            BLEND_COMBINE if matches!(property, "location" | "rotation_euler") => lower + value * influence,
            _ => lower * (1.0 - influence) + value * influence,
        }
    }

    /// Every track over `raw` at `frame`, bottom to top.
    pub fn play(tracks: &[BMotionTrack], solo: bool, raw: &mut ObjectTransformRaw, frame: f32) {
        for track in tracks {
            let skip = if solo { track.flag & TRACK_SOLO == 0 } else { track.flag & TRACK_MUTED != 0 };
            let Some((clip, at)) = (!skip).then(|| track.playing(frame)).flatten() else { continue };
            let influence = clip.strip.influence_at(at);
            if influence <= 0.0 {
                continue;
            }
            let time = clip.strip.action_frame(at);
            for curve in clip.curves.iter().filter(|c| !c.muted()) {
                let (Some(value), Some(slot)) = (curve.eval(time), curve.own_property().and_then(|p| channel_mut(raw, p, curve.index as usize))) else { continue };
                *slot = Self::blend(clip.strip.blend, curve.property(), *slot, value, influence);
            }
        }
    }
}

impl BlendFile {
    /// The object's NLA tracks as motion plays them, bottom first, each
    /// clip with its action slot's curves; and whether a track is soloed.
    pub(crate) fn motion_tracks(&self, ob: &BlendBlock, actions: &HashMap<u64, BAction>, dna: &DnaType) -> (Vec<BMotionTrack>, bool) {
        let Some(adt) = self.block_at(self.ptr(ob, self.off("Object", "*adt", dna))) else { return (Vec::new(), false) };
        let Some(nla) = self.nla("", ob, dna) else { return (Vec::new(), false) };
        let tracks = self.list(self.ptr(adt, self.off("AnimData", "nla_tracks", dna)), self.off("NlaTrack", "*next", dna));
        let out = tracks
            .into_iter()
            .zip(nla.tracks)
            .map(|(track, read)| {
                let blocks = self.list(self.ptr(track, self.off("NlaTrack", "strips", dna)), self.off("NlaStrip", "*next", dna));
                let clips = blocks
                    .into_iter()
                    .zip(read.strips)
                    .filter(|(_, strip)| strip.kind == STRIP_CLIP)
                    .map(|(block, strip)| {
                        let act = self.ptr(block, self.off("NlaStrip", "*act", dna));
                        let curves = actions.get(&act).map(|a| a.curves(strip.slot).to_vec()).unwrap_or_default();
                        BMotionClip { strip, curves }
                    })
                    .collect();
                BMotionTrack { flag: read.flag, clips }
            })
            .collect();
        (out, nla.flag & ADT_SOLO_TRACK != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moving_x(from: (f32, f32), to: (f32, f32)) -> BFCurve {
        let key = |(f, v): (f32, f32)| {
            let mut k = super::super::fcurve::BKey::new([f - 1.0, v], [f, v], [f + 1.0, v], 1);
            k.ipo = super::super::fcurve::IPO_LIN;
            k
        };
        BFCurve::new("location", 0, vec![key(from), key(to)])
    }

    #[test]
    fn a_held_strip_plays_its_action_mapped_and_holds_either_side() {
        // CubeAction pushed down: keys 1 → 0 and 20 → 3, strip 5–30 over
        // action frames 5–30 (its manual range).
        let strip = BNlaStrip::clip("CubeAction", (1, "OBCube"), (5.0, 30.0), true, true);
        let track = BMotionTrack { flag: 0, clips: vec![BMotionClip { strip, curves: vec![moving_x((1.0, 0.0), (20.0, 3.0))] }] };
        let x = |frame: f32| {
            let mut raw = ObjectTransformRaw::default();
            BMotionTrack::play(std::slice::from_ref(&track), false, &mut raw, frame);
            raw.loc[0]
        };
        let at = |f: f32| 3.0 * (f - 1.0) / 19.0;
        assert!((x(0.0) - at(5.0)).abs() < 1e-5, "held before: the strip's first frame");
        assert!((x(10.0) - at(10.0)).abs() < 1e-5);
        assert!((x(40.0) - 3.0).abs() < 1e-5, "held after");
        let muted = BMotionTrack { flag: TRACK_MUTED, ..track.clone() };
        let mut raw = ObjectTransformRaw::default();
        BMotionTrack::play(&[muted], false, &mut raw, 10.0);
        assert_eq!(raw.loc[0], 0.0, "a muted track plays nothing");
        // Twice as slow, repeated: the action's time wraps.
        let slow = BNlaStrip { scale: 2.0, repeat: 2.0, end: 5.0 + 2.0 * 2.0 * 25.0, ..BNlaStrip::clip("A", (1, "OBA"), (5.0, 30.0), true, false) };
        assert!((slow.action_frame(5.0 + 2.0 * 10.0) - 15.0).abs() < 1e-5);
        assert!((slow.action_frame(5.0 + 2.0 * 30.0) - 10.0).abs() < 1e-5, "the second time through");
    }
}
