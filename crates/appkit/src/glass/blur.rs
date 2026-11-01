//! Stage two: three box blurs in each direction, close to a Gaussian. Each
//! pass is a running sum, so its cost does not grow with the radius.

/// The box radius whose three passes have a Gaussian's `sigma`: each box of
/// width `w` adds `(w² - 1) / 12` to the variance.
pub(super) fn box_radius(sigma: f32) -> usize {
    (((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) * 0.5).round().max(0.0) as usize
}

/// Blur `rgb` (`width` x `height`) in place. `spare` is working room, kept by
/// the caller so that no frame allocates.
pub(super) fn blur(rgb: &mut Vec<[f32; 3]>, spare: &mut Vec<[f32; 3]>, width: usize, height: usize, sigma: f32) {
    let radius = box_radius(sigma);
    if radius == 0 || width == 0 || height == 0 {
        return;
    }
    spare.resize(rgb.len(), [0.0; 3]);
    for _ in 0..3 {
        for y in 0..height {
            box_line(rgb, spare, y * width, 1, y * width, 1, width, radius);
        }
        std::mem::swap(rgb, spare);
        for x in 0..width {
            box_line(rgb, spare, x, width, x, width, height, radius);
        }
        std::mem::swap(rgb, spare);
    }
}

/// One line of `len` texels from `src` into `dst`, every `step` texels,
/// holding its ends.
#[allow(clippy::too_many_arguments)]
fn box_line(src: &[[f32; 3]], dst: &mut [[f32; 3]], src0: usize, src_step: usize, dst0: usize, dst_step: usize, len: usize, radius: usize) {
    let at = |i: isize| src[src0 + i.clamp(0, len as isize - 1) as usize * src_step];
    let r = radius as isize;
    let scale = 1.0 / (2 * radius + 1) as f32;
    let mut sum = [0.0f32; 3];
    for i in -r..=r {
        let c = at(i);
        sum = [sum[0] + c[0], sum[1] + c[1], sum[2] + c[2]];
    }
    for i in 0..len as isize {
        dst[dst0 + i as usize * dst_step] = [sum[0] * scale, sum[1] * scale, sum[2] * scale];
        let (add, drop) = (at(i + r + 1), at(i - r));
        sum = [sum[0] + add[0] - drop[0], sum[1] + add[1] - drop[1], sum[2] + add[2] - drop[2]];
    }
}
