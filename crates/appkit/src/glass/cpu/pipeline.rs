//! The stages, in order, over one layer of glass.

use super::backdrop::Backdrop;
use super::blur::blur;
use super::canvas::GlassCanvas;
use super::super::material::GlassMaterial;
use super::shade::{Shader, SHADOW_REACH};
use super::super::shape::{GlassBounds, GlassShape};

/// Glass that renders together: one capture, one blur and one shading pass,
/// however many shapes it has. This is what `NSGlassEffectContainerView`
/// batches, and `spacing` is its merge distance.
#[derive(Debug, Clone, Copy)]
pub struct GlassLayer<'a> {
    pub shapes: &'a [GlassShape],
    pub spacing: f32,
    pub material: GlassMaterial,
}

/// What one render did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GlassStats {
    pub captures: u32,
    /// Texels blurred, in the downsampled copy.
    pub texels: usize,
    /// Canvas pixels the glass covered.
    pub covered: usize,
}

impl GlassStats {
    pub fn merge(self, other: GlassStats) -> GlassStats {
        GlassStats {
            captures: self.captures + other.captures,
            texels: self.texels + other.texels,
            covered: self.covered + other.covered,
        }
    }
}

/// Runs glass layers, keeping its working memory from frame to frame.
#[derive(Default)]
pub struct GlassPipeline {
    backdrop: Backdrop,
    spare: Vec<[f32; 3]>,
}

impl GlassPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render `layer` onto `canvas`, touching nothing outside `clip`.
    pub fn render(&mut self, canvas: &mut impl GlassCanvas, layer: &GlassLayer, clip: Option<GlassBounds>) -> GlassStats {
        let whole = GlassBounds { x0: 0, y0: 0, x1: canvas.width(), y1: canvas.height() };
        let limit = clip.map_or(whole, |c| GlassBounds {
            x0: c.x0.max(whole.x0),
            y0: c.y0.max(whole.y0),
            x1: c.x1.min(whole.x1),
            y1: c.y1.min(whole.y1),
        });
        let m = &layer.material;
        let bulge = layer.spacing * 0.25 + 1.0;
        // A shadow is the widest thing glass paints; with none, only the shape.
        let reach = bulge + if m.shadow_strength > 0.0 { SHADOW_REACH } else { 0.0 };
        let Some(area) = GlassBounds::around(layer.shapes, reach, limit) else {
            return GlassStats::default();
        };
        // The lens and the blur both read past the shape, so the capture goes
        // past it too, as far as the canvas allows.
        let margin = bulge + m.blur_radius + m.lens;
        let Some(region) = GlassBounds::around(layer.shapes, margin, whole) else {
            return GlassStats::default();
        };

        self.backdrop.capture(canvas, region, Backdrop::factor_for(m.blur_radius));
        let sigma = self.backdrop.sigma_in_texels(m.blur_radius * 0.5);
        let (width, height) = (self.backdrop.width, self.backdrop.height);
        blur(&mut self.backdrop.rgb, &mut self.spare, width, height, sigma);

        let shader = Shader { shapes: layer.shapes, spacing: layer.spacing, material: m, backdrop: &self.backdrop };
        let covered = shader.paint(canvas, area);
        GlassStats { captures: 1, texels: width * height, covered }
    }
}
