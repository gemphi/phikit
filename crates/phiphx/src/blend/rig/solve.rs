//! The pose at a frame: the animation and drivers over the rig
//! ([`BRigState`]), then every bone evaluated after what it depends on —
//! its parent, its constraints' targets and custom space — an IK chain as
//! one unit (`BKE_pose_where_is_bone`, `iksolver_execute_tree`).

use super::constraint::{BCob, BConstraintKind, BTarget, CONSTRAINT_OFF};
use super::driver::BRnaPath;
use super::driver::BRnaStep;
use super::ik::BIkTree;
use super::math::{BEulerOrder, BMat4, BQuat, BVec3, BCompose};
use super::space::BParentTransform;
use super::view::BSceneView;
use super::{BONE_CONNECTED, BRig, BRigState};
use crate::blend::anim::channels::channel_mut;
use crate::blend::anim::fcurve::BFCurve;
use crate::blend::object::transform::ObjectTransformRaw;
use glam::{Mat3, Mat4, Vec3};

/// A solved pose: each bone's `pose_mat` (armature space) and `chan_mat`
/// (pose · rest⁻¹, what deforms the mesh).
#[derive(Clone, Debug, Default)]
pub struct BRigPose {
    pub pose: Vec<Mat4>,
    pub chan: Vec<Mat4>,
}

/// A channel's values by RNA property and index.
pub trait BChannelValue {
    fn value(&self, property: &str, index: usize) -> Option<f32>;
    fn set(&mut self, property: &str, index: usize, v: f32);
}

impl BChannelValue for ObjectTransformRaw {
    fn value(&self, property: &str, index: usize) -> Option<f32> {
        let mut copy = *self;
        channel_mut(&mut copy, property, index).map(|v| *v)
    }

    fn set(&mut self, property: &str, index: usize, v: f32) {
        if let Some(slot) = channel_mut(self, property, index) {
            *slot = v;
        }
    }
}

impl BRigState {
    /// A custom property as animated, else as saved.
    pub fn prop(&self, rig: &BRig, bone: usize, name: &str) -> f32 {
        self.props.get(&(bone, name.to_string())).copied().unwrap_or_else(|| rig.bones[bone].props.get(name).and_then(|v| v.as_float()).unwrap_or(0.0) as f32)
    }
}

/// The pose being solved.
pub struct BPoseWork<'a> {
    pub rig: &'a BRig,
    pub state: &'a BRigState,
    /// The rig object's world matrix and its inverse.
    pub ob: Mat4,
    pub ob_inv: Mat4,
    pub pose: Vec<Mat4>,
    pub chan: Vec<Mat4>,
    pub tail: Vec<Vec3>,
    pub done: Vec<bool>,
    visiting: Vec<bool>,
    /// The scene outside the rig: other objects and rigs its constraints
    /// read.
    pub others: &'a dyn BSceneView,
    trees: Vec<BIkTree>,
    /// The tree (by root) each chain bone is solved in.
    tree_of: Vec<Option<usize>>,
}

impl BRig {
    /// The animation over the rig at `frame` (`curves`: its action's slot),
    /// then its drivers.
    pub fn state_at(&self, curves: &[BFCurve], frame: f32) -> BRigState {
        let mut state = BRigState {
            raw: self.bones.iter().map(|b| b.raw).collect(),
            props: Default::default(),
            influence: self.bones.iter().map(|b| b.constraints.iter().map(|c| c.influence).collect()).collect(),
            muted: self.bones.iter().map(|b| b.constraints.iter().map(|c| c.flag & CONSTRAINT_OFF != 0).collect()).collect(),
            weights: self
                .bones
                .iter()
                .map(|b| b.constraints.iter().map(|c| if let BConstraintKind::Armature { targets, .. } = &c.kind { targets.iter().map(|t| t.1).collect() } else { Vec::new() }).collect())
                .collect(),
        };
        for c in curves.iter().filter(|c| !c.muted()) {
            let Some(v) = c.eval(frame) else { continue };
            let Some(path) = BRnaPath::parse(&c.path) else { continue };
            match path.0.as_slice() {
                [BRnaStep::Field(p), BRnaStep::Field(b), BRnaStep::Key(bone), rest @ ..] if p == "pose" && b == "bones" => {
                    let Some(i) = self.bone(bone) else { continue };
                    match rest {
                        [BRnaStep::Field(property)] => state.raw[i].set(property, c.index as usize, v),
                        [BRnaStep::Key(name)] => _ = state.props.insert((i, name.clone()), v),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        for d in &self.drivers {
            d.apply(self, &mut state, frame);
        }
        state
    }

    /// The pose at `frame`: `curves` the rig's action curves, `ob` the rig
    /// object's world matrix, `others` the scene around it (for targets
    /// outside the rig).
    pub fn pose_at(&self, curves: &[BFCurve], frame: f32, ob: Mat4, others: &dyn BSceneView) -> BRigPose {
        let state = self.state_at(curves, frame);
        self.solve(&state, ob, others)
    }

    /// Every bone of `state`, solved.
    pub fn solve(&self, state: &BRigState, ob: Mat4, others: &dyn BSceneView) -> BRigPose {
        let n = self.bones.len();
        let (trees, tree_of) = self.ik_trees(state);
        let mut work = BPoseWork {
            rig: self,
            state,
            ob,
            ob_inv: ob.inverse(),
            pose: self.bones.iter().map(|b| b.arm).collect(),
            chan: vec![Mat4::IDENTITY; n],
            tail: vec![Vec3::ZERO; n],
            done: vec![false; n],
            visiting: vec![false; n],
            others,
            trees,
            tree_of,
        };
        for b in 0..n {
            work.ensure(b);
        }
        BRigPose { pose: work.pose, chan: work.chan }
    }
}

impl BPoseWork<'_> {
    /// The bones `b` must wait for, outside its own IK tree.
    fn deps(&self, b: usize) -> Vec<usize> {
        let bone = &self.rig.bones[b];
        let mut out: Vec<usize> = bone.parent.into_iter().collect();
        for c in &bone.constraints {
            for t in std::iter::once(&c.space).chain(c.kind.targets()) {
                if let BTarget::Rig(Some(i)) = t {
                    out.push(*i);
                }
            }
        }
        out
    }

    /// Solve bone `b` (and whatever it needs first).
    fn ensure(&mut self, b: usize) {
        if self.done[b] || self.visiting[b] {
            return;
        }
        self.visiting[b] = true;
        match self.tree_of[b] {
            Some(t) => {
                let members = self.trees[t].bones.clone();
                // Everything each chain bone depends on outside its tree —
                // and the other trees sharing its root — first.
                let roots: Vec<usize> = (0..self.trees.len()).filter(|&i| self.trees[i].bones[0] == members[0]).collect();
                let all: Vec<usize> = roots.iter().flat_map(|&i| self.trees[i].bones.clone()).collect();
                all.iter().for_each(|&m| self.visiting[m] = true);
                for &m in &all {
                    for d in self.deps(m) {
                        if !all.contains(&d) {
                            self.ensure(d);
                        }
                    }
                }
                for i in roots {
                    let tree = self.trees[i].clone();
                    self.execute_tree(&tree);
                }
                for &m in &all {
                    if !self.done[m] {
                        self.bone_done(m);
                    }
                }
            }
            None => {
                for d in self.deps(b) {
                    self.ensure(d);
                }
                self.where_is_bone(b);
                self.bone_done(b);
            }
        }
    }

    /// `BKE_pchan_to_mat4`: the channel's own matrix.
    fn chan_mat(&self, b: usize) -> Mat4 {
        Self::channel_matrix(&self.state.raw[b], self.rig.bones[b].flag & BONE_CONNECTED != 0)
    }

    /// `BKE_pchan_to_mat4` over `raw`: its rotation and scale, and its
    /// location unless the bone is connected.
    pub fn channel_matrix(raw: &ObjectTransformRaw, connected: bool) -> Mat4 {
        let rot = match raw.rotmode {
            m if m > 0 => BEulerOrder(m).to_mat3(Vec3::from(raw.rot)),
            -1 => {
                let mut axis = Vec3::from(raw.rot_axis);
                if axis.normalize_len() == 0.0 { Mat3::IDENTITY } else { BCompose::axis_angle(axis, raw.rot_angle) }
            }
            _ => {
                let q = raw.quat;
                let len = q.iter().map(|v| v * v).sum::<f32>().sqrt();
                BQuat(if len != 0.0 { q.map(|v| v / len) } else { [1.0, 0.0, 0.0, 0.0] }).to_mat3()
            }
        };
        let mut m = Mat4::from_mat3(rot * Mat3::from_diagonal(Vec3::from(raw.size)));
        if !connected {
            m.set_loc(Vec3::from(raw.loc));
        }
        m
    }

    /// `BKE_pose_where_is_bone` (with constraints): the parent's pose, the
    /// channel, then the constraints.
    pub fn where_is_bone(&mut self, b: usize) {
        let chan = self.chan_mat(b);
        let mut pose = BParentTransform::of(&self.rig.bones, b, &self.pose).apply(chan);
        let bone = &self.rig.bones[b];
        if !bone.constraints.is_empty() {
            let head = pose.loc();
            // `cob->rotOrder`: the channel's Euler order, XYZ for a
            // quaternion or axis-angle.
            let mode = self.state.raw[b].rotmode;
            let mut cob = BCob { bone: Some(b), matrix: self.ob * pose, rot_order: if mode > 0 { mode } else { 1 }, custom: Mat4::IDENTITY };
            self.solve_constraints(&mut cob, b);
            pose = self.ob_inv * cob.matrix;
            if bone.flag & BONE_CONNECTED != 0 {
                pose.set_loc(head);
            }
        }
        self.set_pose(b, pose);
    }

    /// Store a pose and its tail (`BKE_pose_where_is_bone_tail`).
    pub fn set_pose(&mut self, b: usize, pose: Mat4) {
        self.pose[b] = pose;
        self.tail[b] = pose.loc() + pose.y_axis.truncate() * self.rig.bones[b].length;
    }

    /// `BKE_pose_bone_done`: the deform matrix.
    fn bone_done(&mut self, b: usize) {
        self.chan[b] = self.pose[b] * self.rig.bones[b].arm.inverse();
        self.done[b] = true;
    }
}
