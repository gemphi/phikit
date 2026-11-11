//! What is read from one armature: bone and channel shapes, keyed by address.
use crate::device::blend::modifier::bbone::*;

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
