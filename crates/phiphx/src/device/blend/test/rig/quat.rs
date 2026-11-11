use super::*;

#[test]
fn a_quaternion_turns_a_vector_as_its_matrix_does() {
    let q = BQuat([0.8, 0.2, -0.4, 0.4]).normalized();
    let v = Vec3::new(0.3, -1.2, 0.7);
    assert!((q.rotate(v) - q.to_mat3() * v).length() < 1e-5);
}

#[test]
fn tracking_points_the_axis_along_the_vector() {
    // Axis 5 (`-Z` in `TRACK_*` terms) points +Z along the vector here,
    // as the curve's bevel list uses it.
    let dir = Vec3::new(0.3, 0.8, -0.2).normalize();
    let q = BQuat::of_track(dir, 5, 1);
    assert!((q.to_mat3().z_axis - dir).length() < 1e-5);
}
