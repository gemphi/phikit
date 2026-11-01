//! Stage three: each pixel of the shape from the blurred backdrop, then the
//! shadow it casts on what is outside it.

use super::backdrop::Backdrop;
use super::canvas::{from_rgb, to_rgb, GlassCanvas};
use super::material::GlassMaterial;
use super::shape::{union_distance, GlassBounds, GlassShape};

/// How far in from the edge the rim bends light and catches it.
const RIM_BAND: f32 = 38.0;
const SHADOW_DROP: f32 = 10.0;
const SHADOW_SOFTNESS: f32 = 18.0;
/// Past this the shadow is under one level in 255 at full strength.
pub(super) const SHADOW_REACH: f32 = 90.0;
/// Up and to the left, in canvas directions (y runs down).
const LIGHT: [f32; 2] = [-0.507, -0.862];

pub(super) struct Shader<'a> {
    pub shapes: &'a [GlassShape],
    pub spacing: f32,
    pub material: &'a GlassMaterial,
    pub backdrop: &'a Backdrop,
}

impl Shader<'_> {
    fn distance(&self, x: f32, y: f32) -> f32 {
        union_distance(self.shapes, self.spacing, x, y)
    }

    /// Paint `area`, returning how many pixels the glass itself covered.
    pub fn paint(&self, canvas: &mut impl GlassCanvas, area: GlassBounds) -> usize {
        let mut covered = 0;
        for y in area.y0..area.y1 {
            for x in area.x0..area.x1 {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let d = self.distance(px, py);
                let cover = ((0.75 - d) / 1.5).clamp(0.0, 1.0);
                let below = to_rgb(canvas.pixel(x, y));
                let below = self.shadowed(below, px, py, cover);
                let out = match cover > 0.0 {
                    true => {
                        covered += 1;
                        let glass = self.glass_at(px, py, d);
                        std::array::from_fn(|i| below[i] + (glass[i] - below[i]) * cover)
                    }
                    false => below,
                };
                if cover > 0.0 || out != to_rgb(canvas.pixel(x, y)) {
                    canvas.set_pixel(x, y, from_rgb(out));
                }
            }
        }
        covered
    }

    fn shadowed(&self, below: [f32; 3], px: f32, py: f32, cover: f32) -> [f32; 3] {
        let m = self.material;
        if m.shadow_strength <= 0.0 || cover >= 1.0 {
            return below;
        }
        let d = self.distance(px, py - SHADOW_DROP).max(0.0);
        if d >= SHADOW_REACH {
            return below;
        }
        let amount = (-d / SHADOW_SOFTNESS).exp() * m.shadow_strength;
        std::array::from_fn(|i| below[i] + (m.shadow[i] - below[i]) * amount)
    }

    fn glass_at(&self, px: f32, py: f32, d: f32) -> [f32; 3] {
        let m = self.material;
        let band = (1.0 + d / RIM_BAND).clamp(0.0, 1.0);
        let normal = match band > 0.0 {
            true => self.normal(px, py),
            false => [0.0; 2],
        };
        // The lens: near the rim, read from further inside.
        let pull = band.powf(2.5) * m.lens;
        let c = self.backdrop.sample(px - normal[0] * pull, py - normal[1] * pull);
        // Saturation, about the pixel's own luminance.
        let luma = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        let c: [f32; 3] = std::array::from_fn(|i| luma + (c[i] - luma) * m.saturation);
        // Tint, which keeps contrast steady over any backdrop.
        let c: [f32; 3] = std::array::from_fn(|i| c[i] + (m.tint[i] - c[i]) * m.tint_amount);
        // Edge light: a rim that faces the light, and a soft glow inside it.
        let rim = (1.0 - (d + 1.0).abs() / 2.2).clamp(0.0, 1.0);
        let facing = (normal[0] * LIGHT[0] + normal[1] * LIGHT[1]).max(0.0);
        let light = rim * (0.2 + 0.8 * facing * facing) * m.edge_strength + band.powi(4) * 0.07 * m.edge_strength;
        std::array::from_fn(|i| c[i] + m.edge[i] * light)
    }

    fn normal(&self, px: f32, py: f32) -> [f32; 2] {
        let dx = self.distance(px + 1.0, py) - self.distance(px - 1.0, py);
        let dy = self.distance(px, py + 1.0) - self.distance(px, py - 1.0);
        let len = (dx * dx + dy * dy).sqrt();
        match len > 1e-4 {
            true => [dx / len, dy / len],
            false => [0.0; 2],
        }
    }
}
