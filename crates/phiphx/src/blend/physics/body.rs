//! One object's rigid body (`RigidBodyOb`, Physics ▸ Rigid Body).

use crate::blend::file::types::*;

const RBO_FLAG_KINEMATIC: u32 = 1 << 0;
const RBO_FLAG_DISABLED: u32 = 1 << 5;
const RBO_FLAG_USE_DEFORM: u32 = 1 << 7;
const RBO_FLAG_USE_MARGIN: u32 = 1 << 6;
const RBO_FLAG_START_DEACTIVATED: u32 = 1 << 4;
const RBO_FLAG_USE_DEACTIVATION: u32 = 1 << 3;

/// `eRigidBodyOb_Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BRigidKind {
    #[default]
    Active,
    Passive,
}

impl BRigidKind {
    fn of(v: u32) -> Self {
        match v {
            1 => Self::Passive,
            _ => Self::Active,
        }
    }

    pub fn blender(self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Passive => "PASSIVE",
        }
    }
}

/// `eRigidBody_Shape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BRigidShape {
    Box,
    Sphere,
    Capsule,
    Cylinder,
    Cone,
    #[default]
    ConvexHull,
    Mesh,
    Compound,
}

impl BRigidShape {
    fn of(v: u32) -> Self {
        match v {
            0 => Self::Box,
            1 => Self::Sphere,
            2 => Self::Capsule,
            3 => Self::Cylinder,
            4 => Self::Cone,
            6 => Self::Mesh,
            7 => Self::Compound,
            _ => Self::ConvexHull,
        }
    }

    pub fn blender(self) -> &'static str {
        match self {
            Self::Box => "BOX",
            Self::Sphere => "SPHERE",
            Self::Capsule => "CAPSULE",
            Self::Cylinder => "CYLINDER",
            Self::Cone => "CONE",
            Self::ConvexHull => "CONVEX_HULL",
            Self::Mesh => "MESH",
            Self::Compound => "COMPOUND",
        }
    }
}

/// `eRigidBody_MeshSource`: which mesh a Mesh or Convex Hull shape is
/// built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BMeshSource {
    Base,
    #[default]
    Deform,
    Final,
}

impl BMeshSource {
    fn of(v: u32) -> Self {
        match v {
            0 => Self::Base,
            2 => Self::Final,
            _ => Self::Deform,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BRigidBody {
    pub object: String,
    pub kind: BRigidKind,
    pub shape: BRigidShape,
    pub mesh_source: BMeshSource,
    /// Dynamic (not `RBO_FLAG_DISABLED`).
    pub enabled: bool,
    /// Animated.
    pub kinematic: bool,
    /// Deforming: rebuild the shape from the deformed mesh each frame.
    pub deforming: bool,
    pub mass: f32,
    pub friction: f32,
    /// Bounciness.
    pub restitution: f32,
    pub use_margin: bool,
    pub margin: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub use_deactivation: bool,
    pub start_deactivated: bool,
    pub linear_sleep: f32,
    pub angular_sleep: f32,
    /// Collision Collections: a bit per collection, 20 of them.
    pub collections: u32,
}

impl Default for BRigidBody {
    fn default() -> Self {
        Self {
            object: String::new(),
            kind: Default::default(),
            shape: Default::default(),
            mesh_source: Default::default(),
            enabled: true,
            kinematic: false,
            deforming: false,
            mass: 1.0,
            friction: 0.5,
            restitution: 0.0,
            use_margin: false,
            margin: 0.04,
            linear_damping: 0.04,
            angular_damping: 0.1,
            use_deactivation: false,
            start_deactivated: false,
            linear_sleep: 0.4,
            angular_sleep: 0.5,
            collections: 1,
        }
    }
}

impl BlendFile {
    pub(crate) fn rigid_body(&self, rb: &BlendBlock, object: &str, dna: &DnaType) -> BRigidBody {
        let d = BRigidBody::default();
        let flag = self.int_field(rb, "RigidBodyOb", "flag", dna);
        let f = |n: &str, dflt: f32| self.float_field(rb, "RigidBodyOb", n, dna, dflt);
        BRigidBody {
            object: object.to_string(),
            kind: BRigidKind::of(self.int_field(rb, "RigidBodyOb", "type", dna)),
            shape: BRigidShape::of(self.int_field(rb, "RigidBodyOb", "shape", dna)),
            mesh_source: BMeshSource::of(self.int_field(rb, "RigidBodyOb", "mesh_source", dna)),
            enabled: flag & RBO_FLAG_DISABLED == 0,
            kinematic: flag & RBO_FLAG_KINEMATIC != 0,
            deforming: flag & RBO_FLAG_USE_DEFORM != 0,
            mass: f("mass", d.mass),
            friction: f("friction", d.friction),
            restitution: f("restitution", d.restitution),
            use_margin: flag & RBO_FLAG_USE_MARGIN != 0,
            margin: f("margin", d.margin),
            linear_damping: f("lin_damping", d.linear_damping),
            angular_damping: f("ang_damping", d.angular_damping),
            use_deactivation: flag & RBO_FLAG_USE_DEACTIVATION != 0,
            start_deactivated: flag & RBO_FLAG_START_DEACTIVATED != 0,
            linear_sleep: f("lin_sleep_thresh", d.linear_sleep),
            angular_sleep: f("ang_sleep_thresh", d.angular_sleep),
            collections: self.uint_field(rb, "RigidBodyOb", "col_groups", dna).unwrap_or(d.collections),
        }
    }
}
