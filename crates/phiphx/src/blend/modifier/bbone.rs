//! B-Bone segments — Blender `armature.cc` (`BKE_pchan_bbone_spline_compute`,
//! `BKE_pchan_bbone_segments_cache_compute`,
//! `find_bbone_segment_index_straight`) and `armature_deform.cc`
//! (`b_bone_deform`), ported.
//!
//! A bone with `segments > 1` does not deform rigidly: Blender fits a cubic
//! Bezier through the bone, cuts it into equal arcs, and builds one matrix per
//! cut. A vertex is placed by how far it lies along the bone and takes a blend
//! of the two matrices either side of it. Rigify uses this for nearly every
//! deform bone — spine, thigh, shin, face — so without it a character's
//! surface lands about a millimetre off wherever a bone is bent, which is more
//! than a garment's thickness.
//!
//! Not ported: the `BBONE_MAPPING_CURVED` segment lookup (Blender's BSP
//! boundary planes); a curved-mapping bone falls back to the straight one.

use crate::blend::file::types::*;
use glam::{Mat3, Mat4, Quat, Vec3};
use std::collections::HashMap;

/// `MAX_BBONE_SUBDIV`.
const MAX_SUBDIV: usize = 32;

const BONE_CONNECTED: u32 = 1 << 4;
const BBONE_HANDLE_AUTO: u8 = 0;
const BBONE_HANDLE_RELATIVE: u8 = 2;
const BBONE_HANDLE_TANGENT: u8 = 3;
const BBONE_ADD_PARENT_END_ROLL: u32 = 1 << 0;
const BBONE_SCALE_EASING: u32 = 1 << 1;
/// `BBONE_HANDLE_SCALE_X / _Y / _Z / _EASE`.
const SCALE_X: u32 = 1 << 0;
const SCALE_Y: u32 = 1 << 1;
const SCALE_Z: u32 = 1 << 2;
const SCALE_EASE: u32 = 1 << 3;

/// One bone's segment matrices, ready to deform with.
#[derive(Clone, Debug)]
pub(crate) struct BBone {
    /// `bbone_deform_mats[0]` — armature space to bone-local, for finding
    /// which segment a point belongs to.
    to_bone: Mat4,
    /// `bbone_deform_mats[1..]` — `segments + 1` matrices, armature space.
    mats: Vec<Mat4>,
    length: f32,
}

impl BBone {
    /// `find_bbone_segment_index_straight` + `..._deform_clamp_segment_index`:
    /// the first of the two segments this point falls between, and how far it
    /// leans towards the second.
    pub(crate) fn segment(&self, co: Vec3) -> (usize, f32) {
        let segments = self.mats.len() - 1;
        let y = self.to_bone.transform_point3(co).y;
        let head_tail = (y / self.length).clamp(0.0, 1.0);
        let pre = head_tail * segments as f32;
        let index = (pre.floor() as isize).clamp(0, segments as isize - 1) as usize;
        (index, (pre - index as f32).clamp(0.0, 1.0))
    }

    /// The matrix that deforms `co`. `b_bone_deform` adds the two segments'
    /// contributions with weights `1 − blend` and `blend`, and both are affine
    /// in the matrix, so the blended matrix gives the same answer.
    pub(crate) fn matrix(&self, co: Vec3) -> Mat4 {
        let (i, blend) = self.segment(co);
        let (a, b) = (self.mats[i], self.mats[i + 1]);
        Mat4::from_cols_array(&std::array::from_fn(|k| {
            let (x, y) = (a.to_cols_array()[k], b.to_cols_array()[k]);
            x + (y - x) * blend
        }))
    }
}

/// `interpf(target, origin, fac)`.
fn interpf(target: f32, origin: f32, fac: f32) -> f32 {
    fac * target + (1.0 - fac) * origin
}

/// `vec_roll_to_mat3_normalized`: the rotation taking +Y onto `nor`, then
/// `roll` radians about `nor`.
fn vec_roll_to_mat3(vec: Vec3, roll: f32) -> Mat3 {
    const SAFE: f32 = 6.1e-3;
    const CRITICAL: f32 = 2.5e-4;
    let nor = vec.normalize_or_zero();
    let (x, y, z) = (nor.x, nor.y, nor.z);
    let mut theta = 1.0 + y;
    let theta_alt = x * x + z * z;
    let b = if theta > SAFE || theta_alt > CRITICAL * CRITICAL {
        if theta <= SAFE {
            theta = theta_alt * 0.5 + theta_alt * theta_alt * 0.125;
        }
        // Blender writes `bMatrix[col][row]`, which is glam's column order.
        Mat3::from_cols(
            Vec3::new(1.0 - x * x / theta, -x, -x * z / theta),
            Vec3::new(x, y, z),
            Vec3::new(-x * z / theta, -z, 1.0 - z * z / theta),
        )
    } else {
        Mat3::from_cols(Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0), Vec3::Z)
    };
    Mat3::from_axis_angle(nor, roll) * b
}

/// `mat3_vec_to_roll`: the twist of `mat` about `vec`, as a roll angle.
fn mat3_vec_to_roll(mat: Mat3, vec: Vec3) -> f32 {
    let inv = vec_roll_to_mat3(vec, 0.0).inverse();
    let roll = inv * orthonormal(mat);
    let q = Quat::from_mat3(&orthonormal(roll));
    // `quat_split_swing_and_twist(q, axis = 1)` on a canonical quaternion.
    let (w, y) = if q.w < 0.0 { (-q.w, -q.y) } else { (q.w, q.y) };
    2.0 * y.atan2(w)
}

fn orthonormal(m: Mat3) -> Mat3 {
    Mat3::from_cols(m.x_axis.normalize_or_zero(), m.y_axis.normalize_or_zero(), m.z_axis.normalize_or_zero())
}

/// `cubic_tangent_factor_circle_v3`.
fn circle_factor(l: Vec3, r: Vec3) -> f32 {
    const EPS: f32 = 1e-5;
    let dot = l.dot(r);
    if dot > 1.0 - EPS {
        return (1.0 / 3.0) * 0.75;
    }
    if dot < -1.0 + EPS {
        return 0.5;
    }
    let angle = dot.acos() / 2.0;
    let (sin, cos) = angle.sin_cos();
    ((1.0 - cos) / (sin * 2.0)) / sin
}

/// `BKE_curve_forward_diff_bezier`, one axis.
fn forward_diff(q0: f32, q1: f32, q2: f32, q3: f32, it: usize) -> Vec<f32> {
    let f = it as f32;
    let (r0, r1) = (q0, 3.0 * (q1 - q0) / f);
    let r2 = 3.0 * (q0 - 2.0 * q1 + q2) / (f * f);
    let r3 = (q3 - q0 + 3.0 * (q1 - q2)) / (f * f * f);
    let (mut a0, mut a1, mut a2, a3) = (r0, r1 + r2 + r3, 2.0 * r2 + 6.0 * r3, 6.0 * r3);
    let mut out = Vec::with_capacity(it + 1);
    for _ in 0..=it {
        out.push(a0);
        a0 += a1;
        a1 += a2;
        a2 += a3;
    }
    out
}

/// `equalize_cubic_bezier`: parameter values that cut the curve into arcs
/// whose lengths follow `scales`.
fn equalize(control: [Vec3; 4], temp: usize, final_segments: usize, scales: &[f32]) -> Vec<f32> {
    let axes: [Vec<f32>; 3] =
        [0, 1, 2].map(|k| forward_diff(control[0][k], control[1][k], control[2][k], control[3][k], temp));
    let coords: Vec<Vec3> = (0..=temp).map(|i| Vec3::new(axes[0][i], axes[1][i], axes[2][i])).collect();
    let mut pdist = vec![0.0f32; temp + 1];
    for i in 0..temp {
        pdist[i + 1] = pdist[i] + (coords[i + 1] - coords[i]).length();
    }
    let sum: f32 = scales[..final_segments].iter().sum();
    let step = pdist[temp] / sum;
    let mut out = vec![0.0f32; final_segments + 1];
    let (mut dist, mut nr) = (0.0f32, 1usize);
    for i in 1..=final_segments {
        dist += scales[i - 1] * step;
        while nr < temp && dist >= pdist[nr] {
            nr += 1;
        }
        let fac = (pdist[nr] - dist) / (pdist[nr] - pdist[nr - 1]);
        out[i] = (nr as f32 - fac) / temp as f32;
    }
    out[final_segments] = 1.0;
    out
}

/// `evaluate_cubic_bezier`: position and tangent at `t`.
fn bezier_at(c: [Vec3; 4], t: f32) -> (Vec3, Vec3) {
    let l1 = [c[0].lerp(c[1], t), c[1].lerp(c[2], t), c[2].lerp(c[3], t)];
    let l2 = [l1[0].lerp(l1[1], t), l1[1].lerp(l1[2], t)];
    let tangent = l2[1] - l2[0];
    (l2[0] + tangent * t, tangent)
}

/// `ease_handle_axis`: fade to the second derivative when the handle is short.
fn ease_axis(d1: Vec3, d2: Vec3) -> Vec3 {
    const GAP: f32 = 0.1;
    let len2 = d2.length_squared();
    if len2 == 0.0 {
        return d1;
    }
    let ratio = d1.length_squared() / len2;
    if ratio < GAP * GAP { d1 + d2 * (GAP - ratio.sqrt()) } else { d1 }
}

/// `BBoneSplineParameters`.
#[derive(Clone, Debug, Default)]
struct Spline {
    segments: usize,
    length: f32,
    do_scale: bool,
    scale: Vec3,
    use_prev: bool,
    prev_h: Vec3,
    prev_bbone: bool,
    prev_mat: Mat4,
    use_next: bool,
    next_h: Vec3,
    next_bbone: bool,
    next_mat: Mat4,
    ease1: f32,
    ease2: f32,
    roll1: f32,
    roll2: f32,
    scale_in: Vec3,
    scale_out: Vec3,
    curve_in_x: f32,
    curve_in_z: f32,
    curve_out_x: f32,
    curve_out_z: f32,
}

impl Spline {
    /// `BKE_pchan_bbone_handles_compute`.
    fn handles(&self) -> (Vec3, f32, Vec3, f32) {
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
    fn compute(&mut self) -> Vec<Mat4> {
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

/// A bone's saved B-Bone properties and rest placement.
#[derive(Clone, Debug, Default)]
pub(crate) struct BoneShape {
    /// The bone's name. Nothing looks a shape up by it — the map is keyed by
    /// address — but this crate's job is matching Blender segment for
    /// segment, and the name is what says *which* bone a printed record is.
    /// A derived `Debug` does not count as a read, hence the allow.
    #[allow(dead_code)]
    pub name: String,
    pub segments: usize,
    pub length: f32,
    pub arm_mat: Mat4,
    pub arm_head: Vec3,
    pub arm_tail: Vec3,
    pub connected: bool,
    pub prev_type: u8,
    pub next_type: u8,
    pub bbone_flag: u32,
    pub prev_flag: u32,
    pub next_flag: u32,
    pub shape: Shape,
}

/// A pose channel's evaluated placement and its animated B-Bone offsets.
#[derive(Clone, Debug, Default)]
pub(crate) struct ChanShape {
    pub bone: u64,
    pub parent: u64,
    pub child: u64,
    pub bbone_prev: u64,
    pub bbone_next: u64,
    pub pose_mat: Mat4,
    pub pose_head: Vec3,
    pub pose_tail: Vec3,
    pub shape: Shape,
}

/// The B-Bone curve properties, which both `Bone` and `bPoseChannel` carry
/// under the same names — the bone's are the rest shape, the channel's are
/// what an animator moves.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Shape {
    pub roll1: f32,
    pub roll2: f32,
    pub curve_in_x: f32,
    pub curve_in_z: f32,
    pub curve_out_x: f32,
    pub curve_out_z: f32,
    pub ease1: f32,
    pub ease2: f32,
    pub scale_in: Vec3,
    pub scale_out: Vec3,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            roll1: 0.0,
            roll2: 0.0,
            curve_in_x: 0.0,
            curve_in_z: 0.0,
            curve_out_x: 0.0,
            curve_out_z: 0.0,
            ease1: 0.0,
            ease2: 0.0,
            scale_in: Vec3::ONE,
            scale_out: Vec3::ONE,
        }
    }
}

/// Everything read from one armature, keyed by pose-channel address.
#[derive(Clone, Debug, Default)]
pub(crate) struct Pose {
    pub chans: HashMap<u64, ChanShape>,
    pub bones: HashMap<u64, BoneShape>,
}

impl Pose {
    /// The same armature in another pose: each channel `pose_of` names
    /// takes its pose matrix, head and tail (a solved pose,
    /// `rig/solve.rs`); the rest keep the saved ones.
    pub(crate) fn posed(&self, pose_of: &dyn Fn(u64) -> Option<(Mat4, Vec3, Vec3)>) -> Self {
        let mut out = self.clone();
        for (addr, chan) in out.chans.iter_mut() {
            if let Some((mat, head, tail)) = pose_of(*addr) {
                (chan.pose_mat, chan.pose_head, chan.pose_tail) = (mat, head, tail);
            }
        }
        out
    }

    fn bone(&self, chan: &ChanShape) -> Option<&BoneShape> {
        self.bones.get(&chan.bone)
    }

    /// `BKE_pchan_bbone_handles_get`.
    fn handles(&self, addr: u64) -> (Option<&ChanShape>, Option<&ChanShape>) {
        let Some(chan) = self.chans.get(&addr) else { return (None, None) };
        let Some(bone) = self.bone(chan) else { return (None, None) };
        let prev = if bone.prev_type == BBONE_HANDLE_AUTO {
            bone.connected.then_some(chan.parent).unwrap_or(0)
        } else {
            chan.bbone_prev
        };
        let next = if bone.next_type == BBONE_HANDLE_AUTO { chan.child } else { chan.bbone_next };
        (self.chans.get(&prev), self.chans.get(&next))
    }

    /// `BKE_pchan_bbone_spline_params_get`.
    fn params(&self, addr: u64, rest: bool) -> Option<Spline> {
        let chan = self.chans.get(&addr)?;
        let bone = self.bone(chan)?;
        let mut p = Spline { segments: bone.segments, length: bone.length, ..Default::default() };
        if !rest {
            let s = Vec3::new(chan.pose_mat.x_axis.length(), chan.pose_mat.y_axis.length(), chan.pose_mat.z_axis.length());
            if (s.x - s.y).abs() > 1e-6 || (s.y - s.z).abs() > 1e-6 {
                (p.do_scale, p.scale) = (true, s);
            }
        }
        let (prev, next) = self.handles(addr);
        let imat = if rest {
            bone.arm_mat.inverse()
        } else if p.do_scale {
            normalized(chan.pose_mat).inverse()
        } else {
            chan.pose_mat.inverse()
        };
        let (mut prev_scale, mut next_scale) = (Vec3::ONE, Vec3::ONE);
        if let Some(pv) = prev {
            let pb = self.bone(pv);
            p.use_prev = true;
            let mut done = false;
            let mut h1 = Vec3::ZERO;
            if bone.prev_type == BBONE_HANDLE_RELATIVE {
                if rest {
                    done = true;
                } else if let Some(pb) = pb {
                    h1 = chan.pose_head - (pv.pose_head - pb.arm_head);
                }
            } else if bone.prev_type == BBONE_HANDLE_TANGENT {
                h1 = match (rest, pb) {
                    (true, Some(pb)) => bone.arm_head - (pb.arm_tail - pb.arm_head),
                    (false, _) => chan.pose_head - (pv.pose_tail - pv.pose_head),
                    _ => Vec3::ZERO,
                };
            } else {
                p.prev_bbone = pb.is_some_and(|b| b.segments > 1);
                h1 = match (rest, pb) {
                    (true, Some(pb)) => pb.arm_head,
                    _ => pv.pose_head,
                };
            }
            if !done {
                p.prev_h = imat.transform_point3(h1);
            }
            if !p.prev_bbone {
                p.prev_mat = imat * if rest { pb.map_or(Mat4::IDENTITY, |b| b.arm_mat) } else { pv.pose_mat };
                if bone.prev_flag & (SCALE_X | SCALE_Y | SCALE_Z | SCALE_EASE) != 0 && !rest {
                    prev_scale = local_scale(pv, pb);
                }
            }
        }
        if let Some(nx) = next {
            let nb = self.bone(nx);
            p.use_next = true;
            let mut done = false;
            let mut h2 = Vec3::ZERO;
            if bone.next_type == BBONE_HANDLE_RELATIVE {
                if rest {
                    p.next_h = Vec3::new(0.0, p.length, 0.0);
                    done = true;
                } else if let Some(nb) = nb {
                    h2 = chan.pose_tail + (nx.pose_head - nb.arm_head);
                }
            } else if bone.next_type == BBONE_HANDLE_TANGENT {
                h2 = match (rest, nb) {
                    (true, Some(nb)) => bone.arm_tail + (nb.arm_tail - nb.arm_head),
                    (false, _) => chan.pose_tail + (nx.pose_tail - nx.pose_head),
                    _ => Vec3::ZERO,
                };
            } else {
                p.next_bbone = nb.is_some_and(|b| b.segments > 1);
                h2 = match (rest, nb) {
                    (true, Some(nb)) => nb.arm_tail,
                    _ => nx.pose_tail,
                };
            }
            if !done {
                p.next_h = imat.transform_point3(h2);
            }
            p.next_mat = imat * if rest { nb.map_or(Mat4::IDENTITY, |b| b.arm_mat) } else { nx.pose_mat };
            if bone.next_flag & (SCALE_X | SCALE_Y | SCALE_Z | SCALE_EASE) != 0 && !rest {
                next_scale = local_scale(nx, nb);
            }
        }
        let (b, c) = (bone.shape, chan.shape);
        let zero = Shape { scale_in: Vec3::ONE, scale_out: Vec3::ONE, ..Default::default() };
        let c = if rest { zero } else { c };
        p.ease1 = b.ease1 + c.ease1;
        p.ease2 = b.ease2 + c.ease2;
        p.roll1 = b.roll1 + c.roll1;
        p.roll2 = b.roll2 + c.roll2;
        if bone.bbone_flag & BBONE_ADD_PARENT_END_ROLL != 0 {
            if let Some(pv) = prev {
                p.roll1 += self.bone(pv).map_or(0.0, |b| b.shape.roll2);
                if !rest {
                    p.roll1 += pv.shape.roll2;
                }
            }
        }
        p.scale_in = b.scale_in * if rest { Vec3::ONE } else { c.scale_in };
        p.scale_out = b.scale_out * if rest { Vec3::ONE } else { c.scale_out };
        p.curve_in_x = b.curve_in_x + c.curve_in_x;
        p.curve_in_z = b.curve_in_z + c.curve_in_z;
        p.curve_out_x = b.curve_out_x + c.curve_out_x;
        p.curve_out_z = b.curve_out_z + c.curve_out_z;
        if bone.bbone_flag & BBONE_SCALE_EASING != 0 {
            p.ease1 *= p.scale_in.y;
            p.curve_in_x *= p.scale_in.y;
            p.curve_in_z *= p.scale_in.y;
            p.ease2 *= p.scale_out.y;
            p.curve_out_x *= p.scale_out.y;
            p.curve_out_z *= p.scale_out.y;
        }
        apply_handle_scale(&mut p, bone.prev_flag, prev_scale, true);
        apply_handle_scale(&mut p, bone.next_flag, next_scale, false);
        Some(p)
    }

    /// `BKE_pchan_bbone_segments_cache_compute`: the deform matrices, in
    /// armature space. `chan` is `pose_mat · arm_mat⁻¹`, the same rigid deform
    /// matrix the one-segment path uses — with a flat spline the two agree.
    pub(crate) fn bbone(&self, addr: u64) -> Option<BBone> {
        let chan = self.chans.get(&addr)?;
        let bone = self.bone(chan)?;
        if bone.segments <= 1 {
            return None;
        }
        let posed = self.params(addr, false)?.compute();
        let rest = self.params(addr, true)?.compute();
        let to_bone = bone.arm_mat.inverse();
        let mats = (0..=bone.segments)
            .map(|a| chan.pose_mat * posed[a] * rest[a].inverse() * to_bone)
            .collect();
        Some(BBone { to_bone, mats, length: bone.length })
    }
}

fn normalized(m: Mat4) -> Mat4 {
    Mat4::from_cols(
        m.x_axis.normalize_or_zero(),
        m.y_axis.normalize_or_zero(),
        m.z_axis.normalize_or_zero(),
        m.w_axis,
    )
}

/// `BKE_armature_mat_pose_to_bone(pose_mat)` reduced to its scale.
fn local_scale(chan: &ChanShape, bone: Option<&BoneShape>) -> Vec3 {
    let Some(bone) = bone else { return Vec3::ONE };
    let m = bone.arm_mat.inverse() * chan.pose_mat;
    Vec3::new(m.x_axis.truncate().length(), m.y_axis.truncate().length(), m.z_axis.truncate().length())
}

fn apply_handle_scale(p: &mut Spline, flag: u32, scale: Vec3, start: bool) {
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

impl BlendFile {
    fn f3(&self, b: &BlendBlock, st: &str, field: &str, dna: &DnaType) -> Vec3 {
        let Some(o) = self.off(st, field, dna) else { return Vec3::ZERO };
        let f = |i: usize| b.data.get(o + i * 4..o + i * 4 + 4).map_or(0.0, |s| self.read_f32(s));
        Vec3::new(f(0), f(1), f(2))
    }

    fn m4(&self, b: &BlendBlock, st: &str, field: &str, dna: &DnaType) -> Mat4 {
        let Some(o) = self.off(st, field, dna) else { return Mat4::IDENTITY };
        let Some(s) = b.data.get(o..o + 64) else { return Mat4::IDENTITY };
        Mat4::from_cols_array(&std::array::from_fn(|i| self.read_f32(&s[i * 4..i * 4 + 4])))
    }

    fn i8_field(&self, b: &BlendBlock, st: &str, field: &str, dna: &DnaType) -> u8 {
        self.off(st, field, dna).and_then(|o| b.data.get(o).copied()).unwrap_or(0)
    }

    fn shape_of(&self, b: &BlendBlock, st: &str, dna: &DnaType) -> Shape {
        let f = |name: &str| self.float_field(b, st, name, dna, 0.0);
        Shape {
            roll1: f("roll1"),
            roll2: f("roll2"),
            curve_in_x: f("curve_in_x"),
            curve_in_z: f("curve_in_z"),
            curve_out_x: f("curve_out_x"),
            curve_out_z: f("curve_out_z"),
            ease1: f("ease1"),
            ease2: f("ease2"),
            scale_in: self.f3(b, st, "scale_in[3]", dna),
            scale_out: self.f3(b, st, "scale_out[3]", dna),
        }
    }

    /// Every pose channel and bone of `arm`, with the B-Bone properties.
    pub(crate) fn pose_shapes(&self, arm: &BlendBlock, dna: &DnaType) -> Pose {
        let mut pose = Pose { chans: HashMap::new(), bones: HashMap::new() };
        let ptr = |b: &BlendBlock, st: &str, f: &str| self.ptr(b, self.off(st, f, dna));
        self.scoped(arm, || {
            let Some(p) = self.block_at(self.ptr(arm, self.off("Object", "*pose", dna))) else { return };
            for pc in self.list(self.ptr(p, self.off("bPose", "chanbase", dna)), self.off("bPoseChannel", "*next", dna)) {
                let bone_addr = ptr(pc, "bPoseChannel", "*bone");
                pose.chans.insert(
                    pc.old_address,
                    ChanShape {
                        bone: bone_addr,
                        parent: ptr(pc, "bPoseChannel", "*parent"),
                        child: ptr(pc, "bPoseChannel", "*child"),
                        bbone_prev: ptr(pc, "bPoseChannel", "*bbone_prev"),
                        bbone_next: ptr(pc, "bPoseChannel", "*bbone_next"),
                        pose_mat: self.m4(pc, "bPoseChannel", "pose_mat[4][4]", dna),
                        pose_head: self.f3(pc, "bPoseChannel", "pose_head[3]", dna),
                        pose_tail: self.f3(pc, "bPoseChannel", "pose_tail[3]", dna),
                        shape: self.shape_of(pc, "bPoseChannel", dna),
                    },
                );
                let Some(bone) = self.block_at(bone_addr) else { continue };
                pose.bones.insert(
                    bone_addr,
                    BoneShape {
                        name: super::armature::name64(&bone.data, self.off("Bone", "name[64]", dna).unwrap_or(0)),
                        segments: self.int_field(bone, "Bone", "segments", dna).max(0) as usize,
                        length: self.float_field(bone, "Bone", "length", dna, 0.0),
                        arm_mat: self.m4(bone, "Bone", "arm_mat[4][4]", dna),
                        arm_head: self.f3(bone, "Bone", "arm_head[3]", dna),
                        arm_tail: self.f3(bone, "Bone", "arm_tail[3]", dna),
                        connected: self.int_field(bone, "Bone", "flag", dna) & BONE_CONNECTED != 0,
                        prev_type: self.i8_field(bone, "Bone", "bbone_prev_type", dna),
                        next_type: self.i8_field(bone, "Bone", "bbone_next_type", dna),
                        bbone_flag: self.int_field(bone, "Bone", "bbone_flag", dna),
                        prev_flag: self.int_field(bone, "Bone", "bbone_prev_flag", dna),
                        next_flag: self.int_field(bone, "Bone", "bbone_next_flag", dna),
                        shape: self.shape_of(bone, "Bone", dna),
                    },
                );
            }
        });
        pose
    }
}

impl BBone {
    /// The segment matrices, in the order the skinner expects them.
    pub(crate) fn slots(&self) -> impl Iterator<Item = Mat4> + '_ {
        self.mats.iter().copied()
    }
}
