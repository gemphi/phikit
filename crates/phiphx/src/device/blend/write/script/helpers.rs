//! Blender type-name parsing (`From<&str>`).

use super::types::{BlenderLightType, BlenderObjectType};

/// Blender RNA type name → object type (unknown → Empty).
impl From<&str> for BlenderObjectType {
    fn from(type_str: &str) -> Self {
    match type_str {
        "MESH" => BlenderObjectType::Mesh,
        "LIGHT" => BlenderObjectType::Light,
        "CAMERA" => BlenderObjectType::Camera,
        "EMPTY" => BlenderObjectType::Empty,
        "CURVE" => BlenderObjectType::Curve,
        "SURFACE" => BlenderObjectType::Surface,
        "FONT" => BlenderObjectType::Text,
        "ARMATURE" => BlenderObjectType::Armature,
        "LATTICE" => BlenderObjectType::Lattice,
        _ => BlenderObjectType::Empty,
    }
    }
}

/// Blender RNA light type name → light type (unknown → Point).
impl From<&str> for BlenderLightType {
    fn from(type_str: &str) -> Self {
    match type_str {
        "POINT" => BlenderLightType::Point,
        "SUN" => BlenderLightType::Sun,
        "SPOT" => BlenderLightType::Spot,
        "AREA" => BlenderLightType::Area,
        _ => BlenderLightType::Point,
    }
    }
}
