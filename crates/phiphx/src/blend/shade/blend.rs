//! Colour math of Blender's Mix / Color Ramp / Hue-Saturation nodes.

use crate::blend::file::types::*;

/// `MA_RAMP_*` blend of `b` over `a` by `t` (alpha from `a`).
pub(crate) fn mix(blend: u8, t: f32, a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let m = 1.0 - t;
    let ch = |x: f32, y: f32| match blend {
        1 => x + t * y,                                   // Add
        2 => x * (m + t * y),                             // Multiply
        3 => x - t * y,                                   // Subtract
        4 => 1.0 - (m + t * (1.0 - y)) * (1.0 - x),       // Screen
        5 => if y != 0.0 { m * x + t * x / y } else { x }, // Divide
        6 => m * x + t * (x - y).abs(),                   // Difference
        7 => m * x + t * x.min(y),                        // Darken
        8 => m * x + t * x.max(y),                        // Lighten
        9 => {                                            // Overlay
            if x < 0.5 { x * (m + 2.0 * t * y) } else { 1.0 - (m + 2.0 * t * (1.0 - y)) * (1.0 - x) }
        }
        17 => x + t * (2.0 * y - 1.0),                    // Linear Light
        18 => m * x + t * (x + y - 2.0 * x * y),          // Exclusion
        _ => m * x + t * y,                               // Mix (and unsupported modes)
    };
    [ch(a[0], b[0]), ch(a[1], b[1]), ch(a[2], b[2]), a[3]]
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

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let k = |n: f32| {
        let k = (n + h * 6.0) % 6.0;
        v - v * s * k.min(4.0 - k).clamp(0.0, 1.0)
    };
    [k(5.0), k(3.0), k(1.0)]
}
