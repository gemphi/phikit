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
