//! Reading a rig: each pose channel of an armature object (`Object.pose →
//! bPose.chanbase`) with its `Bone`, constraints and custom properties, and
//! the object's drivers.

use super::constraint::read::BTargetNames;
use super::{BRig, BRigBone};
use crate::blend::file::types::*;
use crate::blend::modifier::armature::name64;
use glam::{Mat3, Mat4, Vec3};
use std::collections::HashMap;

/// One block read as the DNA struct `st`: fields by name, with a default
/// where the file's version has no such field.
pub struct BFields<'a> {
    file: &'a BlendFile,
    dna: &'a DnaType,
    block: &'a BlendBlock,
    st: &'static str,
}

impl<'a> BFields<'a> {
    pub fn new(file: &'a BlendFile, dna: &'a DnaType, block: &'a BlendBlock, st: &'static str) -> Self {
        Self { file, dna, block, st }
    }

    fn off(&self, field: &str) -> Option<usize> {
        self.file.off(self.st, field, self.dna)
    }

    pub fn has(&self, field: &str) -> bool {
        self.off(field).is_some()
    }

    /// A `char` / `short` / `int` field.
    pub fn int(&self, field: &str) -> u32 {
        self.file.int_field(self.block, self.st, field, self.dna)
    }

    pub fn f32(&self, field: &str, default: f32) -> f32 {
        self.file.f32s(&self.block.data, self.off(field), [default])[0]
    }

    pub fn vec3(&self, field: &str, default: Vec3) -> Vec3 {
        Vec3::from_array(self.file.f32s(&self.block.data, self.off(field), default.to_array()))
    }

    pub fn ptr(&self, field: &str) -> u64 {
        self.file.ptr(self.block, self.off(field))
    }

    /// A `char name[64]` field.
    pub fn name(&self, field: &str) -> String {
        self.off(field).map_or(String::new(), |o| name64(&self.block.data, o))
    }

    /// A `char text[len]` field, up to its terminator.
    pub fn text(&self, field: &str, len: usize) -> String {
        let Some(o) = self.off(field) else { return String::new() };
        let s = self.block.data.get(o..(o + len).min(self.block.data.len())).unwrap_or(&[]);
        let n = s.iter().position(|&c| c == 0).unwrap_or(s.len());
        String::from_utf8_lossy(&s[..n]).to_string()
    }

    /// Three float fields as one vector (`xmin`, `ymin`, `zmin`).
    pub fn vec3_of(&self, fields: [&str; 3]) -> Vec3 {
        Vec3::from_array(fields.map(|f| self.f32(f, 0.0)))
    }

    /// The first `n` bytes of a `char field[n]`.
    pub fn bytes(&self, field: &str, n: usize) -> Vec<u8> {
        let o = self.off(field);
        (0..n).map(|i| o.and_then(|o| self.block.data.get(o + i)).copied().unwrap_or(0)).collect()
    }

    pub fn mat4(&self, field: &str) -> Mat4 {
        Mat4::from_cols_array(&self.file.f32s(&self.block.data, self.off(field), Mat4::IDENTITY.to_cols_array()))
    }

    pub fn mat3(&self, field: &str) -> Mat3 {
        Mat3::from_cols_array(&self.file.f32s(&self.block.data, self.off(field), Mat3::IDENTITY.to_cols_array()))
    }
}

/// `BONE_NO_SCALE` (before `inherit_scale_mode`): read as
/// `BONE_INHERIT_SCALE_NONE_LEGACY`, as Blender's versioning does.
const BONE_NO_SCALE: u32 = 1 << 15;

impl BlendFile {
    /// The rig of armature object `ob`: its channels in order, each with its
    /// bone, constraints and custom properties, and the object's drivers.
    pub fn rig(&self, ob: &BlendBlock) -> Option<BRig> {
        let dna = self.dna.as_ref()?;
        self.scoped(ob, || {
            let pose = self.block_at(self.ptr(ob, self.off("Object", "*pose", dna)))?;
            let first = self.ptr(pose, self.off("bPose", "chanbase", dna));
            let chans = self.list(first, self.off("bPoseChannel", "*next", dna));
            let by_name: HashMap<String, usize> = chans.iter().enumerate().map(|(i, pc)| (BFields::new(self, dna, pc, "bPoseChannel").name("name[64]"), i)).collect();
            let by_address: HashMap<u64, usize> = chans.iter().enumerate().map(|(i, pc)| (pc.old_address, i)).collect();
            let names = BTargetNames { object: ob.old_address, bones: &by_name };
            let bones = chans.iter().map(|pc| self.rig_bone(pc, dna, &by_address, &names)).collect();
            let drivers = self.rig_drivers(ob, dna);
            let mut rig = BRig { object: ob.old_address, bones, by_name, drivers };
            rig.resolve_drivers();
            Some(rig)
        })
    }

    fn rig_bone(&self, pc: &BlendBlock, dna: &DnaType, by_address: &HashMap<u64, usize>, names: &BTargetNames) -> BRigBone {
        let c = BFields::new(self, dna, pc, "bPoseChannel");
        let bone = self.block_at(c.ptr("*bone"));
        let b = bone.map(|b| BFields::new(self, dna, b, "Bone"));
        let flag = b.as_ref().map_or(0, |b| b.int("flag"));
        let inherit_scale = match &b {
            Some(b) if b.has("inherit_scale_mode") => b.int("inherit_scale_mode") as u8,
            _ if flag & BONE_NO_SCALE != 0 => 4,
            _ => 0,
        };
        let props = self.id_group_at(c.ptr("*prop"), dna, 0).unwrap_or_default();
        BRigBone {
            name: c.name("name[64]"),
            address: pc.old_address,
            parent: by_address.get(&c.ptr("*parent")).copied(),
            arm: b.as_ref().map_or(Mat4::IDENTITY, |b| b.mat4("arm_mat[4][4]")),
            bone_mat: b.as_ref().map_or(Mat3::IDENTITY, |b| b.mat3("bone_mat[3][3]")),
            head: b.as_ref().map_or(Vec3::ZERO, |b| b.vec3("head[3]", Vec3::ZERO)),
            length: b.as_ref().map_or(0.0, |b| b.f32("length", 0.0)),
            flag,
            inherit_scale,
            segments: b.as_ref().map_or(1, |b| b.int("segments") as i16),
            raw: self.pchan_raw(pc, dna),
            ikflag: c.int("ikflag"),
            ikstretch: c.f32("ikstretch", 0.0),
            stiffness: c.vec3("stiffness[3]", Vec3::ZERO),
            limit_min: c.vec3("limitmin[3]", Vec3::ZERO),
            limit_max: c.vec3("limitmax[3]", Vec3::ZERO),
            constraints: self.rig_constraints(pc, dna, names),
            props,
        }
    }
}
