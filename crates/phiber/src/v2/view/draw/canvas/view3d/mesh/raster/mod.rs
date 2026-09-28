//! A depth-buffered software rasteriser for the viewport.
//!
//! The viewport used to hand GPUI one filled vector path per triangle, in
//! painter order. That cannot draw two surfaces that cross each other — a
//! garment against the body it covers, a hand inside a sleeve — because a
//! sort can only say which triangle goes first, never which one is nearer at
//! a given pixel. It also meant a path object per triangle, which at a few
//! million triangles is the whole frame budget.
//!
//! This fills triangles into a colour buffer with a depth buffer beside it,
//! testing each pixel, and the canvas paints the result as one image. There
//! is no sort at all: order stops mattering once depth decides.

use gpui::Hsla;

/// A colour buffer in BGRA (what `RenderImage` takes) with a depth buffer.
pub(super) struct BFrame {
    pub width: usize,
    pub height: usize,
    /// Four bytes per pixel, blue first. Fully transparent where nothing was
    /// drawn, so the canvas behind shows through.
    pub color: Vec<u8>,
    /// Larger is farther, matching the projector's depth. Starts at infinity.
    depth: Vec<f32>,
}

impl BFrame {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            color: vec![0; width * height * 4],
            depth: vec![f32::INFINITY; width * height],
        }
    }

    /// Fill one triangle in a single colour, keeping whichever surface is
    /// nearer at each pixel.
    #[cfg(test)]
    pub fn triangle(&mut self, corners: [(f32, f32, f32); 3], fill: Hsla) {
        self.shaded(corners, [(0.0, 0.0, 0.0); 3], [[0.0; 2]; 3], |_, _| fill);
    }

    /// Fill one triangle, shading each pixel from its own interpolated
    /// normal and UV, and keeping whichever surface is nearer.
    ///
    /// Corners arrive in canvas space: `x` and `y` are fractions of the
    /// canvas measured from its centre, `z` is the projector's depth. The
    /// shader is generic rather than a trait object so it inlines: it runs
    /// once per covered pixel, of which there are millions.
    pub fn shaded<F: Fn((f32, f32, f32), [f32; 2]) -> Hsla>(
        &mut self,
        corners: [(f32, f32, f32); 3],
        normals: [(f32, f32, f32); 3],
        uvs: [[f32; 2]; 3],
        shade: F,
    ) {
        let (w, h) = (self.width as f32, self.height as f32);
        let p = corners.map(|(x, y, z)| ((0.5 + x) * w, (0.5 + y) * h, z));
        // Twice the signed area. A degenerate triangle covers no pixels.
        let area = (p[1].0 - p[0].0) * (p[2].1 - p[0].1) - (p[1].1 - p[0].1) * (p[2].0 - p[0].0);
        if area.abs() < f32::EPSILON {
            return;
        }
        let low = |v: f32| v.floor().max(0.0) as usize;
        let high = |v: f32, max: usize| v.ceil().min(max as f32 - 1.0).max(0.0) as usize;
        let x0 = low(p[0].0.min(p[1].0).min(p[2].0));
        let x1 = high(p[0].0.max(p[1].0).max(p[2].0), self.width);
        let y0 = low(p[0].1.min(p[1].1).min(p[2].1));
        let y1 = high(p[0].1.max(p[1].1).max(p[2].1), self.height);
        if x0 > x1 || y0 > y1 {
            return;
        }
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                // Barycentric weights, as edge functions over twice the area.
                let w0 = ((p[1].0 - px) * (p[2].1 - py) - (p[1].1 - py) * (p[2].0 - px)) / area;
                let w1 = ((p[2].0 - px) * (p[0].1 - py) - (p[2].1 - py) * (p[0].0 - px)) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let z = w0 * p[0].2 + w1 * p[1].2 + w2 * p[2].2;
                let at = y * self.width + x;
                if z >= self.depth[at] {
                    continue;
                }
                self.depth[at] = z;
                let normal = (
                    w0 * normals[0].0 + w1 * normals[1].0 + w2 * normals[2].0,
                    w0 * normals[0].1 + w1 * normals[1].1 + w2 * normals[2].1,
                    w0 * normals[0].2 + w1 * normals[1].2 + w2 * normals[2].2,
                );
                let uv = [
                    w0 * uvs[0][0] + w1 * uvs[1][0] + w2 * uvs[2][0],
                    w0 * uvs[0][1] + w1 * uvs[1][1] + w2 * uvs[2][1],
                ];
                let bgra = to_bgra(shade(normal, uv));
                self.color[at * 4..at * 4 + 4].copy_from_slice(&bgra);
            }
        }
    }

    /// Box-downsample by `factor` in each direction.
    ///
    /// Triangles are filled without antialiasing — a pixel is in or out — so
    /// the frame is rasterised larger than it is shown and averaged down.
    /// That is what keeps a silhouette from stepping, which the vector paths
    /// gave for free.
    pub fn downsample(&self, factor: usize) -> Self {
        if factor <= 1 {
            return Self {
                width: self.width,
                height: self.height,
                color: self.color.clone(),
                depth: self.depth.clone(),
            };
        }
        let (width, height) = (self.width / factor, self.height / factor);
        let mut out = Self::new(width, height);
        let n = (factor * factor) as u32;
        for y in 0..height {
            for x in 0..width {
                let mut sum = [0u32; 4];
                for dy in 0..factor {
                    for dx in 0..factor {
                        let at = ((y * factor + dy) * self.width + x * factor + dx) * 4;
                        for (c, s) in sum.iter_mut().enumerate() {
                            *s += self.color[at + c] as u32;
                        }
                    }
                }
                let at = (y * width + x) * 4;
                for (c, s) in sum.iter().enumerate() {
                    out.color[at + c] = (s / n) as u8;
                }
            }
        }
        out
    }
}

/// An `Hsla` as the four bytes `RenderImage` wants, blue first.
fn to_bgra(color: Hsla) -> [u8; 4] {
    let rgba: gpui::Rgba = color.into();
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(rgba.b), byte(rgba.g), byte(rgba.r), byte(rgba.a)]
}

#[cfg(test)]
mod tests;
