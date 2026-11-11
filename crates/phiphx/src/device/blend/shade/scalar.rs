//! Scalar and per-channel node math — Blender's Bright/Contrast, Gamma,
//! Math and Clamp nodes (`gpu_shader_material_bright_contrast.glsl`,
//! `gpu_shader_material_gamma.glsl`, the `NODE_MATH_*` functions and the
//! safe helpers of `gpu_shader_math_safe_lib.glsl`).

/// `brightness_contrast`: `a = 1 + contrast`, `b = brightness − contrast / 2`,
/// each channel `max(a · c + b, 0)`; alpha untouched.
pub(crate) fn bright_contrast(c: [f32; 4], brightness: f32, contrast: f32) -> [f32; 4] {
    let a = 1.0 + contrast;
    let b = brightness - contrast * 0.5;
    [(a * c[0] + b).max(0.0), (a * c[1] + b).max(0.0), (a * c[2] + b).max(0.0), c[3]]
}

/// `node_gamma`: `c^gamma` on the channels above zero (`compatible_pow`).
pub(crate) fn gamma(c: [f32; 4], gamma: f32) -> [f32; 4] {
    let ch = |x: f32| if x > 0.0 { compatible_pow(x, gamma) } else { x };
    [ch(c[0]), ch(c[1]), ch(c[2]), c[3]]
}

/// `compatible_pow`: C's `pow` — `x^0 = 1`, a negative base is allowed for
/// an even or odd integer exponent, and `0^y = 0`.
pub(crate) fn compatible_pow(x: f32, y: f32) -> f32 {
    if y == 0.0 {
        return 1.0;
    }
    if x < 0.0 {
        return if (-y).rem_euclid(2.0) == 0.0 { (-x).powf(y) } else { -((-x).powf(y)) };
    }
    if x == 0.0 {
        return 0.0;
    }
    x.powf(y)
}

fn safe_divide(a: f32, b: f32) -> f32 {
    if b != 0.0 { a / b } else { 0.0 }
}

/// `compatible_mod`: C's truncated remainder, 0 when `b` is 0.
fn compatible_mod(a: f32, b: f32) -> f32 {
    if b != 0.0 { a - (a / b).trunc() * b } else { 0.0 }
}

/// `SHD_MATH_CLAMP`.
pub(crate) const MATH_CLAMP: u16 = 1;

/// The Math node, `NODE_MATH_*` by `custom1`, over its up to three inputs.
/// `None` for an operation this table does not know.
pub(crate) fn math(operation: u16, a: f32, b: f32, c: f32) -> Option<f32> {
    use std::f32::consts::PI;
    Some(match operation {
        0 => a + b,
        1 => a - b,
        2 => a * b,
        3 => safe_divide(a, b),
        4 => a.sin(),
        5 => a.cos(),
        6 => a.tan(),
        7 => if a.abs() <= 1.0 { a.asin() } else { 0.0 },
        8 => if a.abs() <= 1.0 { a.acos() } else { 0.0 },
        9 => a.atan(),
        10 => compatible_pow(a, b),
        11 => if a > 0.0 && b > 0.0 { a.ln() / b.ln() } else { 0.0 },
        12 => a.min(b),
        13 => a.max(b),
        14 => (a + 0.5).floor(),
        15 => f32::from(a < b),
        16 => f32::from(a > b),
        17 => compatible_mod(a, b),
        18 => a.abs(),
        19 => a.atan2(b),
        20 => a.floor(),
        21 => a.ceil(),
        22 => a - a.floor(),
        23 => a.max(0.0).sqrt(),
        24 => if a > 0.0 { 1.0 / a.sqrt() } else { 0.0 },
        25 => if a > 0.0 { 1.0 } else if a < 0.0 { -1.0 } else { 0.0 },
        26 => a.exp(),
        27 => a * PI / 180.0,
        28 => a * 180.0 / PI,
        29 => a.sinh(),
        30 => a.cosh(),
        31 => a.tanh(),
        32 => a.trunc(),
        33 => safe_divide(a, b).floor() * b,
        34 => {
            let range = c - b;
            if range != 0.0 { a - range * ((a - b) / range).floor() } else { b }
        }
        35 => f32::from((a - b).abs() <= c.max(1e-5)),
        36 => a * b + c,
        37 => {
            if b != 0.0 { b - (compatible_mod(a - b, 2.0 * b).abs() - b).abs() } else { 0.0 }
        }
        38 | 39 => {
            let (a, b) = if operation == 39 { (-a, -b) } else { (a, b) };
            let out = if c != 0.0 {
                let h = ((c - (a - b).abs()) / c).clamp(0.0, 1.0);
                a.min(b) - h * h * h * c * (1.0 / 6.0)
            } else {
                a.min(b)
            };
            if operation == 39 { -out } else { out }
        }
        40 => if b != 0.0 { a - (a / b).floor() * b } else { 0.0 },
        _ => return None,
    })
}

/// The Clamp node: `custom1` 0 is Min/Max, 1 is Range (min and max may be
/// swapped).
pub(crate) fn clamp(range: u16, value: f32, min: f32, max: f32) -> f32 {
    if range == 1 { value.clamp(min.min(max), min.max(max)) } else { value.max(min).min(max) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bright_contrast_and_gamma_follow_the_glsl() {
        assert_eq!(bright_contrast([0.5, 0.25, 1.0, 0.7], 0.1, 0.4), [0.5 * 1.4 - 0.1, 0.25 * 1.4 - 0.1, 1.4 - 0.1, 0.7]);
        assert_eq!(bright_contrast([0.0, 0.0, 0.0, 1.0], -1.0, 0.0), [0.0, 0.0, 0.0, 1.0], "clamped at zero");
        let g = gamma([0.25, -0.5, 0.0, 1.0], 0.5);
        assert!((g[0] - 0.5).abs() < 1e-6);
        assert_eq!((g[1], g[2], g[3]), (-0.5, 0.0, 1.0), "only positive channels are raised");
    }

    #[test]
    fn math_is_blenders_table() {
        assert_eq!(math(3, 1.0, 0.0, 0.0), Some(0.0), "safe divide");
        assert_eq!(math(10, -2.0, 3.0, 0.0), Some(-8.0), "compatible pow keeps a negative base");
        assert_eq!(math(17, 7.5, 2.0, 0.0), Some(1.5));
        assert_eq!(math(33, 7.3, 2.0, 0.0), Some(6.0), "snap");
        assert_eq!(math(34, 5.5, 0.0, 2.0), Some(1.5), "wrap into [0, 2)");
        assert_eq!(math(35, 1.0, 1.0004, 0.001), Some(1.0), "compare within epsilon");
        assert_eq!(math(36, 2.0, 3.0, 4.0), Some(10.0), "multiply add");
        assert_eq!(math(37, 3.0, 2.0, 0.0), Some(1.0), "ping-pong");
        assert_eq!(math(40, -1.0, 3.0, 0.0), Some(2.0), "floored modulo");
        assert_eq!(math(41, 1.0, 1.0, 1.0), None, "an unknown operation folds to nothing");
        assert_eq!(clamp(1, 0.5, 1.0, 0.0), 0.5, "range clamp accepts swapped bounds");
        assert_eq!(clamp(0, 0.5, 1.0, 0.0), 0.0, "min/max clamp applies min then max");
    }
}

/// `svm_node_map_range` (`kernel/svm/map_range.h`): `value` taken from
/// `[from_min, from_max]` to `[to_min, to_max]` by `ty`
/// (`NodeMapRangeType`: 0 linear, 1 stepped, 2 smoothstep, 3
/// smootherstep — the smooth two are symmetric, so a reversed source
/// range needs no special case), then, with the node's Clamp, held to
/// the target range either way round (`MapRangeNode::expand` adds a
/// range Clamp after the node). A source range of no width gives 0.
#[allow(clippy::too_many_arguments)]
pub(crate) fn map_range(ty: u32, value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32, steps: f32, clamp: bool) -> f32 {
    if from_max == from_min {
        return 0.0;
    }
    let t = (value - from_min) / (from_max - from_min);
    let factor = match ty {
        1 => if steps > 0.0 { (t * (steps + 1.0)).floor() / steps } else { 0.0 },
        2 => {
            let x = t.clamp(0.0, 1.0);
            x * x * (3.0 - 2.0 * x)
        }
        3 => {
            let x = t.clamp(0.0, 1.0);
            x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
        }
        _ => t,
    };
    let r = to_min + factor * (to_max - to_min);
    match (clamp, to_min > to_max) {
        (false, _) => r,
        (true, true) => r.clamp(to_max, to_min),
        (true, false) => r.clamp(to_min, to_max),
    }
}

/// A scalar node between a texture and the socket it drives, its other
/// inputs folded: a Math node or a Map Range. The tracer carries one as
/// its slot's tail (`BlendMath`); those between the texture and a Color
/// Ramp are composed into the ramp's bake instead (`procedural.rs`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BScalarOp {
    Math { op: u32, b: f32, c: f32, clamp: bool },
    MapRange { ty: u32, from_min: f32, from_max: f32, to_min: f32, to_max: f32, steps: f32, clamp: bool },
}

impl BScalarOp {
    /// The op over `x`.
    pub fn apply(&self, x: f32) -> f32 {
        match *self {
            Self::Math { op, b, c, clamp } => {
                let v = math(op as u16, x, b, c).unwrap_or(x);
                if clamp { v.clamp(0.0, 1.0) } else { v }
            }
            Self::MapRange { ty, from_min, from_max, to_min, to_max, steps, clamp } => map_range(ty, x, from_min, from_max, to_min, to_max, steps, clamp),
        }
    }
}

#[cfg(test)]
mod map_range_tests {
    use super::*;

    /// 0.3 of 0..1 onto 0.2..0.8 is 0.5; stepped at 4 steps floors to
    /// quarters; smoothstep eases; clamp holds an out-of-range value to
    /// the target range, whichever way round it is.
    #[test]
    fn map_range_follows_svm_node_map_range() {
        assert!((map_range(0, 0.3, 0.0, 1.0, 0.2, 0.8, 4.0, false) - 0.38).abs() < 1e-6);
        assert!((map_range(1, 0.3, 0.0, 1.0, 0.0, 1.0, 4.0, false) - 0.25).abs() < 1e-6, "stepped");
        assert!((map_range(2, 0.25, 0.0, 1.0, 0.0, 1.0, 4.0, false) - 0.15625).abs() < 1e-6, "smoothstep");
        assert!((map_range(3, 0.5, 0.0, 1.0, 0.0, 1.0, 4.0, false) - 0.5).abs() < 1e-6, "smootherstep");
        assert_eq!(map_range(0, 2.0, 0.0, 1.0, 0.0, 1.0, 4.0, true), 1.0, "clamped");
        assert_eq!(map_range(0, 2.0, 0.0, 1.0, 0.0, 1.0, 4.0, false), 2.0, "unclamped");
        assert_eq!(map_range(0, 2.0, 0.0, 1.0, 1.0, 0.0, 4.0, true), 0.0, "clamped to a reversed target");
        assert_eq!(map_range(0, 0.5, 1.0, 1.0, 0.0, 1.0, 4.0, false), 0.0, "no source range");
        let op = BScalarOp::MapRange { ty: 0, from_min: 0.0, from_max: 1.0, to_min: 0.2, to_max: 0.8, steps: 4.0, clamp: true };
        assert!((op.apply(0.3) - 0.38).abs() < 1e-6);
        assert_eq!(BScalarOp::Math { op: 2, b: 3.0, c: 0.0, clamp: true }.apply(0.5), 1.0);
    }
}
