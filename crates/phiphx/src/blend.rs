//! The old home of the `.blend` reader, kept so `phiphx::blend::*` still
//! resolves. The code is the Blend device: `crate::device::blend`.
pub use crate::device::blend::*;
pub use crate::device::blend::read::{curve, mesh, text};
pub use crate::device::blend::{physics, rig, shade, write};