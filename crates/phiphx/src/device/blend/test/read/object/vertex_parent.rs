use super::*;

fn close(a: Mat4, b: Mat4) -> bool {
    a.to_cols_array().iter().zip(b.to_cols_array()).all(|(x, y)| (x - y).abs() < 1e-5)
}

#[test]
fn one_vertex_takes_only_the_place() {
    let world = Mat4::from_rotation_z(0.7) * Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let verts = [[0.0, 0.0, 0.0], [1.0, 0.5, -0.25]];
    let m = parent_matrix(PARVERT1, [1, 0, 0], world, &verts);
    assert!(close(m, Mat4::from_translation(world.transform_point3(Vec3::new(1.0, 0.5, -0.25)))));
}

#[test]
fn a_flat_triangle_on_the_xy_plane_is_the_identity_frame() {
    // Normal +Z, v1 → v2 along +X: Blender's `tri_to_quat` is the unit
    // quaternion here, so the frame is a translation to the centroid.
    let verts = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
    let m = parent_matrix(PARVERT3, [0, 1, 2], Mat4::IDENTITY, &verts);
    assert!(close(m, Mat4::from_translation(Vec3::new(2.0 / 3.0, 1.0, 0.0))), "{m}");
}

#[test]
fn the_frame_carries_the_normal_to_z_and_the_edge_to_x() {
    let (v1, v2, v3) = (Vec3::new(0.3, -0.2, 1.0), Vec3::new(1.4, 0.1, 0.6), Vec3::new(0.5, 1.3, 1.9));
    let rot = Mat3::from_quat(tri_to_quat(v1, v2, v3));
    let n = normal_tri(v1, v2, v3);
    assert!((rot * Vec3::Z - n).length() < 1e-5, "Z lands on the normal");
    let edge = (v2 - v1).normalize();
    assert!((rot * Vec3::X).dot(edge) > 0.999, "X lands along v1 → v2");
}

#[test]
fn an_index_past_the_mesh_is_the_origin() {
    let verts = [[1.0, 1.0, 1.0]];
    let m = parent_matrix(PARVERT1, [7, 0, 0], Mat4::IDENTITY, &verts);
    assert!(close(m, Mat4::IDENTITY));
}
