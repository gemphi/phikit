//! Inverse kinematics with Blender's legacy solver
//! (`ikplugin/intern/iksolver_plugin.cc` over `intern/iksolver`): the IK
//! constraints gathered into trees at their chain roots
//! (`initialize_posetree`), each tree's bones as solver segments
//! (`execute_posetree`), solved, and the change laid back onto the pose
//! (`make_dmats`, `where_is_ik_bone`).

mod jacobian;
mod joint;
mod segment;
mod solver;

use super::constraint::{BConstraintKind, BIkData, CONSTRAINT_DISABLE};
use super::math::{BMat3, BMat4, BQuat, BVec3};
use super::solve::BPoseWork;
use super::space::SPACE_WORLD;
use super::{BRig, BRigState};
use glam::{DMat3, DVec3, Mat3, Mat4, Vec3};
use segment::{BJoint, BSegment};
use solver::BIkSolver;

/// `CONSTRAINT_IK_*`.
const IK_TIP: u32 = 1 << 0;
const IK_ROT: u32 = 1 << 1;
const IK_AUTO: u32 = 1 << 2;
const IK_STRETCH: u32 = 1 << 4;
const IK_POS: u32 = 1 << 5;
const IK_SETANGLE: u32 = 1 << 6;
/// `BONE_IK_NO_?DOF` (and `_TEMP`), `BONE_IK_?LIMIT`.
const NO_DOF: [u32; 3] = [1 << 0, 1 << 1, 1 << 2];
const NO_DOF_TEMP: [u32; 3] = [1 << 10, 1 << 11, 1 << 12];
const LIMIT: [u32; 3] = [1 << 3, 1 << 4, 1 << 5];
/// `IK_STRETCH_STIFF_EPS` / `_MIN` / `_MAX`.
const STRETCH_STIFF_EPS: f64 = 0.01;
const STRETCH_STIFF_MIN: f64 = 0.001;
const STRETCH_STIFF_MAX: f64 = 1e10;

/// A goal in a tree: the constraint and the chain bone it pulls.
#[derive(Clone, Debug)]
pub struct BIkTarget {
    pub bone: usize,
    pub con: usize,
    pub tip: usize,
}

/// `PoseTree`: bones root first, each one's parent within the tree.
#[derive(Clone, Debug)]
pub struct BIkTree {
    pub bones: Vec<usize>,
    pub parent: Vec<i32>,
    pub targets: Vec<BIkTarget>,
    pub iterations: i32,
    pub stretch: bool,
}

impl BRig {
    fn ik_data(&self, bone: usize, con: usize) -> Option<&BIkData> {
        match &self.bones[bone].constraints[con].kind {
            BConstraintKind::Ik(d) => Some(d),
            _ => None,
        }
    }

    /// `iksolver_initialize_tree`: every IK constraint's chain, gathered
    /// into trees at their roots, in evaluation order; and each chain
    /// bone's tree.
    pub fn ik_trees(&self, state: &BRigState) -> (Vec<BIkTree>, Vec<Option<usize>>) {
        let mut by_root: Vec<(usize, Vec<BIkTree>)> = Vec::new();
        for owner in 0..self.bones.len() {
            let cons: Vec<usize> = (0..self.bones[owner].constraints.len())
                .filter(|&c| {
                    let con = &self.bones[owner].constraints[c];
                    let Some(d) = self.ik_data(owner, c) else { return false };
                    // `find_ik_constraints`.
                    d.flag & IK_AUTO != 0 || (!matches!(d.target, super::constraint::BTarget::None | super::constraint::BTarget::Rig(None)) && con.flag & CONSTRAINT_DISABLE == 0)
                })
                .collect();
            let mut tip = owner;
            for c in cons {
                let d = self.ik_data(owner, c).expect("an IK constraint");
                if d.flag & IK_TIP == 0 {
                    if let Some(p) = self.bones[tip].parent {
                        tip = p;
                    }
                }
                let mut chain = Vec::new();
                let mut cur = Some(tip);
                while let Some(b) = cur {
                    chain.push(b);
                    if chain.len() as i32 == d.rootbone || chain.len() > 255 {
                        break;
                    }
                    cur = self.bones[b].parent;
                }
                let root = *chain.last().expect("a chain");
                let trees = match by_root.iter().position(|(r, _)| *r == root) {
                    Some(i) => &mut by_root[i].1,
                    None => {
                        by_root.push((root, Vec::new()));
                        &mut by_root.last_mut().expect("just pushed").1
                    }
                };
                // A tree some of whose targets lie off this chain takes it.
                let mut in_chain: Vec<usize> = chain.clone();
                let found = trees.iter().position(|tree| {
                    tree.targets.iter().any(|t| {
                        let b = tree.bones[t.tip];
                        match in_chain.iter().position(|&x| x == b) {
                            Some(i) => {
                                in_chain.remove(i);
                                false
                            }
                            None => true,
                        }
                    })
                });
                let seg = chain.len();
                let target_tip;
                match found {
                    None => {
                        let bones: Vec<usize> = chain.iter().rev().copied().collect();
                        let parent = (0..seg as i32).map(|a| a - 1).collect();
                        target_tip = seg - 1;
                        trees.push(BIkTree { bones, parent, targets: Vec::new(), iterations: d.iterations, stretch: d.flag & IK_STRETCH != 0 });
                    }
                    Some(i) => {
                        let mut tree = trees.remove(i);
                        tree.iterations = tree.iterations.max(d.iterations);
                        tree.stretch = tree.stretch && d.flag & IK_STRETCH == 0;
                        let size = seg.min(tree.bones.len());
                        let (mut a, mut t) = (0, 0);
                        while a < size && t < tree.bones.len() {
                            while t < tree.bones.len() && tree.bones[t] != chain[seg - a - 1] {
                                t += 1;
                            }
                            if t >= tree.bones.len() {
                                break;
                            }
                            while a < size && t < tree.bones.len() && tree.bones[t] == chain[seg - a - 1] {
                                a += 1;
                                t += 1;
                            }
                        }
                        let rest = seg - a;
                        target_tip = tree.bones.len() + rest - 1;
                        if rest > 0 {
                            let total = tree.bones.len();
                            let first_parent = self.bones[chain[rest - 1]].parent;
                            let parent = (a as i32 - 1).max(0) as usize;
                            let parent = (parent..total).find(|&p| Some(tree.bones[p]) == first_parent).unwrap_or(a.saturating_sub(1));
                            for k in 0..rest {
                                tree.bones.push(chain[rest - k - 1]);
                                tree.parent.push((total + k) as i32 - 1);
                            }
                            tree.parent[total] = parent as i32;
                        }
                        trees.push(tree);
                    }
                }
                let tree = trees.last_mut().expect("a tree");
                tree.targets.push(BIkTarget { bone: owner, con: c, tip: target_tip });
                // One active IK constraint per bone.
                if state.influence[owner][c] != 0.0 && !state.muted[owner][c] {
                    break;
                }
            }
        }
        let trees: Vec<BIkTree> = by_root.into_iter().flat_map(|(_, t)| t).collect();
        let mut tree_of = vec![None; self.bones.len()];
        for (i, t) in trees.iter().enumerate() {
            for &b in &t.bones {
                tree_of[b].get_or_insert(i);
            }
        }
        (trees, tree_of)
    }
}

/// `IK_CreateSegment`'s joints from the degrees of freedom.
struct BJoints;

impl BJoints {
    /// `CreateSegment`: the joint for `flag`'s three bits.
    fn joint(flag: u32, translate: bool) -> Option<BJoint> {
        let axes: Vec<usize> = (0..3).filter(|&i| flag & (1 << i) != 0).collect();
        Some(match (axes.len(), translate) {
            (0, _) => return None,
            (_, true) => BSegment::translate(&axes),
            (1, false) => BJoint::Revolute { axis: axes[0], angle: 0.0, new_angle: 0.0, limit: false, min: 0.0, max: 0.0 },
            (2, false) if axes == [0, 2] => BJoint::Swing { new_basis: DMat3::IDENTITY, limit_x: false, limit_z: false, min: [0.0; 2], max: [0.0; 2] },
            (2, false) => BJoint::Elbow {
                axis: if axes[0] == 0 { 0 } else { 2 },
                twist: 0.0,
                angle: 0.0,
                new_twist: 0.0,
                new_angle: 0.0,
                cos_twist: 1.0,
                sin_twist: 0.0,
                limit: false,
                limit_twist: false,
                min: 0.0,
                max: 0.0,
                min_twist: 0.0,
                max_twist: 0.0,
            },
            _ => BJoint::Spherical { new_basis: DMat3::IDENTITY, limit: [false; 3], min: [0.0; 2], max: [0.0; 2], min_y: 0.0, max_y: 0.0, locked_a: DVec3::ZERO },
        })
    }
}

impl BIkSolver {
    /// `IK_CreateSegment`: the rotation joint, with the translation joint
    /// as its composite; returns the segment the tree refers to.
    fn create(&mut self, flag: u32) -> usize {
        let rot = BJoints::joint(flag, false);
        let trans = BJoints::joint(flag >> 3, true);
        match (rot, trans) {
            (None, None) => self.push(BSegment::new(BJoint::Null)),
            (None, Some(t)) => self.push(BSegment::new(t)),
            (Some(r), None) => self.push(BSegment::new(r)),
            (Some(r), Some(t)) => {
                let r = self.push(BSegment::new(r));
                let t = self.push(BSegment::new(t));
                self.segs[r].composite = Some(t);
                self.set_parent(t, r);
                r
            }
        }
    }

    fn push(&mut self, s: BSegment) -> usize {
        self.segs.push(s);
        self.segs.len() - 1
    }

    /// The segment a goal or limit on `seg`'s translation lands on.
    fn translational(&self, seg: usize) -> Option<usize> {
        if self.segs[seg].translational {
            Some(seg)
        } else {
            self.segs[seg].composite.filter(|&c| self.segs[c].translational)
        }
    }

    /// `IK_SetLimit`: axes 0–2 rotate, 3–5 translate.
    fn limit(&mut self, seg: usize, axis: usize, lo: f64, hi: f64) {
        if axis >= 3 {
            if let Some(t) = self.translational(seg) {
                self.segs[t].set_limit(axis - 3, lo, hi);
            }
        } else {
            self.segs[seg].set_limit(axis, lo, hi);
        }
    }

    /// `IK_SetStiffness`.
    fn stiffness(&mut self, seg: usize, axis: usize, stiffness: f64) {
        if stiffness < 0.0 {
            return;
        }
        let weight = 1.0 - stiffness.min(1.0 - STRETCH_STIFF_EPS);
        if axis >= 3 {
            if let Some(t) = self.translational(seg) {
                self.segs[t].set_weight(axis - 3, weight);
            }
        } else {
            self.segs[seg].set_weight(axis, weight);
        }
    }

    /// The segment goals attach to: the composite, when there is one.
    fn tip(&self, seg: usize) -> usize {
        self.segs[seg].composite.unwrap_or(seg)
    }
}

impl BPoseWork<'_> {
    /// `iksolver_execute_tree` for one tree: the chain posed as without IK,
    /// then solved and laid back.
    pub fn execute_tree(&mut self, tree: &BIkTree) {
        let influenced = tree.targets.iter().any(|t| !self.state.muted[t.bone][t.con] && self.state.influence[t.bone][t.con] != 0.0);
        for &b in &tree.bones {
            if !self.done[b] {
                self.where_is_bone(b);
            }
        }
        if !influenced {
            return;
        }
        let Some(changes) = self.execute_posetree(tree) else { return };
        let dmats: Vec<Mat4> = tree
            .bones
            .iter()
            .map(|&b| match self.rig.bones[b].parent {
                Some(p) => self.pose[p].inverse() * self.pose[b],
                None => self.pose[b],
            })
            .collect();
        for (a, &b) in tree.bones.iter().enumerate() {
            self.where_is_ik_bone(b, dmats[a], changes[a]);
            self.chan[b] = self.pose[b] * self.rig.bones[b].arm.inverse();
            self.done[b] = true;
        }
    }

    /// `where_is_ik_bone`, with the non-uniform scale kept as Blender keeps
    /// it (`USE_NONUNIFORM_SCALE`).
    fn where_is_ik_bone(&mut self, b: usize, dmat: Mat4, ik: Mat3) {
        let mut pose = match self.rig.bones[b].parent {
            Some(p) => self.pose[p] * dmat,
            None => dmat,
        };
        let scale = pose.size();
        let set_len = |v: glam::Vec4, len: f32| {
            let mut t = v.truncate();
            t.normalize_len();
            (t * len).extend(v.w)
        };
        pose.x_axis = set_len(pose.x_axis, scale.y);
        pose.z_axis = set_len(pose.z_axis, scale.y);
        pose *= Mat4::from_mat3(ik);
        let ik_scale = ik.size3();
        pose.x_axis = set_len(pose.x_axis, scale.x * ik_scale.x);
        pose.z_axis = set_len(pose.z_axis, scale.z * ik_scale.z);
        self.set_pose(b, pose);
    }

    /// `execute_posetree`: each bone's basis change, `None` when the solver
    /// had nothing to do.
    fn execute_posetree(&mut self, tree: &BIkTree) -> Option<Vec<Mat3>> {
        let n = tree.bones.len();
        let mut solver = BIkSolver::new(Vec::new());
        let mut iktree = Vec::with_capacity(n);
        let mut hasstretch = false;
        for a in 0..n {
            let b = tree.bones[a];
            let bone = &self.rig.bones[b];
            let mut flag = 0;
            for k in 0..3 {
                if bone.ikflag & NO_DOF[k] == 0 && bone.ikflag & NO_DOF_TEMP[k] == 0 {
                    flag |= 1 << k;
                }
            }
            if tree.stretch && bone.ikstretch > 0.0 {
                flag |= 16;
                hasstretch = true;
            }
            let seg = solver.create(flag);
            iktree.push(seg);
            if a > 0 {
                let parent = iktree[tree.parent[a] as usize];
                let parent = solver.segs[parent].composite.unwrap_or(parent);
                solver.set_parent(seg, parent);
            }
            let pose = self.pose[b];
            let mut r_bone = Mat3::from_mat4(pose);
            let (mut r_par, start) = match bone.parent {
                Some(p) => (Mat3::from_mat4(self.pose[p]), if a > 0 { pose.loc() - self.tail[p] } else { Vec3::ZERO }),
                None => (Mat3::IDENTITY, Vec3::ZERO),
            };
            let length = bone.length * r_bone.y_axis.length();
            r_bone = r_bone.normalized_cols();
            r_par = r_par.normalized_cols();
            let ir_par = r_par.inverse();
            let basis = bone.bone_mat.transpose() * (ir_par * r_bone);
            let start = ir_par * start;
            let (s, rest, basis) = (start.as_dvec3(), bone.bone_mat.as_dmat3(), basis.as_dmat3());
            match solver.segs[seg].composite {
                Some(c) => {
                    solver.segs[seg].set_transform(s, rest, basis, 0.0);
                    solver.segs[c].set_transform(DVec3::ZERO, DMat3::IDENTITY, DMat3::IDENTITY, length as f64);
                }
                None => solver.segs[seg].set_transform(s, rest, basis, length as f64),
            }
            for k in 0..3 {
                if bone.ikflag & LIMIT[k] != 0 {
                    solver.limit(seg, k, bone.limit_min[k] as f64, bone.limit_max[k] as f64);
                }
            }
            for k in 0..3 {
                solver.stiffness(seg, k, bone.stiffness[k] as f64);
            }
            if tree.stretch && bone.ikstretch > 0.0 {
                solver.stiffness(seg, 4, 1.0 - (bone.ikstretch * bone.ikstretch) as f64);
                solver.limit(seg, 4, STRETCH_STIFF_MIN, STRETCH_STIFF_MAX);
            }
        }
        // The goals, in the space of the root's parent at the root's head.
        let root = tree.bones[0];
        let mut rootmat = match self.rig.bones[root].parent {
            Some(p) => {
                let mut m = self.pose[p];
                m.normalize_ex();
                m
            }
            None => Mat4::IDENTITY,
        };
        rootmat.set_loc(self.pose[root].loc());
        let goalinv = (self.ob * rootmat).inverse();
        let owner = super::space::BSpaceOwner { arm: None, custom: Mat4::IDENTITY };
        let (mut resultblend, mut resultinf) = (false, 0.0f32);
        for t in &tree.targets {
            let con = &self.rig.bones[t.bone].constraints[t.con];
            let BConstraintKind::Ik(data) = &con.kind else { continue };
            let (muted, influence) = (self.state.muted[t.bone][t.con], self.state.influence[t.bone][t.con]);
            let target = self.target_matrix(&data.target, &owner, SPACE_WORLD, con.headtail).unwrap_or(self.ob);
            let goal = goalinv * target;
            let mut goalpos = goal.loc();
            let mut goalrot = Mat3::from_mat4(goal).normalized_cols();
            let mut pole = None;
            if !matches!(data.pole, super::constraint::BTarget::None) {
                let m = self.target_matrix(&data.pole, &owner, SPACE_WORLD, con.headtail).unwrap_or(self.ob);
                if data.flag & IK_SETANGLE != 0 {
                    break;
                }
                pole = Some((goalinv * m).loc());
                resultblend = true;
                resultinf = if muted { 0.0 } else { influence };
            }
            if !resultblend && (muted || influence != 1.0) {
                let fac = if muted { 0.0 } else { influence };
                let tip = tree.bones[t.tip];
                let mut end = self.pose[tip];
                end.set_loc(self.tail[tip]);
                let world = goalinv * self.ob * end;
                goalpos = goalpos * fac + world.loc() * (1.0 - fac);
                let q = BQuat::interp(BQuat::of_mat3(&goalrot), BQuat::of_mat3(&Mat3::from_mat4(world)), 1.0 - fac);
                goalrot = q.to_mat3();
            }
            let iktarget = solver.tip(iktree[t.tip]);
            if data.flag & IK_POS != 0 && data.weight != 0.0 {
                if let Some(polepos) = pole {
                    solver.set_pole(iktarget, goalpos.as_dvec3(), polepos.as_dvec3(), data.poleangle as f64);
                }
                solver.add_position(iktarget, goalpos.as_dvec3(), data.weight as f64);
            }
            if data.flag & IK_ROT != 0 && data.orientweight != 0.0 && data.flag & IK_AUTO == 0 {
                solver.add_orientation(iktarget, goalrot.as_dmat3(), data.orientweight as f64);
            }
        }
        solver.solve(tree.iterations);
        let mut stretch: Vec<f32> = vec![1.0; n];
        let mut out = Vec::with_capacity(n);
        for a in 0..n {
            let seg = iktree[a];
            let rot = if solver.segs[seg].translational { solver.segs[seg].composite.unwrap_or(seg) } else { seg };
            let mut change = solver.segs[rot].basis_change().as_mat3();
            if hasstretch {
                let parentstretch = if tree.parent[a] >= 0 { stretch[tree.parent[a] as usize] } else { 1.0 };
                let bone = &self.rig.bones[tree.bones[a]];
                stretch[a] = if tree.stretch && bone.ikstretch > 0.0 {
                    let tseg = solver.translational(seg).unwrap_or(seg);
                    let trans = solver.segs[tseg].translation_change();
                    let length = bone.length * self.pose[tree.bones[a]].y_axis.truncate().length();
                    if length == 0.0 { 1.0 } else { (trans.y as f32 + length) / length }
                } else {
                    1.0
                };
                let s = if parentstretch == 0.0 { 1.0 } else { stretch[a] / parentstretch };
                change *= s;
            }
            if resultblend && resultinf != 1.0 {
                change = Mat3::IDENTITY.blend(&change, resultinf);
            }
            out.push(change);
        }
        Some(out)
    }
}
