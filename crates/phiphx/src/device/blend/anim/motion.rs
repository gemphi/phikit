//! What moves an object over time: its rest transform, the F-curves its
//! action's slot binds to it, and its place in the parent chain — enough to
//! rebuild `object_to_world` at any frame the way Blender's depsgraph does
//! (`BKE_object_where_is_calc`: local from the animated channels, then
//! `parent.world · parentinv · local` up the chain).
//!
//! One [`BlendMotion`] per object in file order; the list is what a player
//! asks for a frame's matrices. An object with no curves still has a motion
//! — its rest transform — so a child of an animated parent moves with it.
//! Constraints and bone parents are solved with it ([`follow`](super::follow)).

use super::channel;
use super::fcurve::BFCurve;
use super::read::BAction;
use crate::device::blend::file::kind::*;
use crate::device::blend::modifier::armature::name64;
use crate::device::blend::read::object::transform::{ObjectTransformRaw, compose_world, object_to_mat4};
use crate::device::blend::read::curve::{BCurvePath, BPathAt, CU_PATH};
use crate::device::blend::rig::BConstraint;
use std::sync::Arc;
use std::collections::HashMap;

/// One object's transform over time.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendMotion {
    /// The object's name, so a matrix can be found by it.
    pub name: String,
    /// Rest transform: what `BKE_object_to_mat4` reads when no curve writes it.
    pub transform: ObjectTransformRaw,
    /// The curves bound to this object (its action's slot), empty when still.
    pub curves: Vec<BFCurve>,
    /// The parent's index in the list, when the parent is in the scene.
    pub parent: Option<usize>,
    /// Its NLA tracks, bottom first, and whether one is soloed
    /// (`nla_eval.rs`) — played under `curves`.
    pub nla: Vec<super::nla_eval::BMotionTrack>,
    pub solo: bool,
    /// The object's block address: what a constraint targeting it names.
    pub address: u64,
    /// Its constraints (`Object.constraints`), solved after its parent.
    pub constraints: Vec<BConstraint>,
    /// `Object.parsubstr`: the bone of its armature parent it hangs from,
    /// when its parent type is `PARBONE`.
    pub parent_bone: String,
    /// `Object.trackflag` and `upflag`: the axes that follow a path it is
    /// parented to.
    pub track: [i16; 2],
    /// A curve object's path, which its children ride.
    pub path: Option<Arc<BlendMotionPath>>,
    /// The drivers it carries over its own transform channels and custom
    /// properties, with those properties as saved (`anim::driver`) — run
    /// after the actions as Blender's depsgraph runs them.
    pub drivers: super::driver::BlendDriverSet,
    /// Motion played from code, read before the curves (`play.rs`).
    pub play: Option<super::play::BPlayRef>,
}

/// A curve's path and its evaluation time over the frames.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendMotionPath {
    pub path: BCurvePath,
    /// `Curve.ctime` as saved, and the curve data's curves (`eval_time`).
    pub ctime: f32,
    pub curves: Vec<BFCurve>,
}

impl BlendMotionPath {
    /// The evaluation time at `frame`: its curve's value (keys, or a
    /// Generator over the frame), else as saved.
    pub fn ctime_at(&self, frame: f32) -> f32 {
        self.curves
            .iter()
            .filter(|c| c.path == "eval_time" && !c.muted() && (!c.keys.is_empty() || c.generated(frame).is_some()))
            .last()
            .map_or(self.ctime, |c| c.evaluate(frame))
    }

    /// Does the evaluation time move?
    pub fn animated(&self) -> bool {
        self.curves.iter().any(|c| c.path == "eval_time" && !c.muted() && (c.keys.len() > 1 || c.generated(0.0).is_some()))
    }

    /// The path at `frame`.
    pub fn at(&self, frame: f32) -> BPathAt<'_> {
        BPathAt { path: &self.path, ctime: self.ctime_at(frame) }
    }
}

impl BlendMotion {
    /// A still object: its rest transform, no curves.
    pub fn still(name: impl Into<String>, transform: ObjectTransformRaw) -> Self {
        Self {
            name: name.into(),
            transform,
            curves: Vec::new(),
            parent: None,
            nla: Vec::new(),
            solo: false,
            address: 0,
            constraints: Vec::new(),
            parent_bone: String::new(),
            track: [1, 2],
            path: None,
            drivers: Default::default(),
            play: None,
        }
    }

    /// Does anything key this object — or, for a path, its evaluation time?
    /// A driver counts: it moves the object over the frames even with no
    /// key on it at all, which is why a driven object has to be re-solved
    /// per frame rather than placed once.
    pub fn animated(&self) -> bool {
        if self.play.is_some() || self.path.as_ref().is_some_and(|p| p.animated()) || !self.drivers.is_empty() {
            return true;
        }
        let clips = self.nla.iter().filter(|t| t.flag & super::nla::TRACK_MUTED == 0).flat_map(|t| &t.clips);
        self.curves.iter().chain(clips.flat_map(|c| &c.curves)).any(|curve| !curve.keys.is_empty())
    }

    /// `BKE_object_to_mat4` at `frame`: the rest transform with every curve
    /// written over it. Blender's `float[4][4]` layout.
    pub fn local_at(&self, frame: f32) -> [[f32; 4]; 4] {
        object_to_mat4(&self.raw_at(frame))
    }

    /// The rest transform with every curve written over it at `frame`: the
    /// values the object's properties hold there — what `I` keys.
    pub fn raw_at(&self, frame: f32) -> ObjectTransformRaw {
        if let Some(raw) = self.play.as_ref().and_then(|p| p.0.object(frame)) {
            return raw;
        }
        let mut raw = self.transform;
        super::nla_eval::BMotionTrack::play(&self.nla, self.solo, &mut raw, frame);
        // Only the object's own channels: a bone's curves in the same
        // action are the rig's.
        channel::apply(&mut raw, self.curves.iter().filter(|c| c.own_property().is_some()), frame);
        raw
    }

    /// The first and last keyed frame, when keyed.
    pub fn range(&self) -> Option<(f32, f32)> {
        if let Some(play) = &self.play {
            return Some(play.0.range());
        }
        let strips = self.nla.iter().flat_map(|t| &t.clips).flat_map(|c| [c.strip.start, c.strip.end]);
        let frames = self.curves.iter().flat_map(|curve| curve.keys.iter().map(|key| key.co[0])).chain(strips);
        frames.clone().map(|f| (f, f)).reduce(|(lo, hi), (f, _)| (lo.min(f), hi.max(f)))
    }
}

/// Every object's motion, in file order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendMotionList {
    pub list: Vec<BlendMotion>,
}

impl BlendMotionList {
    /// Does any object move?
    pub fn animated(&self) -> bool {
        self.list.iter().any(BlendMotion::animated)
    }

    /// Every object's world matrix at `frame`, in list order — each object's
    /// local at the frame, composed up its parent chain, then its
    /// constraints (no rig posed: see [`solve`](Self::solve)).
    pub fn world_at(&self, frame: f32) -> Vec<[[f32; 4]; 4]> {
        self.solve(frame, &[], false).world
    }

    /// Does an object follow another beyond plain parenting — a constraint,
    /// a bone parent, or a curve parent's path?
    pub fn follows(&self) -> bool {
        (0..self.list.len()).any(|i| self.follower(i))
    }

    /// Does object `i` itself follow another that way — so where it stands
    /// depends on the frame, not only on its own and its parents' keys?
    pub fn follower(&self, i: usize) -> bool {
        let m = &self.list[i];
        let on_path = m.transform.partype & 15 == 0 && m.parent.is_some_and(|p| self.list[p].path.as_ref().is_some_and(|p| p.path.flag & CU_PATH != 0));
        !m.constraints.is_empty() || m.transform.partype & 15 == super::follow::PARBONE || on_path
    }

    /// Every object's channels at `frame`: the actions and NLA first, then
    /// the drivers over them — Blender's own order, since the depsgraph
    /// evaluates the animation system for a property and its driver after
    /// (`anim::driver`). A driver on a channel wins over a key on it.
    pub fn raw_at(&self, frame: f32) -> Vec<ObjectTransformRaw> {
        self.driver_state_at(frame).raw
    }

    /// The motion named `name`.
    pub fn named(&self, name: &str) -> Option<&BlendMotion> {
        self.list.iter().find(|motion| motion.name == name)
    }
}

/// `world = parent.world · parentinv · local` for every object, in order.
/// A parent outside the list leaves the object on its local; a cycle is cut
/// at the object first met again, which keeps its local.
pub fn solve_world<'a>(
    local: &[[[f32; 4]; 4]],
    parent: impl Fn(usize) -> Option<usize>,
    parentinv: impl Fn(usize) -> &'a [[f32; 4]; 4],
) -> Vec<[[f32; 4]; 4]> {
    let n = local.len();
    let mut world: Vec<[[f32; 4]; 4]> = local.to_vec();
    let mut state = vec![BSolve::Unsolved; n];
    fn solve<'a>(
        i: usize,
        local: &[[[f32; 4]; 4]],
        parent: &impl Fn(usize) -> Option<usize>,
        parentinv: &impl Fn(usize) -> &'a [[f32; 4]; 4],
        world: &mut [[[f32; 4]; 4]],
        state: &mut [BSolve],
    ) -> [[f32; 4]; 4] {
        match state[i] {
            BSolve::Done => return world[i],
            BSolve::Solving => return local[i],
            BSolve::Unsolved => state[i] = BSolve::Solving,
        }
        if let Some(p) = parent(i).filter(|&p| p != i && p < local.len()) {
            let pw = solve(p, local, parent, parentinv, world, state);
            world[i] = compose_world(&pw, parentinv(i), &local[i]);
        }
        state[i] = BSolve::Done;
        world[i]
    }
    for i in 0..n {
        solve(i, local, &parent, &parentinv, &mut world, &mut state);
    }
    world
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BSolve {
    Unsolved,
    Solving,
    Done,
}

impl BlendFile {
    /// A curve object's path, with its data's evaluation-time curves.
    fn motion_path(&self, ob: &BlendBlock, actions: &HashMap<u64, BAction>, dna: &DnaType) -> Option<BlendMotionPath> {
        let curve = self.object_curve(ob, dna)?;
        let path = BCurvePath::of(&curve)?;
        let data = self.block_at(self.ptr(ob, self.off("Object", "*data", dna)))?;
        let curves = self.id_binding(data, "Curve", dna).and_then(|b| actions.get(&b.action).map(|a| a.curves(b.slot).to_vec())).unwrap_or_default();
        Some(BlendMotionPath { path, ctime: curve.ctime, curves })
    }

    /// Every object's motion: its rest transform, the curves of the action
    /// slot its `AnimData` binds, and its parent's index. `actions` is the
    /// file's action table (`BlendFile::actions`), read once by the caller.
    pub(crate) fn motion_list(
        &self,
        objects: &[BlendSceneObject],
        actions: &HashMap<u64, BAction>,
        dna: &DnaType,
    ) -> BlendMotionList {
        let index_of: HashMap<u64, usize> =
            objects.iter().enumerate().filter(|(_, o)| o.block_address != 0).map(|(i, o)| (o.block_address, i)).collect();
        let list = objects
            .iter()
            .map(|object| {
                let block = self.block_at(object.block_address);
                let curves = block
                    .and_then(|block| self.binding(block, dna))
                    .and_then(|binding| actions.get(&binding.action).map(|action| action.curves(binding.slot).to_vec()))
                    .unwrap_or_default();
                // An object with no block of its own — a collection
                // instance (`object/instance.rs`) — stands at its own rest
                // transform.
                let transform = block.map_or_else(|| ObjectTransformRaw { loc: object.location, quat: object.quaternion, size: object.scale, rotmode: 0, ..Default::default() }, |block| self.read_object_transform_raw(block, dna));
                let parent = index_of.get(&object.parent_address).copied().filter(|&p| objects[p].name != object.name);
                let (nla, solo) = block.map(|block| self.motion_tracks(block, actions, dna)).unwrap_or_default();
                let constraints = block.map(|block| self.object_constraints(block, dna)).unwrap_or_default();
                let parent_bone = block.and_then(|block| self.off("Object", "parsubstr[64]", dna).map(|o| name64(&block.data, o))).unwrap_or_default();
                let track = block.map_or([1, 2], |block| ["trackflag", "upflag"].map(|f| self.int_field(block, "Object", f, dna) as i16));
                let path = block.filter(|_| object.object_type == "CURVE").and_then(|block| self.motion_path(block, actions, dna)).map(Arc::new);
                let drivers = block.map(|block| self.object_driver_set(block, dna)).unwrap_or_default();
                BlendMotion { name: object.name.clone(), transform, curves, parent, nla, solo, address: object.block_address, constraints, parent_bone, track, path, drivers, play: None }
            })
            .collect();
        BlendMotionList { list }
    }
}

#[cfg(test)]
#[path = "../test/anim/motion.rs"]
mod motion_test;
