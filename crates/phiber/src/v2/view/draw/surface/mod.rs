//! GPUI-independent rounded-surface ownership and treatment helpers.
//!
//! These helpers deliberately avoid `overflow_hidden`: GPUI clips that mask
//! to a rectangle rather than to an element's border radius.

mod corner;
mod treatment;

pub use corner::{BCellPosition, BCornerOwnership, BCorners, BRowPosition, BSurfaceOpacity};
pub use treatment::{
    BChildCornerSupport, BChildSurfacePlan, child_surface_plan, compatible_inner_radius,
};
