//! Inverse kinematics with Blender's legacy solver
//! (`ikplugin/intern/iksolver_plugin.cc` over `intern/iksolver`): the IK
//! constraints gathered into trees at their chain roots
//! (`initialize_posetree`), each tree's bones as solver segments
//! (`execute_posetree`), solved, and the change laid back onto the pose
//! (`make_dmats`, `where_is_ik_bone`).

mod build;
mod execute;
mod jacobian;
mod joint;
mod rotation;
mod segment;
mod solver;
mod tree;

use super::constraint::{BConstraintKind, BIkData, CONSTRAINT_DISABLE};
use super::math::{BMat3, BMat4, BQuat, BVec3};
use super::solve::BPoseWork;
use super::space::SPACE_WORLD;
use super::{BRig, BRigState};
use glam::{DMat3, DVec3, Mat3, Mat4, Vec3};
use segment::{BJoint, BSegment};
use solver::BIkSolver;

/// `CONSTRAINT_IK_*`.
const IK_TIP: u32 = 1 << 0;
const IK_ROT: u32 = 1 << 1;
const IK_AUTO: u32 = 1 << 2;
const IK_STRETCH: u32 = 1 << 4;
const IK_POS: u32 = 1 << 5;
const IK_SETANGLE: u32 = 1 << 6;
/// `BONE_IK_NO_?DOF` (and `_TEMP`), `BONE_IK_?LIMIT`.
const NO_DOF: [u32; 3] = [1 << 0, 1 << 1, 1 << 2];
const NO_DOF_TEMP: [u32; 3] = [1 << 10, 1 << 11, 1 << 12];
const LIMIT: [u32; 3] = [1 << 3, 1 << 4, 1 << 5];
/// `IK_STRETCH_STIFF_EPS` / `_MIN` / `_MAX`.
const STRETCH_STIFF_EPS: f64 = 0.01;
const STRETCH_STIFF_MIN: f64 = 0.001;
const STRETCH_STIFF_MAX: f64 = 1e10;

/// A goal in a tree: the constraint and the chain bone it pulls.
#[derive(Clone, Debug)]
pub struct BIkTarget {
    pub bone: usize,
    pub con: usize,
    pub tip: usize,
}

/// `PoseTree`: bones root first, each one's parent within the tree.
#[derive(Clone, Debug)]
pub struct BIkTree {
    pub bones: Vec<usize>,
    pub parent: Vec<i32>,
    pub targets: Vec<BIkTarget>,
    pub iterations: i32,
    pub stretch: bool,
}
