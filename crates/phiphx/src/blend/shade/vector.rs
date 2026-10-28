//! The Vector Math and Vector Rotate nodes, and the Normal node's dot, as
//! Cycles evaluates them: `svm_vector_math` (`kernel/svm/math_util.h`,
//! every `NodeVectorMathType`), `svm_node_vector_rotate`
//! (`kernel/svm/vector_rotate.h`) over `euler_to_transform` and
//! `rotate_around_axis` (`util/transform.h`, `util/math_float3.h`), and
//! the float3 helpers of `util/math_float3.h` they lean on.

type V3 = [f32; 3];

fn map2(a: V3, b: V3, f: impl Fn(f32, f32) -> f32) -> V3 {
    [f(a[0], b[0]), f(a[1], b[1]), f(a[2], b[2])]
}

fn map1(a: V3, f: impl Fn(f32) -> f32) -> V3 {
    [f(a[0]), f(a[1]), f(a[2])]
}

pub(crate) fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub(crate) fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}

fn scale(a: V3, s: f32) -> V3 {
    map1(a, |x| x * s)
}

fn sub(a: V3, b: V3) -> V3 {
    map2(a, b, |x, y| x - y)
}

fn add(a: V3, b: V3) -> V3 {
    map2(a, b, |x, y| x + y)
}

/// `safe_normalize`: the vector itself when it has no length.
pub(crate) fn safe_normalize(a: V3) -> V3 {
    let l = len(a);
    if l > 0.0 { scale(a, 1.0 / l) } else { a }
}

fn safe_divide(a: f32, b: f32) -> f32 {
    if b != 0.0 { a / b } else { 0.0 }
}

/// `safe_floored_fmod`.
fn floored_mod(a: f32, b: f32) -> f32 {
    if b != 0.0 { a - (a / b).floor() * b } else { 0.0 }
}

/// `svm_vector_math`: `(vector, value)` of the operation `op`
/// (`NodeVectorMathType`, `bNode.custom1`) over the node's three vectors
/// and its Scale. `None` for an operation the table does not know.
pub(crate) fn vector_math(op: u32, a: V3, b: V3, c: V3, param: f32) -> Option<(V3, f32)> {
    let v = |v: V3| Some((v, 0.0));
    let f = |f: f32| Some(([0.0; 3], f));
    match op {
        0 => v(add(a, b)),
        1 => v(sub(a, b)),
        2 => v(map2(a, b, |x, y| x * y)),
        3 => v(map2(a, b, safe_divide)),
        4 => v(cross(a, b)),
        // `project`: a's component along b.
        5 => {
            let l2 = dot(b, b);
            v(if l2 != 0.0 { scale(b, dot(a, b) / l2) } else { [0.0; 3] })
        }
        6 => {
            let n = safe_normalize(b);
            v(sub(a, scale(n, 2.0 * dot(a, n))))
        }
        7 => f(dot(a, b)),
        8 => f(len(sub(a, b))),
        9 => f(len(a)),
        10 => v(scale(a, param)),
        11 => v(safe_normalize(a)),
        12 => v(map2(a, b, |x, y| safe_divide(x, y).floor() * y)),
        13 => v(map1(a, f32::floor)),
        14 => v(map1(a, f32::ceil)),
        15 => v(map2(a, b, |x, y| if y != 0.0 { x - (x / y).trunc() * y } else { 0.0 })),
        16 => v(map1(a, |x| x - x.floor())),
        17 => v(map1(a, f32::abs)),
        18 => v(map2(a, b, f32::min)),
        19 => v(map2(a, b, f32::max)),
        // `wrap(value, max, min)`.
        20 => v([0, 1, 2].map(|i| floored_mod(a[i] - c[i], b[i] - c[i]) + c[i])),
        21 => v(map1(a, f32::sin)),
        22 => v(map1(a, f32::cos)),
        23 => v(map1(a, f32::tan)),
        24 => {
            let n = safe_normalize(b);
            let d = dot(n, a);
            let k = 1.0 - param * param * (1.0 - d * d);
            v(if k < 0.0 { [0.0; 3] } else { sub(scale(a, param), scale(n, param * d + k.sqrt())) })
        }
        25 => v(if dot(c, b) < 0.0 { a } else { scale(a, -1.0) }),
        26 => v([0, 1, 2].map(|i| a[i] * b[i] + c[i])),
        27 => v(map2(a, b, super::scalar::compatible_pow)),
        28 => v(map1(a, |x| if x == 0.0 { 0.0 } else { x.signum() })),
        29 => v(map1(a, |x| (x + 0.5).floor())),
        _ => None,
    }
}

/// `euler_to_transform` (`util/transform.h`): the rows `t.x`, `t.y`,
/// `t.z` of Blender's XYZ Euler.
pub(crate) fn euler_rows(e: V3) -> [V3; 3] {
    let (sx, cx) = e[0].sin_cos();
    let (sy, cy) = e[1].sin_cos();
    let (sz, cz) = e[2].sin_cos();
    [
        [cy * cz, sy * sx * cz - cx * sz, sy * cx * cz + sx * sz],
        [cy * sz, sy * sx * sz + cx * cz, sy * cx * sz - sx * cz],
        [-sy, cy * sx, cy * cx],
    ]
}

/// `transform_direction`: the rows dotted with `v`.
fn rows_apply(rows: [V3; 3], v: V3) -> V3 {
    [dot(rows[0], v), dot(rows[1], v), dot(rows[2], v)]
}

/// `transform_direction_transposed`: the columns dotted with `v`.
fn rows_apply_t(rows: [V3; 3], v: V3) -> V3 {
    [0, 1, 2].map(|i| rows[0][i] * v[0] + rows[1][i] * v[1] + rows[2][i] * v[2])
}

/// `rotate_around_axis` (`util/math_float3.h`): Rodrigues about a unit axis.
fn rotate_around_axis(p: V3, axis: V3, angle: f32) -> V3 {
    let (s, c) = angle.sin_cos();
    let k = 1.0 - c;
    let [x, y, z] = axis;
    [
        (c + k * x * x) * p[0] + (k * x * y - z * s) * p[1] + (k * x * z + y * s) * p[2],
        (k * x * y + z * s) * p[0] + (c + k * y * y) * p[1] + (k * y * z - x * s) * p[2],
        (k * x * z - y * s) * p[0] + (k * y * z + x * s) * p[1] + (c + k * z * z) * p[2],
    ]
}

/// `eNodeVectorRotate_Type`: `NODE_VECTOR_ROTATE_TYPE_EULER_XYZ`.
const ROTATE_EULER: u32 = 4;

/// `svm_node_vector_rotate`: `vector` about `center`, by `rotation` (Euler
/// XYZ) or by `angle` about `axis` (or the X / Y / Z axis for types 1–3),
/// inverted when `invert`.
pub(crate) fn vector_rotate(ty: u32, invert: bool, vector: V3, center: V3, axis: V3, angle: f32, rotation: V3) -> V3 {
    let p = sub(vector, center);
    if ty == ROTATE_EULER {
        let rows = euler_rows(rotation);
        return add(if invert { rows_apply_t(rows, p) } else { rows_apply(rows, p) }, center);
    }
    let axis = match ty {
        1 => [1.0, 0.0, 0.0],
        2 => [0.0, 1.0, 0.0],
        3 => [0.0, 0.0, 1.0],
        _ => axis,
    };
    let l = len(axis);
    if l == 0.0 {
        return vector;
    }
    add(rotate_around_axis(p, scale(axis, 1.0 / l), if invert { -angle } else { angle }), center)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn close(a: V3, b: V3) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5)
    }

    #[test]
    fn vector_math_follows_svm_vector_math() {
        assert_eq!(vector_math(7, [1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [0.0; 3], 0.0), Some(([0.0; 3], 32.0)), "dot");
        assert!(close(vector_math(4, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0; 3], 0.0).unwrap().0, [0.0, 0.0, 1.0]), "cross");
        assert!(close(vector_math(10, [1.0, 2.0, 3.0], [0.0; 3], [0.0; 3], 0.5).unwrap().0, [0.5, 1.0, 1.5]), "scale");
        assert!(close(vector_math(3, [1.0, 2.0, 3.0], [2.0, 0.0, 3.0], [0.0; 3], 0.0).unwrap().0, [0.5, 0.0, 1.0]), "safe divide");
        assert!(close(vector_math(20, [1.5, -0.5, 2.5], [1.0; 3], [0.0; 3], 0.0).unwrap().0, [0.5, 0.5, 0.5]), "wrap");
        assert!(close(vector_math(6, [1.0, -1.0, 0.0], [0.0, 2.0, 0.0], [0.0; 3], 0.0).unwrap().0, [1.0, 1.0, 0.0]), "reflect");
        assert_eq!(vector_math(99, [0.0; 3], [0.0; 3], [0.0; 3], 0.0), None);
    }

    /// A quarter turn about Z takes X to Y; inverted, to −Y; the Euler
    /// form agrees with the axis form; a zero axis leaves the vector.
    #[test]
    fn vector_rotate_follows_svm_node_vector_rotate() {
        let x = [1.0, 0.0, 0.0];
        assert!(close(vector_rotate(3, false, x, [0.0; 3], [0.0; 3], FRAC_PI_2, [0.0; 3]), [0.0, 1.0, 0.0]));
        assert!(close(vector_rotate(3, true, x, [0.0; 3], [0.0; 3], FRAC_PI_2, [0.0; 3]), [0.0, -1.0, 0.0]));
        assert!(close(vector_rotate(ROTATE_EULER, false, x, [0.0; 3], [0.0; 3], 0.0, [0.0, 0.0, FRAC_PI_2]), [0.0, 1.0, 0.0]));
        assert!(close(vector_rotate(0, false, x, [0.0; 3], [0.0, 0.0, 2.0], FRAC_PI_2, [0.0; 3]), [0.0, 1.0, 0.0]), "an axis of any length");
        assert!(close(vector_rotate(0, false, x, [0.0; 3], [0.0; 3], FRAC_PI_2, [0.0; 3]), x), "no axis, no turn");
        assert!(close(vector_rotate(3, false, [2.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0; 3], FRAC_PI_2, [0.0; 3]), [1.0, 2.0, 0.0]), "about a centre");
    }
}
