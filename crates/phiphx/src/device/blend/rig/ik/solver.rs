//! `IK_QJacobianSolver` with its tasks (`IK_QTask.cc`): position and
//! orientation goals on a chain of segments, solved by repeated SDLS steps
//! with joint limits locked in turn, a pole target turning the chain's
//! plane first.

use super::jacobian::BJacobian;
use super::segment::{BIkMath, BSegment};
use glam::{DMat3, DVec3};

/// A goal on a segment's tip.
#[derive(Clone, Debug)]
pub enum BTaskGoal {
    Position { goal: DVec3, clamp_length: f64 },
    Orientation { goal: DMat3 },
}

#[derive(Clone, Debug)]
pub struct BTask {
    pub goal: BTaskGoal,
    pub segment: usize,
    pub weight: f64,
    pub id: usize,
}

/// The solver over an arena of segments (index 0 the root).
#[derive(Clone, Debug)]
pub struct BIkSolver {
    pub segs: Vec<BSegment>,
    pub tasks: Vec<BTask>,
    order: Vec<usize>,
    jacobian: BJacobian,
    root_lin: DMat3,
    root_t: DVec3,
    pole: Option<(usize, DVec3, DVec3, f64)>,
}

impl BIkSolver {
    pub fn new(segs: Vec<BSegment>) -> Self {
        Self { segs, tasks: Vec::new(), order: Vec::new(), jacobian: BJacobian::default(), root_lin: DMat3::IDENTITY, root_t: DVec3::ZERO, pole: None }
    }

    /// `seg.parent = parent`, as `SetParent` links it (at the head of the
    /// parent's children).
    pub fn set_parent(&mut self, seg: usize, parent: usize) {
        self.segs[seg].parent = Some(parent);
        self.segs[parent].children.insert(0, seg);
    }

    /// `IK_QPositionTask`: its clamp length from the chain's reach.
    pub fn add_position(&mut self, tip: usize, goal: DVec3, weight: f64) {
        let (mut len, mut num) = (0.0, 0);
        let mut s = Some(tip);
        while let Some(i) = s {
            len += self.segs[i].max_extension;
            num += 1;
            s = self.segs[i].parent;
        }
        let clamp_length = len / (2 * num) as f64;
        self.tasks.push(BTask { goal: BTaskGoal::Position { goal, clamp_length }, segment: tip, weight, id: 0 });
    }

    pub fn add_orientation(&mut self, tip: usize, goal: DMat3, weight: f64) {
        self.tasks.push(BTask { goal: BTaskGoal::Orientation { goal }, segment: tip, weight, id: 0 });
    }

    /// `SetPoleVectorConstraint`.
    pub fn set_pole(&mut self, tip: usize, goal: DVec3, polegoal: DVec3, poleangle: f64) {
        self.pole = Some((tip, goal, polegoal, poleangle));
    }

    fn add_segment_list(&mut self, seg: usize) {
        self.order.push(seg);
        for c in self.segs[seg].children.clone() {
            self.add_segment_list(c);
        }
    }

    /// `UpdateTransform` from the root down.
    fn update_transform(&mut self, seg: usize, lin: DMat3, t: DVec3) {
        let s = &mut self.segs[seg];
        s.global_start = t + lin * s.start;
        s.global_lin = lin * s.rest_basis * s.basis;
        s.global_t = s.global_start + s.global_lin * s.translation;
        let (l, t) = (s.global_lin, s.global_t);
        for c in s.children.clone() {
            self.update_transform(c, l, t);
        }
    }

    /// `Setup`: DoF ids, task ids and weights; `false` with nothing to do.
    fn setup(&mut self) -> bool {
        self.order.clear();
        self.add_segment_list(0);
        let mut num_dof = 0;
        for &s in &self.order {
            self.segs[s].dof_id = num_dof;
            num_dof += self.segs[s].num_dof;
        }
        if num_dof == 0 {
            return false;
        }
        let (mut size, mut weight) = (0, 0.0);
        for t in &mut self.tasks {
            t.id = size;
            size += 3;
            weight += t.weight;
        }
        if size == 0 || BIkMath::fuzzy_zero(weight) {
            return false;
        }
        self.tasks.iter_mut().for_each(|t| t.weight /= weight);
        self.jacobian.arm(num_dof, size);
        for &s in &self.order {
            for i in 0..self.segs[s].num_dof {
                self.jacobian.set_dof_weight(self.segs[s].dof_id + i, self.segs[s].weight[i]);
            }
        }
        true
    }

    /// `Scale`.
    fn scale(&mut self, s: f64) {
        for t in &mut self.tasks {
            if let BTaskGoal::Position { goal, clamp_length } = &mut t.goal {
                *goal *= s;
                *clamp_length *= s;
            }
        }
        for &i in &self.order {
            self.segs[i].scale(s);
        }
        self.root_t *= s;
        if let Some((_, goal, polegoal, _)) = &mut self.pole {
            *goal *= s;
            *polegoal *= s;
        }
    }

    /// `ConstrainPoleVector`: turn the root so the chain bends towards the
    /// pole.
    fn constrain_pole(&mut self) {
        let Some((tip, goal, polegoal, poleangle)) = self.pole else { return };
        if self.tasks.iter().filter(|t| matches!(t.goal, BTaskGoal::Position { .. })).count() >= 2 {
            self.pole = None;
            return;
        }
        self.update_transform(0, self.root_lin, self.root_t);
        let rootpos = self.segs[0].global_start;
        let endpos = self.segs[tip].global_t;
        let rootbasis = self.segs[0].global_lin;
        let dir = BIkMath::normalize(endpos - rootpos);
        let (rootx, rootz) = (rootbasis.col(0), rootbasis.col(2));
        let up = rootx * poleangle.cos() + rootz * poleangle.sin();
        let poledir = BIkMath::normalize(goal - rootpos);
        let poleup = BIkMath::normalize(polegoal - rootpos);
        let frame = |dir: DVec3, up: DVec3| {
            let r0 = BIkMath::normalize(dir.cross(up));
            let r1 = r0.cross(dir);
            DMat3::from_cols(r0, r1, -dir).transpose()
        };
        let (mat, polemat) = (frame(dir, up), frame(poledir, poleup));
        self.root_lin = polemat.transpose() * mat * self.root_lin;
        self.root_t = polemat.transpose() * mat * self.root_t;
    }

    /// `ComputeJacobian` for each task.
    fn compute_jacobian(&mut self) {
        for t in &self.tasks {
            let seg = &self.segs[t.segment];
            match &t.goal {
                BTaskGoal::Position { goal, clamp_length } => {
                    let pos = seg.global_t;
                    let mut d = *goal - pos;
                    let len = d.length();
                    if len > *clamp_length {
                        d *= clamp_length / len;
                    }
                    self.jacobian.set_betas(t.id, d * t.weight);
                    let mut s = Some(t.segment);
                    while let Some(i) = s {
                        let seg = &self.segs[i];
                        let p = seg.global_start - pos;
                        for k in 0..seg.num_dof {
                            let axis = seg.axis(k) * t.weight;
                            if seg.translational {
                                self.jacobian.set_derivatives(t.id, seg.dof_id + k, axis, 1e2);
                            } else {
                                self.jacobian.set_derivatives(t.id, seg.dof_id + k, p.cross(axis), 1e0);
                            }
                        }
                        s = seg.parent;
                    }
                }
                BTaskGoal::Orientation { goal } => {
                    let d = (*goal * seg.global_lin.transpose()).transpose();
                    let at = |r: usize, c: usize| d.col(c)[r];
                    let d_rot = DVec3::new(at(2, 1) - at(1, 2), at(0, 2) - at(2, 0), at(1, 0) - at(0, 1)) * -0.5;
                    self.jacobian.set_betas(t.id, d_rot * t.weight);
                    let mut s = Some(t.segment);
                    while let Some(i) = s {
                        let seg = &self.segs[i];
                        for k in 0..seg.num_dof {
                            if seg.translational {
                                self.jacobian.set_derivatives(t.id, seg.dof_id + k, DVec3::ZERO, 1e2);
                            } else {
                                self.jacobian.set_derivatives(t.id, seg.dof_id + k, seg.axis(k) * t.weight, 1e0);
                            }
                        }
                        s = seg.parent;
                    }
                }
            }
        }
    }

    /// `UpdateAngles`: `true` when a limit locked a joint and the step must
    /// be solved again.
    fn update_angles(&mut self, norm: &mut f64) -> bool {
        let mut minseg = None;
        let (mut minabsdelta, mut mindelta, mut mindof) = (1e10, DVec3::ZERO, 0);
        let mut locked = false;
        for &s in &self.order.clone() {
            let (mut delta, mut clamp) = (DVec3::ZERO, [false; 3]);
            if !self.segs[s].update_angle(&self.jacobian, &mut delta, &mut clamp) {
                continue;
            }
            for i in 0..self.segs[s].num_dof {
                if clamp[i] && !self.segs[s].locked[i] {
                    let absdelta = delta[i].abs();
                    if absdelta < super::segment::IK_EPSILON {
                        self.segs[s].lock(i, &mut self.jacobian, &delta);
                        locked = true;
                    } else if absdelta < minabsdelta {
                        (minabsdelta, mindelta, minseg, mindof) = (absdelta, delta, Some(s), i);
                    }
                }
            }
        }
        if let Some(s) = minseg {
            self.segs[s].lock(mindof, &mut self.jacobian, &mindelta);
            locked = true;
            *norm = minabsdelta.max(*norm);
        }
        if !locked {
            for &s in &self.order {
                self.segs[s].unlock();
                self.segs[s].apply_angle();
            }
        }
        locked
    }

    /// `IK_Solve` → `Setup` and `Solve`.
    pub fn solve(&mut self, max_iterations: i32) -> bool {
        if !self.setup() {
            return false;
        }
        // `ComputeScale`, kept in single precision as Blender stores it.
        let length: f64 = self.order.iter().map(|&s| self.segs[s].max_extension).sum();
        let scale = if length == 0.0 { 1.0 } else { (1.0 / length) as f32 as f64 };
        self.scale(scale);
        self.constrain_pole();
        self.update_transform(0, self.root_lin, self.root_t);
        let mut solved = false;
        for iteration in 0..max_iterations {
            self.update_transform(0, self.root_lin, self.root_t);
            self.compute_jacobian();
            let mut norm = 0.0;
            loop {
                self.jacobian.invert();
                if !self.update_angles(&mut norm) {
                    break;
                }
            }
            for &s in &self.order {
                self.segs[s].unlock();
            }
            norm = self.jacobian.angle_update_norm().max(norm);
            if norm < 1e-3 && iteration > 10 {
                solved = true;
                break;
            }
        }
        if self.pole.is_some() {
            let lin = self.root_lin;
            self.segs[0].prepend_basis(&lin);
        }
        self.scale(1.0 / scale as f32 as f64);
        solved
    }
}
