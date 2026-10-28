//! A rigid body constraint (`RigidBodyCon`, Physics ▸ Rigid Body
//! Constraint): the joint an object (usually an empty) places between two
//! bodies.

use crate::blend::file::types::*;

const RBC_FLAG_ENABLED: u32 = 1 << 0;
const RBC_FLAG_DISABLE_COLLISIONS: u32 = 1 << 2;
const RBC_FLAG_USE_BREAKING: u32 = 1 << 3;
const RBC_FLAG_OVERRIDE_SOLVER_ITERATIONS: u32 = 1 << 4;
/// `RBC_FLAG_USE_LIMIT_LIN_X` and the five after it: lin x y z, ang x y z.
const RBC_FLAG_USE_LIMIT: u32 = 1 << 5;
/// `RBC_FLAG_USE_SPRING_X`, `_Y`, `_Z`.
const RBC_FLAG_USE_SPRING: u32 = 1 << 11;
const RBC_FLAG_USE_MOTOR_LIN: u32 = 1 << 14;
const RBC_FLAG_USE_MOTOR_ANG: u32 = 1 << 15;
/// `RBC_FLAG_USE_SPRING_ANG_X`, `_Y`, `_Z`.
const RBC_FLAG_USE_SPRING_ANG: u32 = 1 << 16;

/// `eRigidBodyCon_Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BJointKind {
    Point,
    Hinge,
    Slider,
    Piston,
    #[default]
    Fixed,
    Generic,
    GenericSpring,
    Motor,
}

impl BJointKind {
    /// `RBC_TYPE_POINT` 0, `_HINGE` 1, `_SLIDER` 3, `_6DOF` 5,
    /// `_6DOF_SPRING` 6, `_FIXED` 8, `_PISTON` 9, `_MOTOR` 11 (2, 4, 7 and
    /// 10 are unused).
    fn of(v: u32) -> Self {
        match v {
            0 => Self::Point,
            1 => Self::Hinge,
            3 => Self::Slider,
            9 => Self::Piston,
            5 => Self::Generic,
            6 => Self::GenericSpring,
            11 => Self::Motor,
            _ => Self::Fixed,
        }
    }

    pub fn blender(self) -> &'static str {
        match self {
            Self::Point => "POINT",
            Self::Hinge => "HINGE",
            Self::Slider => "SLIDER",
            Self::Piston => "PISTON",
            Self::Fixed => "FIXED",
            Self::Generic => "GENERIC",
            Self::GenericSpring => "GENERIC_SPRING",
            Self::Motor => "MOTOR",
        }
    }
}

/// `eRigidBodyCon_SpringType`: Blender 2.7's spring model or 2.8's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BSpringKind {
    Spring1,
    #[default]
    Spring2,
}

/// One axis of a Generic or Generic Spring joint, linear or angular.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BJointAxis {
    pub use_limit: bool,
    pub lower: f32,
    pub upper: f32,
    pub use_spring: bool,
    pub stiffness: f32,
    pub damping: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BRigidJoint {
    /// The object carrying the constraint.
    pub object: String,
    pub kind: BJointKind,
    /// The two bodies, by object name; empty when unset.
    pub first: String,
    pub second: String,
    pub enabled: bool,
    pub disable_collisions: bool,
    pub use_breaking: bool,
    pub breaking_threshold: f32,
    pub override_iterations: bool,
    pub solver_iterations: u32,
    pub spring_kind: BSpringKind,
    /// Linear x, y, z.
    pub linear: [BJointAxis; 3],
    /// Angular x, y, z.
    pub angular: [BJointAxis; 3],
    pub use_motor_linear: bool,
    pub motor_linear_velocity: f32,
    pub motor_linear_impulse: f32,
    pub use_motor_angular: bool,
    pub motor_angular_velocity: f32,
    pub motor_angular_impulse: f32,
}

impl BlendFile {
    pub(crate) fn rigid_joint(&self, con: &BlendBlock, object: &str, dna: &DnaType) -> BRigidJoint {
        let flag = self.int_field(con, "RigidBodyCon", "flag", dna);
        let f = |n: &str, dflt: f32| self.float_field(con, "RigidBodyCon", n, dna, dflt);
        let axis = |space: &str, k: usize, spring_bit: u32, limit_bit: u32| {
            let a = ["x", "y", "z"][k];
            // `limit_lin_x_lower`, `spring_stiffness_ang_x`: the angular
            // spring names carry `ang_`, the linear ones nothing.
            let spring = match space {
                "ang" => format!("ang_{a}"),
                _ => a.to_string(),
            };
            BJointAxis {
                use_limit: flag & limit_bit != 0,
                lower: f(&format!("limit_{space}_{a}_lower"), -1.0),
                upper: f(&format!("limit_{space}_{a}_upper"), 1.0),
                use_spring: flag & spring_bit != 0,
                stiffness: f(&format!("spring_stiffness_{spring}"), 10.0),
                damping: f(&format!("spring_damping_{spring}"), 0.5),
            }
        };
        BRigidJoint {
            object: object.to_string(),
            kind: BJointKind::of(self.int_field(con, "RigidBodyCon", "type", dna)),
            first: self.object_name_at(self.ptr(con, self.off("RigidBodyCon", "*ob1", dna))),
            second: self.object_name_at(self.ptr(con, self.off("RigidBodyCon", "*ob2", dna))),
            enabled: flag & RBC_FLAG_ENABLED != 0,
            disable_collisions: flag & RBC_FLAG_DISABLE_COLLISIONS != 0,
            use_breaking: flag & RBC_FLAG_USE_BREAKING != 0,
            breaking_threshold: f("breaking_threshold", 10.0),
            override_iterations: flag & RBC_FLAG_OVERRIDE_SOLVER_ITERATIONS != 0,
            solver_iterations: self.uint_field(con, "RigidBodyCon", "num_solver_iterations", dna).unwrap_or(10),
            spring_kind: match self.int_field(con, "RigidBodyCon", "spring_type", dna) {
                0 => BSpringKind::Spring1,
                _ => BSpringKind::Spring2,
            },
            linear: std::array::from_fn(|k| axis("lin", k, RBC_FLAG_USE_SPRING << k, RBC_FLAG_USE_LIMIT << k)),
            angular: std::array::from_fn(|k| axis("ang", k, RBC_FLAG_USE_SPRING_ANG << k, RBC_FLAG_USE_LIMIT << (3 + k))),
            use_motor_linear: flag & RBC_FLAG_USE_MOTOR_LIN != 0,
            motor_linear_velocity: f("motor_lin_target_velocity", 1.0),
            motor_linear_impulse: f("motor_lin_max_impulse", 1.0),
            use_motor_angular: flag & RBC_FLAG_USE_MOTOR_ANG != 0,
            motor_angular_velocity: f("motor_ang_target_velocity", 1.0),
            motor_angular_impulse: f("motor_ang_max_impulse", 1.0),
        }
    }
}
