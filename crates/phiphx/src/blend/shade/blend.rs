//! Colour math of Blender's Mix / Color Ramp / Hue-Saturation nodes.

use crate::blend::file::types::*;

/// `MA_RAMP_*` blend of `b` over `a` by `t`, keeping `a`'s alpha — what a
/// material's Mix node does (Cycles' `svm_mix` takes the first colour's
/// alpha through whatever the blend mode is). The compositor's Mix blends
/// the alpha too, and calls [`ramp_blend`] directly.
pub(crate) fn mix(blend: u8, t: f32, a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let out = ramp_blend(blend, t, a, b);
    [out[0], out[1], out[2], a[3]]
}

/// `MA_RAMP_*` (`DNA_material_types.h:96`).
pub(crate) const RAMP_BLEND: u8 = 0;
pub(crate) const RAMP_ADD: u8 = 1;
pub(crate) const RAMP_MULT: u8 = 2;
pub(crate) const RAMP_SUB: u8 = 3;
pub(crate) const RAMP_SCREEN: u8 = 4;
pub(crate) const RAMP_DIV: u8 = 5;
pub(crate) const RAMP_DIFF: u8 = 6;
pub(crate) const RAMP_DARK: u8 = 7;
pub(crate) const RAMP_LIGHT: u8 = 8;
pub(crate) const RAMP_OVERLAY: u8 = 9;
pub(crate) const RAMP_DODGE: u8 = 10;
pub(crate) const RAMP_BURN: u8 = 11;
pub(crate) const RAMP_HUE: u8 = 12;
pub(crate) const RAMP_SAT: u8 = 13;
pub(crate) const RAMP_VAL: u8 = 14;
pub(crate) const RAMP_COLOR: u8 = 15;
pub(crate) const RAMP_SOFT: u8 = 16;
pub(crate) const RAMP_LINEAR: u8 = 17;
pub(crate) const RAMP_EXCLUSION: u8 = 18;

/// `ramp_blend` (`blenkernel/intern/material.cc:1891`) whole: `col` blended
/// over `r_col` by `fac`.
///
/// Only `MA_RAMP_BLEND` touches the alpha; every other mode leaves
/// `r_col[3]` as it found it, which is Blender's own behaviour and not an
/// omission. The three HSV modes (Hue, Saturation, Color) are no-ops when
/// the colour they take their channel from is grey — Blender guards on
/// `colS != 0` / `rS != 0` rather than producing a hue out of nothing.
pub(crate) fn ramp_blend(mode: u8, fac: f32, r_col: [f32; 4], col: [f32; 4]) -> [f32; 4] {
    let facm = 1.0 - fac;
    let mut out = r_col;
    let each = |f: &mut dyn FnMut(usize)| (0..3).for_each(f);
    match mode {
        RAMP_BLEND => {
            for k in 0..4 {
                out[k] = facm * r_col[k] + fac * col[k];
            }
        }
        RAMP_ADD => each(&mut |k| out[k] += fac * col[k]),
        RAMP_MULT => each(&mut |k| out[k] *= facm + fac * col[k]),
        RAMP_SUB => each(&mut |k| out[k] -= fac * col[k]),
        RAMP_SCREEN => each(&mut |k| out[k] = 1.0 - (facm + fac * (1.0 - col[k])) * (1.0 - r_col[k])),
        RAMP_DIV => each(&mut |k| {
            if col[k] != 0.0 {
                out[k] = facm * r_col[k] + fac * r_col[k] / col[k];
            }
        }),
        RAMP_DIFF => each(&mut |k| out[k] = facm * r_col[k] + fac * (r_col[k] - col[k]).abs()),
        RAMP_DARK => each(&mut |k| out[k] = r_col[k].min(col[k]) * fac + r_col[k] * facm),
        RAMP_LIGHT => each(&mut |k| out[k] = r_col[k].max(col[k]) * fac + r_col[k] * facm),
        RAMP_OVERLAY => each(&mut |k| {
            out[k] = if r_col[k] < 0.5 {
                r_col[k] * (facm + 2.0 * fac * col[k])
            } else {
                1.0 - (facm + 2.0 * fac * (1.0 - col[k])) * (1.0 - r_col[k])
            };
        }),
        RAMP_DODGE => each(&mut |k| {
            if r_col[k] != 0.0 {
                let tmp = 1.0 - fac * col[k];
                out[k] = if tmp <= 0.0 { 1.0 } else { (r_col[k] / tmp).min(1.0) };
            }
        }),
        RAMP_BURN => each(&mut |k| {
            let tmp = facm + fac * col[k];
            out[k] = if tmp <= 0.0 { 0.0 } else { (1.0 - (1.0 - r_col[k]) / tmp).clamp(0.0, 1.0) };
        }),
        RAMP_HUE => {
            let c = rgb_to_hsv(col);
            if c[1] != 0.0 {
                let r = rgb_to_hsv(r_col);
                let tmp = hsv_to_rgb(c[0], r[1], r[2]);
                each(&mut |k| out[k] = facm * r_col[k] + fac * tmp[k]);
            }
        }
        RAMP_SAT => {
            let r = rgb_to_hsv(r_col);
            if r[1] != 0.0 {
                let c = rgb_to_hsv(col);
                let tmp = hsv_to_rgb(r[0], facm * r[1] + fac * c[1], r[2]);
                each(&mut |k| out[k] = tmp[k]);
            }
        }
        RAMP_VAL => {
            let (r, c) = (rgb_to_hsv(r_col), rgb_to_hsv(col));
            let tmp = hsv_to_rgb(r[0], r[1], facm * r[2] + fac * c[2]);
            each(&mut |k| out[k] = tmp[k]);
        }
        RAMP_COLOR => {
            let c = rgb_to_hsv(col);
            if c[1] != 0.0 {
                let r = rgb_to_hsv(r_col);
                let tmp = hsv_to_rgb(c[0], c[1], r[2]);
                each(&mut |k| out[k] = facm * r_col[k] + fac * tmp[k]);
            }
        }
        RAMP_SOFT => each(&mut |k| {
            // The non-`fac` Screen mix first, then weighted in.
            let sc = 1.0 - (1.0 - col[k]) * (1.0 - r_col[k]);
            out[k] = facm * r_col[k] + fac * ((1.0 - r_col[k]) * col[k] * r_col[k] + r_col[k] * sc);
        }),
        RAMP_LINEAR => each(&mut |k| {
            // Not `x + fac (2 y − 1)`: Blender splits on the *blend* colour,
            // so a `col` above a half lightens by `2 (col − 0.5)` and one
            // below darkens by `2 col − 1`. The single-branch form is right
            // only below a half, which left every highlight short.
            out[k] = if col[k] > 0.5 {
                r_col[k] + fac * (2.0 * (col[k] - 0.5))
            } else {
                r_col[k] + fac * (2.0 * col[k] - 1.0)
            };
        }),
        RAMP_EXCLUSION => each(&mut |k| {
            out[k] = (facm * r_col[k] + fac * (r_col[k] + col[k] - 2.0 * r_col[k] * col[k])).max(0.0);
        }),
        _ => {}
    }
    out
}

/// `rgb_to_hsv` (`math_color.c`): hue and saturation in `0..=1`, value the
/// largest channel.
pub(crate) fn rgb_to_hsv(c: [f32; 4]) -> [f32; 3] {
    let [r, g, b, _] = c;
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let d = mx - mn;
    let h = if d == 0.0 {
        0.0
    } else if mx == r {
        ((g - b) / d).rem_euclid(6.0) / 6.0
    } else if mx == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    [h, if mx != 0.0 { d / mx } else { 0.0 }, mx]
}

/// `BKE_colorband_evaluate` for linear / ease / constant interpolation.
pub(crate) fn ramp(f: &BlendFile, dna: &DnaType, band: &BlendBlock, at: f32) -> Option<[f32; 4]> {
    let tot = f.read_u16(band.data.get(f.off("ColorBand", "tot", dna)?..)?.get(..2)?) as usize;
    let ipo = *band.data.get(f.off("ColorBand", "ipotype", dna)?)?;
    let base = f.off("ColorBand", "data[32]", dna)?;
    let size = 24;
    let mut stops: Vec<(f32, [f32; 4])> = (0..tot.min(32))
        .filter_map(|i| {
            let o = base + i * size;
            let v = |k: usize| band.data.get(o + k * 4..o + k * 4 + 4).map(|b| f.read_f32(b));
            Some((v(4)?, [v(0)?, v(1)?, v(2)?, v(3)?]))
        })
        .collect();
    stops.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (first, last) = (stops.first()?, stops.last()?);
    if at <= first.0 {
        return Some(first.1);
    }
    if at >= last.0 {
        return Some(last.1);
    }
    let i = stops.iter().position(|s| s.0 > at)?;
    let (lo, hi) = (stops[i - 1], stops[i]);
    let span = (hi.0 - lo.0).max(1e-9);
    let mut t = (at - lo.0) / span;
    match ipo {
        1 => t = t * t * (3.0 - 2.0 * t), // ease
        4 => return Some(lo.1),          // constant
        _ => {}
    }
    Some(std::array::from_fn(|k| lo.1[k] + t * (hi.1[k] - lo.1[k])))
}

/// Hue / Saturation / Value node (hue 0.5 = unchanged).
pub(crate) fn hue_sat(c: [f32; 4], hue: f32, sat: f32, val: f32, fac: f32) -> [f32; 4] {
    let [r, g, b, a] = c;
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let d = mx - mn;
    let mut h = if d == 0.0 {
        0.0
    } else if mx == r {
        ((g - b) / d).rem_euclid(6.0) / 6.0
    } else if mx == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    let s = if mx != 0.0 { d / mx } else { 0.0 };
    h = (h + hue + 0.5).fract();
    let (s, v) = ((s * sat).clamp(0.0, 1.0), mx * val);
    let rgb = hsv_to_rgb(h, s, v);
    [r + fac * (rgb[0] - r), g + fac * (rgb[1] - g), b + fac * (rgb[2] - b), a]
}

pub(crate) fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let k = |n: f32| {
        let k = (n + h * 6.0) % 6.0;
        v - v * s * k.min(4.0 - k).clamp(0.0, 1.0)
    };
    [k(5.0), k(3.0), k(1.0)]
}
