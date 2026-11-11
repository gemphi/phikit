use crate::device::blend::rig::math::*;
use glam::{Mat3, Vec3};

#[test]
fn polar_splits_rotation_from_scale() {
    let r = BEulerOrder(1).to_mat3(Vec3::new(0.3, -0.7, 1.1));
    let s = Mat3::from_cols_array_2d(&[[2.0, 0.1, 0.0], [0.1, 0.5, 0.2], [0.0, 0.2, 1.3]]);
    let (u, p) = (r * s).polar();
    assert!((u - r).abs_diff_eq(Mat3::ZERO, 1e-5), "{u} vs {r}");
    assert!((p - s).abs_diff_eq(Mat3::ZERO, 1e-5), "{p} vs {s}");
}

#[test]
fn eulers_round_trip_in_every_order() {
    let e = Vec3::new(0.4, -0.2, 0.9);
    for order in 1..=6 {
        let o = BEulerOrder(order);
        let back = o.of_normalized(&o.to_mat3(e));
        assert!((back - e).abs().max_element() < 1e-5, "order {order}: {back}");
    }
}

#[test]
fn a_quaternion_survives_its_matrix() {
    let q = BQuat([0.8, 0.2, -0.4, 0.4]);
    let n: f32 = q.0.iter().map(|v| v * v).sum::<f32>().sqrt();
    let q = BQuat(q.0.map(|v| v / n));
    let back = BQuat::of_normalized_fast(&q.to_mat3());
    assert!(back.0.iter().zip(q.0).all(|(a, b)| (a - b).abs() < 1e-5), "{back:?}");
}
