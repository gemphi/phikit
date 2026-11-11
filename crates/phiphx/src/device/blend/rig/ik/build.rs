//! The solver's segments from a tree's bones (`IK_CreateSegment` and friends).
use super::*;

/// `IK_CreateSegment`'s joints from the degrees of freedom.
pub(super) struct BJoints;

impl BJoints {
    /// `CreateSegment`: the joint for `flag`'s three bits.
    pub(super) fn joint(flag: u32, translate: bool) -> Option<BJoint> {
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
    pub(super) fn create(&mut self, flag: u32) -> usize {
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

    pub(super) fn push(&mut self, s: BSegment) -> usize {
        self.segs.push(s);
        self.segs.len() - 1
    }

    /// The segment a goal or limit on `seg`'s translation lands on.
    pub(super) fn translational(&self, seg: usize) -> Option<usize> {
        if self.segs[seg].translational {
            Some(seg)
        } else {
            self.segs[seg].composite.filter(|&c| self.segs[c].translational)
        }
    }

    /// `IK_SetLimit`: axes 0–2 rotate, 3–5 translate.
    pub(super) fn limit(&mut self, seg: usize, axis: usize, lo: f64, hi: f64) {
        if axis >= 3 {
            if let Some(t) = self.translational(seg) {
                self.segs[t].set_limit(axis - 3, lo, hi);
            }
        } else {
            self.segs[seg].set_limit(axis, lo, hi);
        }
    }

    /// `IK_SetStiffness`.
    pub(super) fn stiffness(&mut self, seg: usize, axis: usize, stiffness: f64) {
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
    pub(super) fn tip(&self, seg: usize) -> usize {
        self.segs[seg].composite.unwrap_or(seg)
    }
}
