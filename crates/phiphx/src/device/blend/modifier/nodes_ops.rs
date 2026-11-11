//! The function nodes the group evaluates, and the angle its Edge Angle
//! node measures — one small file per Blender source:
//!
//! | here | Blender |
//! |---|---|
//! | [`angle_normalized`] | `angle_normalized_v3v3` (`blenlib/intern/math_vector.cc`) |
//! | [`boolean_math`] | `node_fn_boolean_math.cc` (`NodeBooleanMathOperation`) |
//! | [`compare`] | `node_fn_compare.cc` (`NodeCompareOperation`, floats) |
//!
//! The Boolean and Compare tables cover every operation those two nodes
//! have, not only the AND / OR / Less-or-equal the group uses, because a
//! file is free to have saved any of them and `nodes.rs` turns a modifier
//! down rather than guess (an operation outside the tables is `None`).

use super::nodes_field::BField;

/// `angle_normalized_v3v3`: `acos` of the dot, by the more accurate chord.
pub(crate) fn angle_normalized(a: [f32; 3], b: [f32; 3]) -> f32 {
    let len = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let asin = |x: f32| x.clamp(-1.0, 1.0).asin();
    if dot >= 0.0 {
        2.0 * asin(len([a[0] - b[0], a[1] - b[1], a[2] - b[2]]) / 2.0)
    } else {
        std::f32::consts::PI - 2.0 * asin(len([a[0] + b[0], a[1] + b[1], a[2] + b[2]]) / 2.0)
    }
}

pub(super) fn invert(f: BField) -> BField {
    match f {
        BField::Bool(v) => BField::Bool(v.into_iter().map(|b| !b).collect()),
        other => other,
    }
}

impl BField {
    pub fn bools(self) -> Option<Vec<bool>> {
        match self {
            Self::Bool(v) => Some(v),
            Self::Float(v) => Some(v.into_iter().map(|x| x > 0.0).collect()),
        }
    }

    pub fn floats(self) -> Option<Vec<f32>> {
        match self {
            Self::Float(v) => Some(v),
            Self::Bool(v) => Some(v.into_iter().map(|b| if b { 1.0 } else { 0.0 }).collect()),
        }
    }
}

/// `NodeBooleanMathOperation`, by `custom1`.
pub(super) fn boolean_math(op: i32) -> Option<fn(bool, bool) -> bool> {
    Some(match op {
        0 => |a, b| a && b,
        1 => |a, b| a || b,
        2 => |a, _| !a,
        3 => |a, b| !(a && b),
        4 => |a, b| !(a || b),
        5 => |a, b| a == b,
        6 => |a, b| a != b,
        7 => |a, b| !a || b,
        8 => |a, b| a && !b,
        _ => return None,
    })
}

/// `NodeCompareOperation` on floats, with the Epsilon the equality ones take.
pub(super) fn compare(op: i32) -> Option<fn(f32, f32, f32) -> bool> {
    Some(match op {
        0 => |a, b, _| a < b,
        1 => |a, b, _| a <= b,
        2 => |a, b, _| a > b,
        3 => |a, b, _| a >= b,
        4 => |a, b, e| (a - b).abs() <= e,
        5 => |a, b, e| (a - b).abs() > e,
        _ => return None,
    })
}

