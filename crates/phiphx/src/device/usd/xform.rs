//! XformResolver — composes `xformOp:*` attributes on a `UsdPrim` into a
//! `glam::Mat4` local transform. Mirrors Apple's USDKit xform composition.

use glam::{Mat4, Quat, Vec3};

use crate::device::usd::attribute::UsdValue;
use crate::device::usd::prim::UsdPrim;

/// Composes `xformOp:*` attributes on a prim into a single `Mat4`.
pub struct XformResolver;

impl XformResolver {
    /// Resolve the composed local transform for a prim by reading its
    /// `xformOp:*` attributes and composing them in declared order.
    ///
    /// Returns `Mat4::IDENTITY` when no xformOp attributes are authored.
    pub fn resolve(prim: &UsdPrim) -> Mat4 {
        let ops = Self::op_order(prim);
        if ops.is_empty() {
            return Mat4::IDENTITY;
        }
        let mut mat = Mat4::IDENTITY;
        for op_name in &ops {
            if let Some(value) = prim.attribute(op_name) {
                mat = Self::apply_op(mat, op_name, value);
            }
        }
        mat
    }

    /// Read the `xformOpOrder` attribute if present; otherwise default to
    /// scale → rotate → translate.
    fn op_order(prim: &UsdPrim) -> Vec<String> {
        // Check for explicit xformOpOrder (stored as a token array or string array)
        if let Some(value) = prim.attribute("xformOpOrder") {
            if let Some(names) = Self::extract_op_names(value) {
                return names;
            }
        }
        // Fall back: collect authored xformOp attributes in default order
        Self::default_op_order(prim)
    }

    /// Build the per-op `Mat4` and multiply into the accumulator.
    fn apply_op(mat: Mat4, op_name: &str, value: &UsdValue) -> Mat4 {
        let suffix = op_name.strip_prefix("xformOp:").unwrap_or(op_name);
        match suffix {
            "translate" => {
                let t = Self::extract_vec3(value);
                mat * Mat4::from_translation(t)
            }
            "scale" => {
                let s = Self::extract_vec3(value);
                mat * Mat4::from_scale(s)
            }
            "rotateXYZ" => {
                let r = Self::extract_vec3(value);
                let q = Self::euler_xyz_to_quat(r);
                mat * Mat4::from_quat(q)
            }
            "orient" => {
                let q = Self::extract_quat(value);
                mat * Mat4::from_quat(q)
            }
            "transform" => match value {
                UsdValue::Matrix(m) => {
                    let cols = m.map(|v| v as f32);
                    mat * Mat4::from_cols_array(&cols)
                }
                _ => mat,
            },
            _ => mat, // Unknown op: skipped
        }
    }

    /// Extract a `Vec3` from a `UsdValue::Float3`.
    fn extract_vec3(value: &UsdValue) -> Vec3 {
        match value {
            UsdValue::Float3(v) => Vec3::new(v[0], v[1], v[2]),
            _ => Vec3::ZERO,
        }
    }

    /// Extract a quaternion from a `UsdValue`.
    /// USD stores quatf as (real, i, j, k) = (w, x, y, z).
    /// We attempt to read from a Float3Array with one 3-element + separate w,
    /// but since UsdValue lacks Float4, we fall back to identity.
    fn extract_quat(value: &UsdValue) -> Quat {
        // If stored as Float3 (euler fallback), convert
        if let UsdValue::Float3(v) = value {
            return Self::euler_xyz_to_quat(Vec3::new(v[0], v[1], v[2]));
        }
        Quat::IDENTITY
    }

    /// Convert Euler angles (degrees) in XYZ order to a quaternion.
    fn euler_xyz_to_quat(degrees: Vec3) -> Quat {
        let r = degrees * (std::f32::consts::PI / 180.0);
        Quat::from_euler(glam::EulerRot::XYZ, r.x, r.y, r.z)
    }

    /// Try to extract op names from an xformOpOrder value.
    fn extract_op_names(value: &UsdValue) -> Option<Vec<String>> {
        match value {
            UsdValue::StringArray(arr) => Some(arr.clone()),
            UsdValue::String(s) => {
                // Single token like "xformOp:translate"
                Some(vec![s.clone()])
            }
            UsdValue::Token(s) => Some(vec![s.clone()]),
            _ => None,
        }
    }

    /// Default composition order: scale → rotate → translate.
    /// Only includes ops that are actually authored on the prim.
    fn default_op_order(prim: &UsdPrim) -> Vec<String> {
        let candidates = [
            "xformOp:scale",
            "xformOp:rotateXYZ",
            "xformOp:orient",
            "xformOp:translate",
        ];
        candidates
            .iter()
            .filter(|name| prim.attribute(name).is_some())
            .map(|s| s.to_string())
            .collect()
    }
}
