//! Composing xformOp attributes into a matrix.
use crate::device::usd::xform::XformResolver;
use crate::device::usd::{UsdPrim, UsdValue};
use glam::{Mat4, Quat, Vec3};


#[test]
fn no_xform_ops_returns_identity() {
    let prim = UsdPrim::new("/Root", "Xform");
    let mat = XformResolver::resolve(&prim);
    assert_eq!(mat, Mat4::IDENTITY);
}

#[test]
fn translate_only_produces_translation_matrix() {
    let mut prim = UsdPrim::new("/Root", "Xform");
    prim.set_attribute("xformOp:translate", UsdValue::Float3([1.0, 2.0, 3.0]));
    let mat = XformResolver::resolve(&prim);
    let expected = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(mat, expected);
}

#[test]
fn scale_only_produces_scale_matrix() {
    let mut prim = UsdPrim::new("/Root", "Xform");
    prim.set_attribute("xformOp:scale", UsdValue::Float3([2.0, 3.0, 4.0]));
    let mat = XformResolver::resolve(&prim);
    let expected = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
    assert_eq!(mat, expected);
}

#[test]
fn rotate_xyz_produces_rotation_matrix() {
    let mut prim = UsdPrim::new("/Root", "Xform");
    prim.set_attribute("xformOp:rotateXYZ", UsdValue::Float3([90.0, 0.0, 0.0]));
    let mat = XformResolver::resolve(&prim);
    let r = std::f32::consts::FRAC_PI_2;
    let expected = Mat4::from_quat(Quat::from_euler(glam::EulerRot::XYZ, r, 0.0, 0.0));
    assert!((mat.col(0) - expected.col(0)).length() < 1e-6);
    assert!((mat.col(1) - expected.col(1)).length() < 1e-6);
    assert!((mat.col(2) - expected.col(2)).length() < 1e-6);
    assert!((mat.col(3) - expected.col(3)).length() < 1e-6);
}

#[test]
fn default_order_is_scale_rotate_translate() {
    let mut prim = UsdPrim::new("/Root", "Xform");
    prim.set_attribute("xformOp:translate", UsdValue::Float3([10.0, 0.0, 0.0]));
    prim.set_attribute("xformOp:scale", UsdValue::Float3([2.0, 2.0, 2.0]));
    let mat = XformResolver::resolve(&prim);
    // Default: scale first, then translate
    let expected = Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0))
        * Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
    assert_eq!(mat, expected);
}

#[test]
fn explicit_xform_op_order_overrides_default() {
    let mut prim = UsdPrim::new("/Root", "Xform");
    prim.set_attribute("xformOp:translate", UsdValue::Float3([5.0, 0.0, 0.0]));
    prim.set_attribute("xformOp:scale", UsdValue::Float3([2.0, 2.0, 2.0]));
    // Explicit order: translate first, then scale (opposite of default)
    prim.set_attribute(
        "xformOpOrder",
        UsdValue::StringArray(vec![
            "xformOp:translate".to_string(),
            "xformOp:scale".to_string(),
        ]),
    );
    let mat = XformResolver::resolve(&prim);
    let expected = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0))
        * Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0));
    assert_eq!(mat, expected);
}
