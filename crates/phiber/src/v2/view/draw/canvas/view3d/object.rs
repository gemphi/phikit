//! Per-object helpers shared by the mesh painter and the click pick.

use crate::v2::scene::BObject3D;

pub(in super::super::super) fn name(object: &BObject3D) -> &str {
    match object {
        BObject3D::Primitive { name, .. } | BObject3D::Loaded { name, .. } => name,
    }
}
