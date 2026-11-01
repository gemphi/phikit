//! Where glass meets the view tree: which shapes a view contributes, and when.

use super::cpu::{BufferCanvas, GlassLayer, GlassPipeline};
use super::material::{GlassMaterial, GlassStyle};
use super::shape::{GlassBounds, GlassShape};
use crate::render::RenderContext;
use crate::theme::keys;
use crate::view::NSView;
use crate::{NSColor, NSPoint};
use std::cell::RefCell;

thread_local! {
    static PIPELINE: RefCell<GlassPipeline> = RefCell::new(GlassPipeline::new());
}

impl NSView {
    pub fn glass_style(&self) -> GlassStyle {
        self.glass_style
    }
    /// `NSGlassEffectView.style`. Glass is on with [`NSView::set_glass`].
    pub fn set_glass_style(&mut self, style: GlassStyle) {
        self.glass_style = style;
    }
    /// `NSGlassEffectView.tintColor`. Without one the view's background
    /// colour tints its glass.
    pub fn set_glass_tint(&mut self, tint: Option<NSColor>) {
        self.glass_tint = tint;
    }
    /// `NSGlassEffectContainerView.spacing`. With a value, this view renders
    /// the glass of its whole subtree as one layer, merged within that distance.
    pub fn set_glass_container(&mut self, spacing: Option<f32>) {
        self.glass_container = spacing.map(|s| s.max(0.0));
    }
    pub fn glass_container(&self) -> Option<f32> {
        self.glass_container
    }

    /// The colour this view's glass takes on.
    fn glass_color(&self) -> Option<NSColor> {
        self.glass_tint.or(match self.is_hovered {
            true => self.hover_color.or(self.background_color),
            false => self.background_color,
        })
    }

    /// Draw this view's glass, if it has any. Called with the context at the
    /// view's origin. `true` means its background is already painted.
    pub(crate) fn paint_glass(&self, context: &mut RenderContext) -> bool {
        if !crate::theme::bool_or(keys::GLASS_ENABLED, true) {
            return false;
        }
        if let Some(spacing) = self.glass_container {
            let mut shapes = Vec::new();
            let mut first: Option<&NSView> = None;
            for sub in &self.subviews {
                sub.collect_glass(context.offset, &mut shapes, &mut first);
            }
            if let Some(view) = first {
                let material = GlassMaterial::themed(view.glass_style, view.glass_color());
                render_layer(context, &GlassLayer { shapes: &shapes, spacing, material });
            }
            return false;
        }
        match (self.is_glass, context.glass_batched > 0) {
            (false, _) => false,
            (true, true) => true,
            (true, false) => {
                let shape = self.glass_shape(context.offset);
                let material = GlassMaterial::themed(self.glass_style, self.glass_color());
                render_layer(context, &GlassLayer { shapes: &[shape], spacing: 0.0, material });
                true
            }
        }
    }

    fn glass_shape(&self, origin: NSPoint) -> GlassShape {
        GlassShape::new(origin.x, origin.y, self.frame.width(), self.frame.height(), self.corner_radius)
    }

    /// The glass views at and under this one, in canvas positions.
    /// `origin` is where this view's parent draws from.
    fn collect_glass<'a>(&'a self, origin: NSPoint, shapes: &mut Vec<GlassShape>, first: &mut Option<&'a NSView>) {
        if self.is_hidden {
            return;
        }
        let at = NSPoint::new(origin.x + self.frame.origin.x, origin.y + self.frame.origin.y);
        if self.is_glass {
            shapes.push(self.glass_shape(at));
            first.get_or_insert(self);
        }
        let inner = NSPoint::new(at.x - self.bounds.origin.x, at.y - self.bounds.origin.y);
        for sub in &self.subviews {
            sub.collect_glass(inner, shapes, first);
        }
    }
}

fn render_layer(context: &mut RenderContext, layer: &GlassLayer) {
    let clip = context.clip_rect.map(|c| GlassBounds {
        x0: c.min_x().floor().max(0.0) as usize,
        y0: c.min_y().floor().max(0.0) as usize,
        x1: c.max_x().ceil().max(0.0) as usize,
        y1: c.max_y().ceil().max(0.0) as usize,
    });
    let (width, height) = (context.width, context.height);
    let mut canvas = BufferCanvas::new(context.buffer_mut(), width, height);
    PIPELINE.with(|p| p.borrow_mut().render(&mut canvas, layer, clip));
}
