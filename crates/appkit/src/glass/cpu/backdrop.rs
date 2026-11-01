//! Stage one: copy what lies below the glass into a small image of its own.

use super::canvas::{to_rgb, GlassCanvas};
use super::super::shape::GlassBounds;

/// A downsampled copy of a region of the canvas. It keeps its allocation
/// between frames.
#[derive(Default)]
pub(super) struct Backdrop {
    pub rgb: Vec<[f32; 3]>,
    pub width: usize,
    pub height: usize,
    region: Option<GlassBounds>,
    factor: usize,
}

impl Backdrop {
    /// How much smaller the copy is. A wide blur loses nothing at a quarter
    /// size and costs a sixteenth as much.
    pub fn factor_for(blur_radius: f32) -> usize {
        match blur_radius {
            r if r >= 12.0 => 4,
            r if r >= 6.0 => 2,
            _ => 1,
        }
    }

    /// Average each `factor` x `factor` block of `region` into one texel.
    pub fn capture(&mut self, canvas: &impl GlassCanvas, region: GlassBounds, factor: usize) {
        self.region = Some(region);
        self.factor = factor;
        self.width = region.width().div_ceil(factor);
        self.height = region.height().div_ceil(factor);
        self.rgb.clear();
        self.rgb.reserve(self.width * self.height);
        for ty in 0..self.height {
            let ys = region.y0 + ty * factor..(region.y0 + (ty + 1) * factor).min(region.y1);
            for tx in 0..self.width {
                let xs = region.x0 + tx * factor..(region.x0 + (tx + 1) * factor).min(region.x1);
                let mut sum = [0.0f32; 3];
                let mut count = 0.0f32;
                for y in ys.clone() {
                    for x in xs.clone() {
                        let c = to_rgb(canvas.pixel(x, y));
                        sum = [sum[0] + c[0], sum[1] + c[1], sum[2] + c[2]];
                        count += 1.0;
                    }
                }
                self.rgb.push([sum[0] / count, sum[1] / count, sum[2] / count]);
            }
        }
    }

    /// The copy at a canvas position, filtered, and held at its edges.
    pub fn sample(&self, x: f32, y: f32) -> [f32; 3] {
        let Some(region) = self.region else { return [0.0; 3] };
        let f = self.factor as f32;
        let tx = ((x - region.x0 as f32) / f - 0.5).clamp(0.0, (self.width - 1) as f32);
        let ty = ((y - region.y0 as f32) / f - 0.5).clamp(0.0, (self.height - 1) as f32);
        let (ix, iy) = (tx as usize, ty as usize);
        let (nx, ny) = ((ix + 1).min(self.width - 1), (iy + 1).min(self.height - 1));
        let (fx, fy) = (tx - ix as f32, ty - iy as f32);
        let at = |px: usize, py: usize| self.rgb[py * self.width + px];
        let (a, b, c, d) = (at(ix, iy), at(nx, iy), at(ix, ny), at(nx, ny));
        std::array::from_fn(|i| {
            let top = a[i] + (b[i] - a[i]) * fx;
            let bottom = c[i] + (d[i] - c[i]) * fx;
            top + (bottom - top) * fy
        })
    }

    /// Reach of a Gaussian of canvas sigma `sigma`, in texels of this copy.
    pub fn sigma_in_texels(&self, sigma: f32) -> f32 {
        sigma / self.factor.max(1) as f32
    }
}
