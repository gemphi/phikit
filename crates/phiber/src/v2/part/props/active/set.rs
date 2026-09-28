//! What the Object and Material tabs show of the active object, from the
//! runtime scene: `rna_object.cc`'s `location`, `rotation_euler`, `scale`
//! and `dimensions`, and `rna_material.cc`'s `diffuse_color`, `metallic`
//! and `roughness` — the Principled values the colour graph folds to.

use crate::phiphx::scene::{SceneMaterial, SceneObject};
use crate::v2::state::{BKey as _, BState, ButsKey, val::BValue};
use super::super::set::prop_named;

const AXIS: [&str; 3] = ["x", "y", "z"];

/// One vector property, an axis at a time — the panel draws a `Vec3` as
/// three number fields under `ctx.buts.<key>.<axis>`.
fn write_vec3(state: &mut BState, key: &str, value: [f32; 3]) {
    let Some(spec) = prop_named(key) else { return };
    for (axis, component) in AXIS.iter().zip(value) {
        state.set(ButsKey::Axis(spec.key, axis).path(), BValue::Float(component).encode());
    }
}

fn write(state: &mut BState, key: &str, value: BValue) {
    if let Some(spec) = prop_named(key) {
        state.set(ButsKey::Value(spec.key), value.encode());
    }
}

/// `mat3_to_eul` for Blender's default XYZ order, in degrees as the panel
/// prints an angle. Scale is divided out first, as `BKE_object_rot_to_mat3`
/// never put it in.
pub fn euler_xyz_degrees(matrix: [[f32; 4]; 4], scale: [f32; 3]) -> [f32; 3] {
    let column = |c: usize| {
        let s = if scale[c].abs() > 1e-8 { scale[c] } else { 1.0 };
        [matrix[c][0] / s, matrix[c][1] / s, matrix[c][2] / s]
    };
    let (x, y, z) = (column(0), column(1), column(2));
    // Blender's `float[4][4]` is `m[col][row]`, so `x` is the rotated X axis.
    let cy = (x[0] * x[0] + x[1] * x[1]).sqrt();
    let (ex, ey, ez) = if cy > 16.0 * f32::EPSILON {
        ((y[2]).atan2(z[2]), (-x[2]).atan2(cy), (x[1]).atan2(x[0]))
    } else {
        ((-z[1]).atan2(y[1]), (-x[2]).atan2(cy), 0.0)
    };
    [ex.to_degrees(), ey.to_degrees(), ez.to_degrees()]
}

/// `BKE_object_dimensions_get`: the object-space bounding box scaled by the
/// object's scale.
pub fn dimensions(object: &SceneObject) -> [f32; 3] {
    let mut low = [f32::INFINITY; 3];
    let mut high = [f32::NEG_INFINITY; 3];
    for vertex in object.mesh.vertices.iter() {
        for axis in 0..3 {
            low[axis] = low[axis].min(vertex[axis]);
            high[axis] = high[axis].max(vertex[axis]);
        }
    }
    if object.mesh.vertices.is_empty() {
        return [0.0; 3];
    }
    [0, 1, 2].map(|axis| (high[axis] - low[axis]) * object.scale[axis].abs())
}

pub fn seed_transform(state: &mut BState, object: &SceneObject) {
    write_vec3(state, "location", object.location);
    write_vec3(state, "rotation_euler", euler_xyz_degrees(object.matrix_world, object.scale));
    write_vec3(state, "scale", object.scale);
    write_vec3(state, "dimensions", dimensions(object));
}

/// The panel's own values for an object it already changed.
pub fn seed_edited(state: &mut BState, t: &crate::v2::scene::transform::BTransform) {
    write_vec3(state, "location", t.location);
    write_vec3(state, "rotation_euler", t.rotation);
    write_vec3(state, "scale", t.scale);
}

pub fn seed_material(state: &mut BState, material: &SceneMaterial) {
    write(state, "diffuse_color", BValue::Color(material.base_color));
    write(state, "metallic", BValue::Float(material.metallic));
    write(state, "roughness", BValue::Float(material.roughness));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rotation_about_z_reads_back_in_degrees_with_the_scale_divided_out() {
        // 90° about Z with a scale of 2: the X axis lands on +Y, doubled.
        let matrix = [[0.0, 2.0, 0.0, 0.0], [-2.0, 0.0, 0.0, 0.0], [0.0, 0.0, 2.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
        let e = euler_xyz_degrees(matrix, [2.0, 2.0, 2.0]);
        assert!(e[0].abs() < 1e-4 && e[1].abs() < 1e-4 && (e[2] - 90.0).abs() < 1e-3, "{e:?}");
        let identity = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
        assert_eq!(euler_xyz_degrees(identity, [1.0; 3]), [0.0, 0.0, 0.0]);
    }
}
