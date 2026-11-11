//! The spline parameters (`BBoneSplineParameters`) and the matrices computed from them.
use super::*;

/// `BBoneSplineParameters`.
#[derive(Clone, Debug, Default)]
pub(super) struct Spline {
    pub(super) segments: usize,
    pub(super) length: f32,
    pub(super) do_scale: bool,
    pub(super) scale: Vec3,
    pub(super) use_prev: bool,
    pub(super) prev_h: Vec3,
    pub(super) prev_bbone: bool,
    pub(super) prev_mat: Mat4,
    pub(super) use_next: bool,
    pub(super) next_h: Vec3,
    pub(super) next_bbone: bool,
    pub(super) next_mat: Mat4,
    pub(super) ease1: f32,
    pub(super) ease2: f32,
    pub(super) roll1: f32,
    pub(super) roll2: f32,
    pub(super) scale_in: Vec3,
    pub(super) scale_out: Vec3,
    pub(super) curve_in_x: f32,
    pub(super) curve_in_z: f32,
    pub(super) curve_out_x: f32,
    pub(super) curve_out_z: f32,
}

impl Spline {
    /// `BKE_pchan_bbone_handles_compute`.
    pub(super) fn handles(&self) -> (Vec3, f32, Vec3, f32) {
        let mut length = self.length;
        let epsilon = 1e-5 * length;
        if self.do_scale {
            length *= self.scale.y;
        }
        let (mut roll1, mut roll2) = (0.0f32, 0.0f32);
        let mut h1 = if self.use_prev {
            let mut h = self.prev_h;
            if self.prev_bbone {
                h.y -= length;
            }
            h = if h.length() < epsilon { Vec3::NEG_Y } else { h.normalize() };
            h = -h;
            if !self.prev_bbone {
                roll1 = mat3_vec_to_roll(Mat3::from_mat4(self.prev_mat), h);
            }
            h
        } else {
            Vec3::Y
        };
        let mut h2 = if self.use_next {
            let mut h = self.next_h;
            if !self.next_bbone {
                h.y -= length;
            }
            h = if h.length() < epsilon { Vec3::Y } else { h.normalize() };
            roll2 = mat3_vec_to_roll(Mat3::from_mat4(self.next_mat), h);
            h
        } else {
            Vec3::Y
        };
        let circle = length * (circle_factor(h1, h2) / 0.75);
        h1 *= self.ease1 * circle;
        h2 *= -(self.ease2 * circle);
        roll1 += self.roll1;
        roll2 += self.roll2;
        let (xs, zs) = if self.do_scale { (self.scale.x, self.scale.z) } else { (1.0, 1.0) };
        h1.x += self.curve_in_x * xs;
        h1.z += self.curve_in_z * zs;
        h2.x += self.curve_out_x * xs;
        h2.z += self.curve_out_z * zs;
        (h1, roll1, h2, roll2)
    }

    /// `BKE_pchan_bbone_spline_compute` with `for_deform = true`:
    /// `segments + 1` matrices at the cuts between segments.
    pub(super) fn compute(&mut self) -> Vec<Mat4> {
        let mut length = self.length;
        let scalemats = self.do_scale.then(|| {
            length *= self.scale.y;
            let up = Mat4::from_scale(self.scale);
            (up.inverse(), up)
        });
        let (h1, roll1, h2, roll2) = self.handles();
        self.segments = self.segments.min(MAX_SUBDIV);
        let end = Vec3::new(0.0, length, 0.0);
        let control = [Vec3::ZERO, h1, end + h2, end];

        self.scale_in.y = self.scale_in.y.max(0.0001);
        self.scale_out.y = self.scale_out.y.max(0.0001);
        let (log_in, log_out) = (self.scale_in.y.ln(), self.scale_out.y.ln());
        let scales: Vec<f32> = (0..self.segments)
            .map(|i| interpf(log_out, log_in, i as f32 / (self.segments - 1) as f32).exp())
            .collect();
        let points = equalize(control, MAX_SUBDIV, self.segments, &scales);

        let d1: [Vec3; 3] = std::array::from_fn(|i| control[i + 1] - control[i]);
        let d2: [Vec3; 2] = std::array::from_fn(|i| d1[i + 1] - d1[i]);
        let mut out = Vec::with_capacity(self.segments + 1);
        let mat = |pos: Vec3, axis: Vec3, roll: f32, sx: f32, sz: f32| {
            let mut m = Mat4::from_mat3(vec_roll_to_mat3(axis, roll));
            m.w_axis = pos.extend(1.0);
            if let Some((down, up)) = scalemats {
                m = down * m * up;
            }
            m.x_axis *= sx;
            m.z_axis *= sz;
            m
        };
        out.push(mat(control[0], ease_axis(d1[0], d2[0]), roll1, self.scale_in.x, self.scale_in.z));
        for a in 1..self.segments {
            let (cur, axis) = bezier_at(control, points[a]);
            let fac = a as f32 / self.segments as f32;
            out.push(mat(
                cur,
                axis,
                interpf(roll2, roll1, fac),
                interpf(self.scale_out.x, self.scale_in.x, fac),
                interpf(self.scale_out.z, self.scale_in.z, fac),
            ));
        }
        out.push(mat(control[3], ease_axis(d1[2], -d2[1]), roll2, self.scale_out.x, self.scale_out.z));
        out
    }
}

pub(super) fn apply_handle_scale(p: &mut Spline, flag: u32, scale: Vec3, start: bool) {
    let (s, ease, cx, cz) = if start {
        (&mut p.scale_in, &mut p.ease1, &mut p.curve_in_x, &mut p.curve_in_z)
    } else {
        (&mut p.scale_out, &mut p.ease2, &mut p.curve_out_x, &mut p.curve_out_z)
    };
    if flag & SCALE_X != 0 {
        s.x *= scale.x;
    }
    if flag & SCALE_Y != 0 {
        s.y *= scale.y;
    }
    if flag & SCALE_Z != 0 {
        s.z *= scale.z;
    }
    if flag & SCALE_EASE != 0 {
        *ease *= scale.y;
        *cx *= scale.y;
        *cz *= scale.y;
    }
}
