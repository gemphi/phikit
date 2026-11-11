//! Running a tree: `iksolver_execute_tree`, `execute_posetree` and laying the change back.
use crate::device::blend::rig::ik::*;

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
    pub(super) fn where_is_ik_bone(&mut self, b: usize, dmat: Mat4, ik: Mat3) {
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
    pub(super) fn execute_posetree(&mut self, tree: &BIkTree) -> Option<Vec<Mat3>> {
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
        let owner = super::super::space::BSpaceOwner { arm: None, custom: Mat4::IDENTITY };
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
            if !matches!(data.pole, super::super::constraint::BTarget::None) {
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
