//! What the Graph editor paints for its curves — `graph_draw_curves`:
//! each F-curve (`draw_fcurve`) as the line `curve/line` traces, then —
//! while its keyframes are usable (`BKE_fcurve_are_keyframes_usable`: no
//! replacing Generator) — its handles (`curve/handle`, unless protected)
//! and its keys as vertices (`draw_fcurve_keyframe_vertices`), unselected
//! curves before selected so the selected stay visible.

use super::super::set::{hsla, rect};
use super::super::BTimeView;
use super::BDrawnCurve;
use crate::phiphx::blend::FLAG_PROTECTED;
use crate::v2::theme::BAnimColors;
use gpui::{Bounds, Corners, Pixels, Window, fill, px};

/// `space_graph.vertex_size`; a protected curve's keys are drawn at 0.8.
const VERTEX_SIZE: f32 = 6.0;
const PROTECTED_VERTEX_SCALE: f32 = 0.8;
/// `set_fcurve_vertex_color`: a vertex fades half as far as its curve, and
/// never under a fifth.
const VERTEX_ALPHA_FLOOR: f32 = 0.2;

fn rgba(color: [f32; 3], alpha: f32) -> gpui::Hsla {
    gpui::Rgba { r: color[0], g: color[1], b: color[2], a: alpha }.into()
}

/// The curve's line colour: its own, faded when unselected; the theme's
/// muted grey when muted.
pub fn line_color(drawn: &BDrawnCurve, colors: &BAnimColors) -> gpui::Hsla {
    if drawn.muted { hsla(colors.muted) } else { rgba(drawn.color, drawn.alpha) }
}

/// `set_fcurve_vertex_color`: `TH_VERTEX` or `TH_VERTEX_SELECT` (muted on a
/// protected curve), faded by half the curve's fade.
pub fn vertex_color(drawn: &BDrawnCurve, selected: bool, colors: &BAnimColors) -> gpui::Hsla {
    let base = if drawn.curve.flag & FLAG_PROTECTED != 0 {
        colors.muted
    } else if selected {
        colors.vertex_select
    } else {
        colors.vertex
    };
    let alpha = (1.0 - (1.0 - drawn.alpha) * 0.5).clamp(VERTEX_ALPHA_FLOOR, 1.0);
    let mut color = hsla(base);
    color.a = alpha;
    color
}

impl BTimeView {
    /// Every curve: its line, then a dot on each key in view.
    pub fn paint_curves(&self, curves: &[BDrawnCurve], colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
        let width = f32::from(at.size.width);
        if width < 2.0 || self.values.is_none() {
            return;
        }
        // `graph_draw_curves(…, 0)` then `(…, 1)`: unselected curves first.
        let mut ordered: Vec<&BDrawnCurve> = curves.iter().collect();
        ordered.sort_by_key(|drawn| drawn.curve.selected());
        for drawn in ordered {
            self.paint_curve_line(drawn, colors, at, window);
            if drawn.curve.generator.is_some() {
                continue;
            }
            // `draw_fcurve_handles_check`: no handles on a protected curve.
            if drawn.curve.flag & FLAG_PROTECTED == 0 {
                self.paint_handles(drawn, colors, at, window);
            }
            self.paint_vertices(drawn, colors, at, window);
        }
    }
}

impl BTimeView {
    fn paint_vertices(&self, drawn: &BDrawnCurve, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
        let width = f32::from(at.size.width);
        let height = f32::from(at.size.height);
        let size = if drawn.curve.flag & FLAG_PROTECTED != 0 { VERTEX_SIZE * PROTECTED_VERTEX_SCALE } else { VERTEX_SIZE };
        let half = size / 2.0;
        // Unselected keys first, selected over them.
        for selected in [false, true] {
            let color = vertex_color(drawn, selected, colors);
            for key in drawn.curve.keys.iter().filter(|key| key.selected == selected) {
                let (x, y) = (self.x_of(key.co[0], width), self.y_of(key.co[1], height));
                if x >= -half && x <= width + half {
                    window.paint_quad(fill(rect(x - half, y - half, x + half, y + half, at), color).corner_radii(Corners::all(px(half))));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::{BFCurve, FLAG_MUTED, FLAG_SELECTED};

    fn drawn(flag: u32, alpha: f32) -> BDrawnCurve {
        let mut curve = BFCurve::new("location", 0, vec![]);
        curve.flag = flag;
        BDrawnCurve { curve, color: [1.0, 0.0, 0.0], alpha, active: false, muted: flag & FLAG_MUTED != 0 }
    }

    fn colors() -> BAnimColors {
        let state = crate::v2::state::BState::new();
        crate::v2::theme::resolve_ui_style(crate::v2::theme::style::ui_style_defaults(crate::v2::theme::BPaletteName::Sphere), &state).anim()
    }

    #[test]
    fn vertices_fade_half_as_far_as_their_curve_and_muted_curves_go_grey() {
        let colors = colors();
        let faded = drawn(0, 0.25);
        assert!((vertex_color(&faded, false, &colors).a - 0.625).abs() < 1e-6);
        assert_eq!(vertex_color(&faded, true, &colors).a, vertex_color(&faded, false, &colors).a);
        assert_eq!(vertex_color(&drawn(FLAG_SELECTED, 1.0), true, &colors).a, 1.0);
        let selected = vertex_color(&drawn(FLAG_SELECTED, 1.0), true, &colors);
        assert_eq!((selected.h, selected.s, selected.l), (hsla(colors.vertex_select).h, hsla(colors.vertex_select).s, hsla(colors.vertex_select).l), "a selected key is the theme's warm accent");
        let plain = vertex_color(&drawn(0, 1.0), false, &colors);
        assert_eq!(plain.l, hsla(colors.vertex).l);
        let protected = vertex_color(&drawn(FLAG_PROTECTED, 1.0), true, &colors);
        assert_eq!(protected.l, hsla(colors.muted).l, "a protected curve's keys go muted");
        assert_eq!(line_color(&drawn(FLAG_MUTED, 1.0), &colors), hsla(colors.muted));
        assert_eq!(line_color(&drawn(0, 0.25), &colors).a, 0.25);
    }
}
