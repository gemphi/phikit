//! A frame of the scene's motion, solved in the order Blender's depsgraph
//! evaluates it: each object after its parent and whatever its constraints
//! read — other objects, other rigs' bones (the rig posed first) — then
//! `BKE_object_where_is_calc`: its local, its parent's matrix (the parent
//! object — or where it is on a curve parent's path — or one of its bones
//! for `PARBONE`, `BKE_object_get_parent_matrix`) through `parentinv`, then
//! its constraints (`BObjectWork`).

use super::motion::{BlendMotion, BlendMotionList};
use crate::blend::curve::BPathAt;
use crate::blend::object::transform::object_to_mat4;
use crate::blend::rig::{BObjectSpace, BObjectWork, BRigPose, BRigSpace, BSceneView, BlendRigSource};
use glam::Mat4;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// `PARTYPE`, `PAROBJECT`, `PARBONE`.
const PARTYPE: u8 = 15;
const PAROBJECT: u8 = 0;
pub const PARBONE: u8 = 7;
/// `BONE_RELATIVE_PARENTING`: a child of the bone rides its deform, not its
/// tail.
const BONE_RELATIVE_PARENTING: u32 = 1 << 23;

/// Every object's world matrix at a frame, and the poses of the rigs posed
/// to find them (or of every rig, when asked), by object index.
#[derive(Clone, Debug, Default)]
pub struct BMotionAt {
    pub frame: f32,
    pub world: Vec<[[f32; 4]; 4]>,
    pub poses: HashMap<usize, BRigPose>,
    /// Where each object hangs from its parent — the parent matrix's place
    /// (`parent_display_origin`): the parent object, a bone's tail, a point
    /// on a curve parent's path. `None` without a parent.
    pub parent_origin: Vec<Option<[f32; 3]>>,
    /// Each object's open constraints' targets, where their lines run from
    /// (`BObjectWork::target_points`).
    pub targets: Vec<Vec<[f32; 3]>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BStep {
    Unsolved,
    Solving,
    Done,
}

/// The frame being solved.
struct BFollow<'a> {
    list: &'a [BlendMotion],
    rigs: &'a [Arc<BlendRigSource>],
    frame: f32,
    index: HashMap<u64, usize>,
    local: Vec<Mat4>,
    rotmode: Vec<i16>,
    world: Vec<Mat4>,
    step: Vec<BStep>,
    poses: HashMap<usize, BRigPose>,
    posing: HashSet<usize>,
    parent_origin: Vec<Option<[f32; 3]>>,
    targets: Vec<Vec<[f32; 3]>>,
}

impl BlendMotionList {
    /// Every object at `frame`: `rigs` the rigs to pose (by object index),
    /// each posed when an object follows one of its bones — or every one,
    /// with `all_rigs`. A cycle is cut where it closes, as in
    /// [`solve_world`](super::motion::solve_world).
    pub fn solve(&self, frame: f32, rigs: &[Arc<BlendRigSource>], all_rigs: bool) -> BMotionAt {
        let raw: Vec<_> = self.list.iter().map(|m| m.raw_at(frame)).collect();
        let local: Vec<Mat4> = raw.iter().map(|r| Mat4::from_cols_array_2d(&object_to_mat4(r))).collect();
        let n = local.len();
        let mut f = BFollow {
            list: &self.list,
            rigs,
            frame,
            index: self.list.iter().enumerate().filter(|(_, m)| m.address != 0).map(|(i, m)| (m.address, i)).collect(),
            rotmode: raw.iter().map(|r| r.rotmode).collect(),
            world: local.clone(),
            local,
            step: vec![BStep::Unsolved; n],
            poses: HashMap::new(),
            posing: HashSet::new(),
            parent_origin: vec![None; n],
            targets: vec![Vec::new(); n],
        };
        for i in 0..n {
            f.ensure(i);
        }
        if all_rigs {
            for r in rigs {
                f.ensure_pose(r.object);
            }
        }
        BMotionAt { frame, world: f.world.iter().map(Mat4::to_cols_array_2d).collect(), poses: f.poses, parent_origin: f.parent_origin, targets: f.targets }
    }
}

impl BFollow<'_> {
    /// Solve object `i`, and whatever it follows first.
    fn ensure(&mut self, i: usize) {
        if i >= self.step.len() || self.step[i] != BStep::Unsolved {
            return;
        }
        self.step[i] = BStep::Solving;
        let list = self.list;
        let m = &list[i];
        let parent = m.parent.filter(|&p| p != i);
        if let Some(p) = parent {
            self.ensure(p);
            if m.transform.partype & PARTYPE == PARBONE {
                self.ensure_pose(p);
            }
        }
        for (address, bone) in m.constraints.iter().flat_map(|c| c.reads()) {
            if let Some(&j) = self.index.get(&address) {
                self.ensure(j);
                if bone {
                    self.ensure_pose(j);
                }
            }
        }
        let mut world = match parent {
            Some(p) => {
                let at = self.parent_matrix(m, p);
                self.parent_origin[i] = Some(at.w_axis.truncate().to_array());
                at * Mat4::from_cols_array_2d(&m.transform.parentinv) * self.local[i]
            }
            None => self.local[i],
        };
        if !m.constraints.is_empty() {
            let work = BObjectWork { basis: self.basis_at(i), scene: &*self };
            world = work.solve(&m.constraints, &m.curves, self.frame, world, self.rotmode[i]);
            let points = work.target_points(&m.constraints).iter().map(|p| p.to_array()).collect();
            self.targets[i] = points;
        }
        self.world[i] = world;
        self.step[i] = BStep::Done;
    }

    /// Pose the rig of object `j` (after its object and whatever its bones'
    /// constraints read), once.
    fn ensure_pose(&mut self, j: usize) {
        let rigs = self.rigs;
        let Some(source) = rigs.iter().find(|r| r.object == j) else { return };
        if self.poses.contains_key(&j) || !self.posing.insert(j) {
            return;
        }
        self.ensure(j);
        for (address, bone) in source.rig.bones.iter().flat_map(|b| &b.constraints).flat_map(|c| c.reads()) {
            if let Some(&k) = self.index.get(&address).filter(|&&k| k != j) {
                self.ensure(k);
                if bone {
                    self.ensure_pose(k);
                }
            }
        }
        let pose = source.pose_at(self.frame, self.world[j], &*self);
        self.poses.insert(j, pose);
    }

    /// `BKE_object_get_parent_matrix`: the parent's world, and for a bone
    /// parent (`ob_parbone`) the bone's pose moved to its tail — or its
    /// deform, with relative parenting. A missing bone leaves the parent's
    /// world, as Blender does.
    fn parent_matrix(&self, m: &BlendMotion, p: usize) -> Mat4 {
        let world = self.world[p];
        if m.transform.partype & PARTYPE == PAROBJECT {
            // A curve parent's path (`ob_parcurve`).
            let on_path = self.list[p].path.as_ref().and_then(|path| path.at(self.frame).parent_matrix(m.track[0], m.track[1]));
            return on_path.map_or(world, |path| world * path);
        }
        if m.transform.partype & PARTYPE != PARBONE {
            return world;
        }
        let Some((rig, b)) = self.rig_at(p).and_then(|r| r.rig.bone(&m.parent_bone).map(|b| (r, b))) else { return world };
        let bone = &rig.rig.bones[b];
        let pose = rig.pose[b];
        let bone_matrix = if bone.flag & BONE_RELATIVE_PARENTING != 0 {
            pose * bone.arm.inverse()
        } else {
            let mut t = pose;
            t.w_axis += (pose.y_axis.truncate() * bone.length).extend(0.0);
            t
        };
        world * bone_matrix
    }

    /// What object `i`'s local space is taken against: its parent's world
    /// through `parentinv`, or its own rotation.
    fn basis_at(&self, i: usize) -> Mat4 {
        let m = &self.list[i];
        let parent = m.parent.filter(|&p| p != i).map(|p| (self.world[p], Mat4::from_cols_array_2d(&m.transform.parentinv)));
        BObjectSpace::basis_of(parent, self.local[i])
    }

    /// Object `i`'s rig as posed, when it has been.
    fn rig_at(&self, i: usize) -> Option<BRigSpace<'_>> {
        let source = self.rigs.iter().find(|r| r.object == i)?;
        let pose = self.poses.get(&i)?;
        let ob = self.world[i];
        Some(BRigSpace { rig: &source.rig, pose: &pose.pose, ob, ob_inv: ob.inverse(), basis: Some(self.basis_at(i)) })
    }
}

impl BSceneView for BFollow<'_> {
    fn world(&self, address: u64) -> Option<Mat4> {
        self.index.get(&address).map(|&i| self.world[i])
    }

    fn basis(&self, address: u64) -> Option<Mat4> {
        self.index.get(&address).map(|&i| self.basis_at(i))
    }

    fn rotmode(&self, address: u64) -> i16 {
        self.index.get(&address).map_or(1, |&i| self.rotmode[i])
    }

    fn rig(&self, address: u64) -> Option<BRigSpace<'_>> {
        self.index.get(&address).and_then(|&i| self.rig_at(i))
    }

    fn path(&self, address: u64) -> Option<BPathAt<'_>> {
        let &i = self.index.get(&address)?;
        self.list[i].path.as_ref().map(|p| p.at(self.frame))
    }
}
