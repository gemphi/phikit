//! Tests for the `BKE_object_to_mat4` port.

use crate::blend::object::transform::*;
use glam::{Mat4, Vec3};

    fn approx(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> bool {
        a.iter()
            .flatten()
            .zip(b.iter().flatten())
            .all(|(x, y)| (x - y).abs() < 1e-5)
    }

    #[test]
    fn identity_object_is_identity() {
        assert!(approx(object_to_mat4(&ObjectTransformRaw::default()), IDENTITY4));
    }

    #[test]
    fn translation_lands_in_column_3() {
        let raw = ObjectTransformRaw { loc: [1.0, 2.0, 3.0], ..Default::default() };
        let m = object_to_mat4(&raw);
        assert_eq!(m[3], [1.0, 2.0, 3.0, 1.0]);
    }

    #[test]
    fn euler_xyz_90deg_about_z_maps_x_to_y() {
        let raw = ObjectTransformRaw {
            rot: [0.0, 0.0, std::f32::consts::FRAC_PI_2],
            ..Default::default()
        };
        let m = Mat4::from_cols_array_2d(&object_to_mat4(&raw));
        let v = m.transform_vector3(Vec3::X);
        assert!((v - Vec3::Y).length() < 1e-5, "{v:?}");
    }

    #[test]
    fn scale_and_delta_scale_multiply() {
        let raw = ObjectTransformRaw { size: [2.0; 3], dscale: [0.5, 1.0, 1.0], ..Default::default() };
        let m = object_to_mat4(&raw);
        assert!((m[0][0] - 1.0).abs() < 1e-6 && (m[1][1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn child_inherits_parent_translation() {
        let parent = ObjectTransformRaw { loc: [10.0, 0.0, 0.0], ..Default::default() };
        let child = ObjectTransformRaw { loc: [0.0, 1.0, 0.0], ..Default::default() };
        let pw = object_to_mat4(&parent);
        let w = compose_world(&pw, &IDENTITY4, &object_to_mat4(&child));
        assert_eq!(w[3], [10.0, 1.0, 0.0, 1.0]);
    }
