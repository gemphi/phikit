//! `image.new`'s generated pictures — `BKE_image_buf_fill_checker` (the UV
//! Grid: 32-pixel checks of two greys with a coloured `+` in each, its hue
//! stepping by an eighth across and down) and `BKE_image_buf_fill_checker_
//! color` (the Color Grid: a hue sweep across, brightening down, tinted
//! by nested checks of 1, 4, 32 and 128 pixels and ruled every 32), as
//! `image_gen.cc` draws them byte for byte, rows up from the bottom as
//! Blender's buffers run, then turned over for the canvas. The grid's
//! lettering (`checker_board_text`, Blender's mono font) is not drawn.

/// The two grids, as `w × h` BGRA rows top down.
pub struct BImageGrid;

impl BImageGrid {
    const CHECK: usize = 32;

    /// `hsv_to_rgb_v`.
    fn hsv(h: f32, s: f32, v: f32) -> [f32; 3] {
        let i = (h * 6.0).floor();
        let f = h * 6.0 - i;
        let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
        match (i as i32).rem_euclid(6) {
            0 => [v, t, p],
            1 => [q, v, p],
            2 => [p, v, t],
            3 => [p, q, v],
            4 => [t, p, v],
            _ => [v, p, q],
        }
    }

    fn byte(v: f32) -> u8 {
        (v * 255.0) as u8
    }

    /// Blender's rows run up from the bottom; the canvas's down from the top.
    fn flipped(w: usize, h: usize, mut rgba: Vec<u8>) -> Vec<u8> {
        let row = 4 * w;
        for y in 0..h / 2 {
            let (a, b) = (y * row, (h - 1 - y) * row);
            for i in 0..row {
                rgba.swap(a + i, b + i);
            }
        }
        rgba.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
        rgba
    }

    /// `image_buf_fill_checker_slice`: the checks, then the coloured `+`.
    pub fn uv(w: u32, h: u32) -> Vec<u8> {
        let (w, h) = (w as usize, h as usize);
        let c = Self::CHECK;
        let mut rgba = vec![0u8; 4 * w * h];
        for y in 0..h {
            let mut dark = if (y / c) % 2 == 0 { 1 } else { -1 };
            for x in 0..w {
                if x % c == 0 {
                    dark = -dark;
                }
                let g = if dark > 0 { 64 } else { 150 };
                rgba[4 * (y * w + x)..4 * (y * w + x) + 4].copy_from_slice(&[g, g, g, 255]);
            }
        }
        for y in 0..h {
            let hoffs = 0.125 * (y / c) as f32;
            for x in 0..w {
                let hue = 0.125 * (x / c) as f32;
                let test_x = (x % c) as i32 - (c / 2) as i32;
                let test_y = (y % c) as i32 - (c / 2) as i32;
                if test_x.abs() < 4 && test_y.abs() < 4 && (test_x.abs() < 1 || test_y.abs() < 1) {
                    let rgb = Self::hsv((hue - hoffs).abs() % 1.0, 0.9, 0.9);
                    rgba[4 * (y * w + x)..4 * (y * w + x) + 4].copy_from_slice(&[Self::byte(rgb[0]), Self::byte(rgb[1]), Self::byte(rgb[2]), 255]);
                }
            }
        }
        Self::flipped(w, h, rgba)
    }

    /// `BLEND_CHAR`: brightened by `add`, held at 255.
    fn blend(v: u8, add: f32) -> u8 {
        v.saturating_add((add * 255.0) as u8)
    }

    /// `checker_board_color_prepare_slice`: the sweep, four tints, the grid.
    pub fn color(w: u32, h: u32) -> Vec<u8> {
        let (w, h) = (w as usize, h as usize);
        let mut rgba = vec![0u8; 4 * w * h];
        // `power_of_2_max_i(width / 8)`, at least 8.
        let hue_step = (w / 8).next_power_of_two().max(8);
        for y in 0..h {
            let v = 0.1 + y as f32 * (0.4 / h as f32);
            for x in 0..w {
                let hue = ((x / hue_step) as f64 / w as f64 * hue_step as f64) as f32;
                let rgb = Self::hsv(hue, 1.0, v);
                rgba[4 * (y * w + x)..4 * (y * w + x) + 4].copy_from_slice(&[Self::byte(rgb[0]), Self::byte(rgb[1]), Self::byte(rgb[2]), 255]);
            }
        }
        for (size, add) in [(1usize, 0.03f32), (4, 0.05), (32, 0.07), (128, 0.15)] {
            for y in 0..h {
                for x in 0..w {
                    let same = ((y / size) % 2 == 1 && (x / size) % 2 == 1) || ((y / size) % 2 == 0 && (x / size) % 2 == 0);
                    let a = if same { add } else { add * 0.5 };
                    let p = 4 * (y * w + x);
                    for i in 0..3 {
                        rgba[p + i] = Self::blend(rgba[p + i], a);
                    }
                }
            }
        }
        for y in 0..h {
            for x in 0..w {
                if y % Self::CHECK == 0 || x % Self::CHECK == 0 {
                    let p = 4 * (y * w + x);
                    for i in 0..3 {
                        rgba[p + i] = Self::blend(rgba[p + i], 0.25);
                    }
                }
            }
        }
        Self::flipped(w, h, rgba)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_uv_grid_checks_in_32s_with_a_plus_in_each() {
        let px = BImageGrid::uv(64, 64);
        let at = |x: usize, y: usize| &px[4 * (y * 64 + x)..4 * (y * 64 + x) + 4];
        // Blender's row 0 is the bottom: there the first check is bright
        // (`dark` starts at 1 and flips at x = 0), so the top row — Blender's
        // row 63, `floor(63 / 32)` odd — starts dark.
        assert_eq!(at(2, 2), &[64, 64, 64, 255], "top-left check dark");
        assert_eq!(at(34, 2), &[150, 150, 150, 255], "next check bright");
        assert_eq!(at(2, 61), &[150, 150, 150, 255], "bottom-left bright");
        // The `+` at a check's centre: red at hue 0 (bottom-left check) —
        // BGRA, so red is the third byte.
        let centre = at(16, 63 - 16);
        assert!(centre[2] > 200 && centre[1] < 60 && centre[0] < 60, "{centre:?}");
        assert_eq!(BImageGrid::uv(3, 2).len(), 24);
    }

    #[test]
    fn the_color_grid_sweeps_hue_across_and_brightens_down() {
        let px = BImageGrid::color(256, 128);
        let at = |x: usize, y: usize| [px[4 * (y * 256 + x)], px[4 * (y * 256 + x) + 1], px[4 * (y * 256 + x) + 2]];
        // Top row (Blender's row 127) is brighter than the bottom row (row 0).
        let sum = |c: [u8; 3]| c.iter().map(|v| *v as u32).sum::<u32>();
        assert!(sum(at(5, 1)) > sum(at(5, 126)), "{:?} vs {:?}", at(5, 1), at(5, 126));
        // Hue 0 at the left (red leads), green leads a third of the way
        // across — BGRA: red is the third byte, green the second.
        assert!(at(5, 64)[2] >= at(5, 64)[1] && at(5, 64)[2] >= at(5, 64)[0], "{:?}", at(5, 64));
        assert!(at(90, 64)[1] >= at(90, 64)[0] && at(90, 64)[1] >= at(90, 64)[2], "{:?}", at(90, 64));
    }
}
