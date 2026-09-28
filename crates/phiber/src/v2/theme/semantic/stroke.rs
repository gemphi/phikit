//! The strokes, the Image checker, the sidebar's floor and the shadow — the
//! resolved style's lines and effects, each a ladder step or a stroke width
//! from the resolved strokes.

use super::{BCheckerColors, BStrokeStyle};
use crate::v2::theme::{BColor, BElevation, BResolvedUiStyle, BShadowStyle};

impl BResolvedUiStyle {
    pub fn border_stroke(&self) -> BStrokeStyle {
        BStrokeStyle { color: self.colors.border, width: self.strokes.border }
    }

    pub fn separator_stroke(&self) -> BStrokeStyle {
        BStrokeStyle { color: self.colors.sep, width: self.strokes.hairline }
    }

    /// The grid line every canvas draws — the 3D floor and the UV editor's
    /// square alike (`draw_image_grid`): the anim canvases' grid shade lifted
    /// a tenth, a grid stroke wide. One line, so the shapes match up.
    pub fn viewport_grid_stroke(&self) -> BStrokeStyle {
        BStrokeStyle { color: self.anim().grid_line, width: self.strokes.grid }
    }

    pub fn axis_stroke(&self, mut color: BColor) -> BStrokeStyle {
        color.3 = self.effects.axis_alpha;
        BStrokeStyle { color, width: self.strokes.grid }
    }

    /// The node canvas's dots: the step inward of the canvas, at the grid
    /// alpha — light on a dark canvas, like every grid.
    pub fn node_grid_stroke(&self) -> BStrokeStyle {
        let mut color = self.step(self.colors.bg_viewport, 1);
        color.3 = self.effects.node_grid_alpha;
        BStrokeStyle { color, width: self.strokes.grid }
    }

    /// The Image editor's checker: the canvas and the step under it.
    pub fn image_checker_colors(&self) -> BCheckerColors {
        BCheckerColors { light: self.colors.bg_viewport, dark: self.step(self.colors.bg_viewport, -1) }
    }

    pub fn sidebar_min_width(&self) -> f32 {
        self.dimensions.sidebar_width.max(self.dimensions.sidebar_min_width)
    }

    /// What a surface at `elevation` casts in this theme: the theme's own
    /// shadow is the ceiling a modal carries, and each level under it is a
    /// rung softer (`style/elevation`). A modal, a dropdown, a floating
    /// toolbar and a docked card used to share one shadow, so nothing in the
    /// window said which of them was nearer the reader.
    pub fn shadow_for(&self, elevation: BElevation) -> BShadowStyle {
        elevation.shadow(self.effects.surface_shadow)
    }

    /// The theme's shadow ceiling — what Preferences edits, and what a modal
    /// casts. Draw sites want [`Self::shadow_for`] instead.
    pub fn surface_shadow(&self) -> BShadowStyle {
        self.effects.surface_shadow
    }
}
