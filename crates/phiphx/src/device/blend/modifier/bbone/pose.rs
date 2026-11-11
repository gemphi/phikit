//! A pose's B-Bone spline parameters and segment matrices (`BKE_pchan_bbone_*`).
use super::*;

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

pub(super) fn normalized(m: Mat4) -> Mat4 {
    Mat4::from_cols(
        m.x_axis.normalize_or_zero(),
        m.y_axis.normalize_or_zero(),
        m.z_axis.normalize_or_zero(),
        m.w_axis,
    )
}

/// `BKE_armature_mat_pose_to_bone(pose_mat)` reduced to its scale.
pub(super) fn local_scale(chan: &ChanShape, bone: Option<&BoneShape>) -> Vec3 {
    let Some(bone) = bone else { return Vec3::ONE };
    let m = bone.arm_mat.inverse() * chan.pose_mat;
    Vec3::new(m.x_axis.truncate().length(), m.y_axis.truncate().length(), m.z_axis.truncate().length())
}
