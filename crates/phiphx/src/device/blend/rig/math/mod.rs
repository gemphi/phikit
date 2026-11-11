//! Blender's matrix and rotation math, as the pose solver needs it
//! (`blenlib/intern/math_matrix_c.cc`, `math_rotation_c.cc`), in single
//! precision where Blender works in single precision. Matrices are glam's,
//! whose columns are Blender's `m[0..4]`: `m[i][j]` is `col(i)[j]`.

mod euler;
mod mat4;
mod ortho;
mod quat;
mod symmetric;
mod vec;

pub use euler::BEulerOrder;
pub use mat4::{BCompose, BMat4};
pub use quat::BQuat;
pub use vec::{BMat3, BVec3};

#[cfg(test)]
#[path = "../../test/rig/math.rs"]
mod math_test;
