//! Reading B-Bone properties from the file.
use crate::device::blend::modifier::bbone::*;

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
                        name: crate::device::blend::modifier::armature::name64(&bone.data, self.off("Bone", "name[64]", dna).unwrap_or(0)),
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
