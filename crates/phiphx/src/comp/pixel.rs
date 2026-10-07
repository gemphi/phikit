//! The per-pixel operations, each ported from its node's own
//! `node_build_multi_function` in Blender 5.2.0.
//!
//! Every one takes and returns a straight RGBA quadruple in scene-linear
//! space. Nothing here looks at a neighbour, a frame, or the image's size —
//! that is the whole reason the compositor can run as `eval.rs` runs it.

use crate::blend::shade::blend::{hsv_to_rgb, rgb_to_hsv};

/// A pixel in flight.
pub type BCompPixel = [f32; 4];

/// Rec.709 luminance (`IMB_colormanagement_get_luminance_coefficients` under
/// the default Filmic/AgX config, whose primaries are Rec.709's).
pub const LUMINANCE: [f32; 3] = [0.2126, 0.7152, 0.0722];

fn lerp(a: BCompPixel, b: BCompPixel, t: f32) -> BCompPixel {
    std::array::from_fn(|k| a[k] + t * (b[k] - a[k]))
}

fn rgb(c: BCompPixel, f: impl Fn(f32) -> f32) -> BCompPixel {
    [f(c[0]), f(c[1]), f(c[2]), c[3]]
}

/// `brightness_and_contrast` (`node_composite_brightness_contrast.cc:46`).
///
/// The brightness is a *percentage* and the contrast a percentage halved
/// again — `/100` and `/200` — and the two branches are not symmetric: a
/// positive contrast divides by a multiplier that is clamped away from zero
/// (so contrast 50 does not divide by nothing), a negative one multiplies by
/// a multiplier clamped at zero.
pub fn bright_contrast(c: BCompPixel, brightness: f32, contrast: f32) -> BCompPixel {
    let scaled = brightness / 100.0;
    let mut delta = contrast / 200.0;
    let (multiplier, offset);
    if contrast > 0.0 {
        multiplier = 1.0 / (1.0 - delta * 2.0).max(f32::EPSILON);
        offset = multiplier * (scaled - delta);
    } else {
        delta *= -1.0;
        multiplier = (1.0 - delta * 2.0).max(0.0);
        offset = multiplier * scaled + delta;
    }
    rgb(c, |x| x * multiplier + offset)
}

/// `adjust_exposure` (`node_composite_exposure.cc`): stops, so `2^exposure`.
pub fn exposure(c: BCompPixel, stops: f32) -> BCompPixel {
    let m = stops.exp2();
    rgb(c, |x| x * m)
}

/// `ShaderNodeGamma`: `x^gamma`, negatives left alone (Blender's
/// `safe_powf` gives 0 for a negative base, and a negative colour channel is
/// a real thing in a scene-linear render).
pub fn gamma(c: BCompPixel, g: f32) -> BCompPixel {
    rgb(c, |x| if x > 0.0 { x.powf(g) } else { x })
}

/// `invert` (`node_composite_invert.cc:46`): the colour and the alpha are
/// inverted separately, then the whole thing is mixed back by `factor`.
pub fn invert(c: BCompPixel, factor: f32, color: bool, alpha: bool) -> BCompPixel {
    let mut out = c;
    if color {
        out = [1.0 - out[0], 1.0 - out[1], 1.0 - out[2], out[3]];
    }
    if alpha {
        out[3] = 1.0 - out[3];
    }
    lerp(c, out, factor)
}

/// `hue_saturation_value` (`node_composite_hue_saturation_value.cc`).
///
/// The hue input is an offset with **0.5 as no change**: the node adds
/// `hue + 0.5` and takes the fraction, so the slider's default 0.5 wraps the
/// hue back to itself. Saturation and value are multipliers, and the result
/// is clamped at zero before being mixed back — a value above 1 can push a
/// channel negative through the HSV round trip.
pub fn hue_sat(c: BCompPixel, hue: f32, saturation: f32, value: f32, factor: f32) -> BCompPixel {
    let hsv = rgb_to_hsv(c);
    let h = (hsv[0] + hue + 0.5).fract();
    let out = hsv_to_rgb(h, hsv[1] * saturation, hsv[2] * value);
    let out = [out[0].max(0.0), out[1].max(0.0), out[2].max(0.0), c[3]];
    lerp(c, out, factor)
}

/// `posterize` (`node_composite_posterize.cc`): the step count is clamped to
/// 2..=1024 first, so a tree asking for one step gets two rather than a
/// division by one.
pub fn posterize(c: BCompPixel, steps: f32) -> BCompPixel {
    let steps = steps.clamp(2.0, 1024.0);
    rgb(c, |x| (x * steps).floor() / steps)
}

/// `color_to_luminance` (`node_composite_rgb_to_bw.cc`): a single number,
/// which a colour socket downstream broadcasts to grey.
pub fn rgb_to_bw(c: BCompPixel) -> f32 {
    c[0] * LUMINANCE[0] + c[1] * LUMINANCE[1] + c[2] * LUMINANCE[2]
}

/// `CMP_NODE_SETALPHA_MODE_APPLY` — the default.
pub const SET_ALPHA_APPLY: i32 = 0;
/// `CMP_NODE_SETALPHA_MODE_REPLACE_ALPHA`.
pub const SET_ALPHA_REPLACE: i32 = 1;

/// `set_alpha` (`node_composite_set_alpha.cc`): Apply multiplies the whole
/// quadruple — the alpha included — where Replace swaps the alpha alone.
pub fn set_alpha(c: BCompPixel, alpha: f32, mode: i32) -> BCompPixel {
    match mode {
        SET_ALPHA_REPLACE => [c[0], c[1], c[2], alpha],
        _ => [c[0] * alpha, c[1] * alpha, c[2] * alpha, c[3] * alpha],
    }
}

/// `CMP_NODE_ALPHA_CONVERT_PREMULTIPLY`.
pub const ALPHA_PREMULTIPLY: i32 = 0;
/// `CMP_NODE_ALPHA_CONVERT_UNPREMULTIPLY`.
pub const ALPHA_UNPREMULTIPLY: i32 = 1;

/// `convert_alpha` (`node_composite_alpha_convert.cc`). Unpremultiplying a
/// pixel of zero alpha returns it untouched rather than dividing by it.
pub fn alpha_convert(c: BCompPixel, mode: i32) -> BCompPixel {
    match mode {
        ALPHA_UNPREMULTIPLY if c[3] != 0.0 => [c[0] / c[3], c[1] / c[3], c[2] / c[3], c[3]],
        ALPHA_UNPREMULTIPLY => c,
        _ => [c[0] * c[3], c[1] * c[3], c[2] * c[3], c[3]],
    }
}

/// Alpha Over's three operations (`node_composite_alpha_over.cc`): over,
/// disjoint over and conjoint over.
pub const OVER_PLAIN: i32 = 0;
pub const OVER_DISJOINT: i32 = 1;
pub const OVER_CONJOINT: i32 = 2;

fn premul_foreground(fg: BCompPixel, straight: bool) -> BCompPixel {
    if !straight {
        return fg;
    }
    let a = fg[3].clamp(0.0, 1.0);
    [fg[0] * a, fg[1] * a, fg[2] * a, a]
}

fn safe_div(c: BCompPixel, d: f32) -> BCompPixel {
    if d == 0.0 { [0.0; 4] } else { std::array::from_fn(|k| c[k] / d) }
}

/// `alpha_over` and its disjoint / conjoint siblings. `straight` is the
/// node's Straight Alpha switch: with it on the foreground is premultiplied
/// first, with it off the pixel is taken as already premultiplied.
pub fn alpha_over(bg: BCompPixel, fg: BCompPixel, factor: f32, straight: bool, mode: i32) -> BCompPixel {
    let front = premul_foreground(fg, straight);
    let fa = fg[3].clamp(0.0, 1.0);
    let ba = bg[3].clamp(0.0, 1.0);
    let mixed: BCompPixel = match mode {
        OVER_DISJOINT if fa + ba < 1.0 => std::array::from_fn(|k| front[k] + bg[k]),
        OVER_DISJOINT => {
            let straight_bg = safe_div(bg, ba);
            std::array::from_fn(|k| front[k] + straight_bg[k] * (1.0 - fa))
        }
        OVER_CONJOINT if fa > ba => front,
        OVER_CONJOINT => {
            let ratio = if ba == 0.0 { 0.0 } else { fa / ba };
            std::array::from_fn(|k| front[k] + bg[k] * (1.0 - ratio))
        }
        _ => std::array::from_fn(|k| front[k] + bg[k] * (1.0 - fa)),
    };
    lerp(bg, mixed, factor)
}

/// The compositor's Mix: `ramp_blend` over the colour, with **A's alpha
/// kept**. `clamp` is the node's own Clamp Result switch.
///
/// Measured, not assumed: `ramp_blend`'s own `MA_RAMP_BLEND` arm mixes
/// `r_col[3]` along with the colour, so the first guess here blended the
/// alpha too. Blender's own render of a Mix over a pixel of alpha 0.5
/// comes back at alpha 0.5, not at the blend of 0.5 and 1 — the node takes
/// the first input's alpha through every mode, exactly as a material's Mix
/// does (`shade::blend::mix`, which this calls).
pub fn mix(mode: i32, factor: f32, a: BCompPixel, b: BCompPixel, clamp: bool) -> BCompPixel {
    let out = crate::blend::shade::blend::mix(mode.clamp(0, 255) as u8, factor, a, b);
    if clamp { out.map(|x| x.clamp(0.0, 1.0)) } else { out }
}
