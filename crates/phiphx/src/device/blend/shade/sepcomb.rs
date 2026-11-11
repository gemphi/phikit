//! The Separate Color and Combine Color nodes — RGB, HSV and HSL — as
//! Cycles evaluates them (`svm_node_separate_color`,
//! `svm_node_combine_color` in `kernel/svm/sepcomb_color.h`, over
//! `svm_separate_color` / `svm_combine_color` in `svm/color_util.h` and
//! `rgb_to_hsl` / `hsl_to_rgb` in `util/color.h`), and the legacy
//! Separate / Combine RGB and HSV nodes, which are the same with the mode
//! fixed.

use super::blend::{hsv_to_rgb, rgb_to_hsv};

/// `NodeCombSepColorMode` (`DNA_node_types.h`).
pub(crate) const COMBSEP_RGB: u32 = 0;
pub(crate) const COMBSEP_HSV: u32 = 1;
pub(crate) const COMBSEP_HSL: u32 = 2;

/// `svm_separate_color`: a colour's three channels in `mode`.
pub(crate) fn separate_color(mode: u32, rgb: [f32; 3]) -> [f32; 3] {
    match mode {
        COMBSEP_HSV => rgb_to_hsv([rgb[0], rgb[1], rgb[2], 1.0]),
        COMBSEP_HSL => rgb_to_hsl(rgb),
        _ => rgb,
    }
}

/// `svm_combine_color`: three channels in `mode` back to a colour.
pub(crate) fn combine_color(mode: u32, c: [f32; 3]) -> [f32; 3] {
    match mode {
        COMBSEP_HSV => hsv_to_rgb(c[0], c[1], c[2]),
        COMBSEP_HSL => hsl_to_rgb(c),
        _ => c,
    }
}

/// Cycles' `rgb_to_hsl` (`util/color.h`): lightness the mean of the
/// extremes capped at 1, saturation over the lightness' side.
pub(crate) fn rgb_to_hsl(rgb: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = rgb;
    let cmax = r.max(g).max(b);
    let cmin = r.min(g).min(b);
    let l = ((cmax + cmin) / 2.0).min(1.0);
    if cmax == cmin {
        return [0.0, 0.0, l];
    }
    let d = cmax - cmin;
    let s = if l > 0.5 { d / (2.0 - cmax - cmin) } else { d / (cmax + cmin) };
    let h = match cmax {
        m if m == r => (g - b) / d + if g < b { 6.0 } else { 0.0 },
        m if m == g => (b - r) / d + 2.0,
        _ => (r - g) / d + 4.0,
    };
    [h / 6.0, s, l]
}

/// Cycles' `hsl_to_rgb` (`util/color.h`).
pub(crate) fn hsl_to_rgb(hsl: [f32; 3]) -> [f32; 3] {
    let [h, s, l] = hsl;
    let nr = ((h * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0);
    let ng = (2.0 - (h * 6.0 - 2.0).abs()).clamp(0.0, 1.0);
    let nb = (2.0 - (h * 6.0 - 4.0).abs()).clamp(0.0, 1.0);
    let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
    [(nr - 0.5) * chroma + l, (ng - 0.5) * chroma + l, (nb - 0.5) * chroma + l]
}

#[cfg(test)]
#[path = "../test/shade/sepcomb.rs"]
mod sepcomb_test;
