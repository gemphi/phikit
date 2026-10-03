//! The directions and rotations of a bevel list's points: bisected
//! directions, then a rotation per point by the curve's twist method
//! (`make_bevel_list_3D_minimum_twist`, `_tangent`, `_zup`), smoothed and
//! tilted — or, for a flat curve, tracked with Z up (`make_bevel_list_2D`).

use super::bevel::BBevelList;
use super::{CU_TWIST_MINIMUM, CU_TWIST_TANGENT};
use crate::blend::rig::math::{BQuat, BVec3};
use glam::Vec3;

impl BBevelList {
    /// Indices in Blender's walking order: the last point, then the
    /// first onwards (`bevp1 = last, bevp2 = first`).
    fn walk(&self) -> impl Iterator<Item = (usize, usize, usize)> + '_ {
        let n = self.points.len();
        (0..n).map(move |k| {
            let i1 = (k + n - 1) % n;
            ((i1 + n - 1) % n, i1, k)
        })
    }

    /// `bevel_list_calc_bisect`.
    fn bisect(&mut self) {
        let n = self.points.len();
        let triples: Vec<(usize, usize, usize)> = if self.cyclic { self.walk().collect() } else { (1..n - 1).map(|i| (i - 1, i, i + 1)).collect() };
        for (a, b, c) in triples {
            let dir = self.points[b].vec.bisect(self.points[a].vec, self.points[c].vec);
            self.points[b].dir = dir;
        }
        if !self.cyclic {
            for (end, near) in [(0, 1), (n - 1, n - 2)] {
                if self.points[end].dir == Vec3::ZERO {
                    let mut d = if end == 0 { self.points[near].vec - self.points[end].vec } else { self.points[end].vec - self.points[near].vec };
                    if d.normalize_len() == 0.0 {
                        d = self.points[near].dir;
                    }
                    self.points[end].dir = d;
                }
            }
        }
    }

    /// `make_bevel_list_3D`: by the twist method, smoothed, then tilted.
    pub(super) fn orient(&mut self, twist_mode: u16, smooth: usize) {
        match twist_mode {
            CU_TWIST_TANGENT => self.tangent(),
            CU_TWIST_MINIMUM => self.minimum_twist(),
            _ => {
                self.bisect();
                for p in &mut self.points {
                    p.quat = BQuat::of_track(p.dir, 5, 1).0;
                }
            }
        }
        if smooth > 0 {
            self.smooth(smooth);
        }
        for p in &mut self.points {
            p.quat = BQuat::from_axis_angle(p.dir, p.tilt).then(BQuat(p.quat)).normalized().0;
        }
    }

    /// `minimum_twist_between_two_points`.
    fn twist_from(&mut self, cur: usize, prev: usize) {
        let (pd, cd, pq) = (self.points[prev].dir, self.points[cur].dir, self.points[prev].quat);
        let angle = pd.angle_normalized(cd);
        self.points[cur].quat = if angle > 0.0 { BQuat::from_axis_angle(pd.cross(cd), angle).then(BQuat(pq)).0 } else { pq };
    }

    /// `make_bevel_list_3D_minimum_twist`.
    fn minimum_twist(&mut self) {
        let n = self.points.len();
        let nr_init = n - if self.cyclic { 1 } else { 2 };
        self.bisect();
        for (step, (b0, b1, _)) in self.walk().enumerate().collect::<Vec<_>>() {
            let nr = n - 1 - step;
            if nr >= nr_init {
                self.points[b1].quat = BQuat::of_track(self.points[b1].dir, 5, 1).0;
            } else {
                self.twist_from(b1, b0);
            }
        }
        if self.cyclic {
            let (first, last) = (n - 1, n - 2);
            let mut v1 = BQuat(self.points[first].quat).rotate(Vec3::Y);
            let mut v2 = BQuat(self.points[last].quat).rotate(Vec3::Y);
            v1.normalize_len();
            v2.normalize_len();
            let (fd, ld) = (self.points[first].dir, self.points[last].dir);
            v2 = BQuat::from_axis_angle(ld.cross(fd), fd.angle_normalized(ld)).rotate(v2);
            let mut angle = v1.angle_normalized(v2);
            let mut cross = v1.cross(v2);
            cross.normalize_len();
            if fd.angle_normalized(cross) < 90f32.to_radians() {
                angle = -angle;
            }
            for (step, (_, b1, _)) in self.walk().enumerate().collect::<Vec<_>>() {
                let nr = n - 1 - step;
                let fac = angle * (1.0 - nr as f32 / n as f32);
                self.points[b1].quat = BQuat::from_axis_angle(self.points[b1].dir, fac).then(BQuat(self.points[b1].quat)).0;
            }
        } else {
            self.twist_from(0, 1);
            self.twist_from(n - 1, n - 2);
        }
    }

    /// `make_bevel_list_3D_tangent`.
    fn tangent(&mut self) {
        self.bisect();
        // `bevel_list_flip_tangents`.
        for (b0, b1, _) in self.walk().collect::<Vec<_>>() {
            if self.points[b0].tan.angle_normalized(self.points[b1].tan) > 90f32.to_radians() {
                self.points[b1].tan = -self.points[b1].tan;
            }
        }
        for p in &mut self.points {
            let mut t = p.tan.cross(p.dir).cross(p.dir);
            t.normalize_len();
            p.tan = t;
        }
        for p in &mut self.points {
            let mut c = p.tan.cross(p.dir);
            c.normalize_len();
            p.quat = Self::tri_quat(c, p.tan);
        }
    }

    /// `tri_to_quat(q, zero, a, b)`: the rotation of the triangle
    /// `(0, a, b)`'s plane and first edge.
    fn tri_quat(a: Vec3, b: Vec3) -> [f32; 4] {
        // `normal_tri_v3(0, a, b)`.
        let mut vec = (Vec3::ZERO - a).cross(a - b);
        vec.normalize_len();
        let mut n = Vec3::new(vec.y, -vec.x, 0.0);
        n.normalize_len();
        if n.x == 0.0 && n.y == 0.0 {
            n.x = 1.0;
        }
        let (si, co) = (-0.5 * vec.z.clamp(-1.0, 1.0).acos()).sin_cos();
        let q1 = BQuat([co, n.x * si, n.y * si, 0.0]);
        let mut edge = q1.to_mat3().inverse() * a;
        edge.z = 0.0;
        edge.normalize_len();
        let (si, co) = (0.5 * edge.y.atan2(edge.x)).sin_cos();
        q1.then(BQuat([co, 0.0, 0.0, si])).0
    }

    /// `bevel_list_smooth`.
    fn smooth(&mut self, iterations: usize) {
        let n = self.points.len();
        for _ in 0..iterations {
            // Blender's walk, skipping the first two for an open list.
            let (mut b1, mut b2, mut left) = (n - 1, 0usize, n);
            let mut b0 = if n >= 2 { n - 2 } else { 0 };
            if !self.cyclic {
                b1 = b2;
                b2 += 1;
                left -= 1;
                b0 = b1;
                b1 = b2;
                b2 += 1;
                left -= 1;
            }
            let mut q0 = self.points[b0].quat;
            for _ in 0..left {
                let q = BQuat::interp(BQuat(q0), BQuat(self.points[b2 % n].quat), 0.5).normalized();
                let z = q.rotate(Vec3::Z);
                let q2 = BQuat::from_axis_angle(z.cross(self.points[b1].dir), z.angle_normalized(self.points[b1].dir)).normalized();
                q0 = self.points[b1].quat;
                let target = q2.then(q);
                self.points[b1].quat = BQuat::interp(BQuat(self.points[b1].quat), target, 0.5).normalized().0;
                b1 = b2 % n;
                b2 += 1;
            }
        }
    }

    /// `make_bevel_list_segment_3D`: two points share one direction.
    pub(super) fn segment(&mut self) {
        let dir = (self.points[1].vec - self.points[0].vec).normalize_or_zero();
        for p in &mut self.points {
            p.dir = dir;
            p.quat = BQuat::from_axis_angle(dir, p.tilt).then(BQuat::of_track(dir, 5, 1)).normalized().0;
        }
    }

    /// `make_bevel_list_2D`: bisected directions tracked with Z up, the
    /// open ends by their own directions.
    pub(super) fn flat(&mut self) {
        let n = self.points.len();
        let triples: Vec<(usize, usize, usize)> = if self.cyclic { self.walk().collect() } else { (1..n - 1).map(|i| (i - 1, i, i + 1)).collect() };
        for (a, b, c) in triples {
            let dir = self.points[b].vec.bisect(self.points[a].vec, self.points[c].vec);
            self.points[b].dir = dir;
            self.points[b].quat = BQuat::of_track(dir, 5, 1).0;
        }
        if !self.cyclic {
            for i in [0, n - 1] {
                self.points[i].quat = BQuat::of_track(self.points[i].dir, 5, 1).0;
            }
        }
    }

    /// STEP 3 for a flat closed spline alone: turned to run one way
    /// round (`sd->dir`), as Blender orders an outline that is no hole.
    pub(super) fn reverse_2d(&mut self) {
        if !self.cyclic {
            return;
        }
        let p = &self.points;
        let n = p.len();
        let mut left = 0;
        for i in 0..n {
            if p[left].vec.x > p[i].vec.x {
                left = i;
            }
        }
        let prev = if left == 0 { n - 1 } else { left - 1 };
        let next = if left == n - 1 { 0 } else { left + 1 };
        let (b0, b1, b2) = (p[prev].vec, p[left].vec, p[next].vec);
        let inp = (b1.x - b0.x) * (b0.y - b2.y) + (b0.y - b1.y) * (b0.x - b2.x);
        // `hole` (0) == `dir` (0): reversed.
        if inp <= 0.0 {
            self.points.reverse();
        }
    }
}
