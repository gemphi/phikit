//! Stage three: each pixel of the shape from the blurred backdrop, then the
//! shadow it casts on what is outside it.

use super::backdrop::Backdrop;
use super::canvas::{from_rgb, to_rgb, GlassCanvas};
use super::material::GlassMaterial;
use super::shape::{union_distance, GlassBounds, GlassShape};

/// How far in from the edge the rim bends light and catches it.
const RIM_BAND: f32 = 30.0;
const SHADOW_DROP: f32 = 10.0;
/// Where the shadow has faded to nothing.
pub(super) const SHADOW_REACH: f32 = 48.0;
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
        // Distance changes by at most one per pixel, so one reading says how
        // far a run stays deep inside (no rim to work out) or beyond the shadow
        // (nothing to do). Most of a panel is one or the other.
        let beyond = SHADOW_REACH + SHADOW_DROP + 1.0;
        for y in area.y0..area.y1 {
            let py = y as f32 + 0.5;
            let mut x = area.x0;
            while x < area.x1 {
                let px = x as f32 + 0.5;
                let d = self.distance(px, py);
                let left = area.x1 - x;
                if d < -RIM_BAND - 1.0 {
                    let run = ((-d - RIM_BAND) as usize).clamp(1, left);
                    for i in 0..run {
                        let c = self.backdrop.sample(px + i as f32, py);
                        canvas.set_pixel(x + i, y, from_rgb(self.colored(c)));
                    }
                    covered += run;
                    x += run;
                    continue;
                }
                if d > beyond {
                    x += ((d - beyond) as usize).clamp(1, left);
                    continue;
                }
                x += 1;
                let x = x - 1;
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
        // A smooth fall to exactly zero at the reach, so it leaves no edge.
        let t = 1.0 - d / SHADOW_REACH;
        let amount = t * t * t * m.shadow_strength;
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
        let pull = band * band * band.sqrt() * m.lens;
        let c = self.colored(self.backdrop.sample(px - normal[0] * pull, py - normal[1] * pull));
        // Edge light: a rim that faces the light, and a soft glow inside it.
        let rim = (1.0 - (d + 1.0).abs() / 2.2).clamp(0.0, 1.0);
        let facing = (normal[0] * LIGHT[0] + normal[1] * LIGHT[1]).max(0.0);
        let light = rim * (0.2 + 0.8 * facing * facing) * m.edge_strength + band.powi(4) * 0.07 * m.edge_strength;
        std::array::from_fn(|i| c[i] + m.edge[i] * light)
    }

    /// Saturation about the pixel's own luminance, then the tint, which keeps
    /// contrast steady over any backdrop.
    #[inline]
    fn colored(&self, c: [f32; 3]) -> [f32; 3] {
        let m = self.material;
        let luma = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        std::array::from_fn(|i| {
            let vivid = luma + (c[i] - luma) * m.saturation;
            vivid + (m.tint[i] - vivid) * m.tint_amount
        })
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
