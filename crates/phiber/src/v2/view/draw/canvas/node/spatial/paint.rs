//! The canvas element: one `canvas()` that paints the graph as solids.
//!
//! The one place pixels exist, as `paint/canvas.rs` is for the flat editor:
//! the element tree is built before the region is measured, so the camera is
//! made here, from the flat view and the region the paint pass finally knows.
//!
//! A painter's ordering, far to near. Every face and every tube is projected,
//! given the depth of its centre and sorted once; the floor and its grid go
//! first, under everything. Faces turned away from the eye are left out
//! (`BSpatialCamera::faces_eye`), which is all the hidden-surface removal a
//! view of convex solids over a floor needs — one solid can still overdraw a
//! tube that passes behind it at a shallow pitch, and that is the known cost
//! of a painter's sort; a depth buffer is the fix if it ever matters.

use super::super::super::super::ctx::BRenderCtx;
use super::super::view::{BNodeView, store as view_store};
use super::camera::{BOrbit, BSpatialCamera};
use super::ink::BSpatialInk;
use super::layout::{BSolid, BSpatialLayout};
use super::set::TUBE_WIDTH;
use crate::v2::theme::palette::BColor;
use crate::v2::view::draw::color::to_hsla;
use gpui::{Bounds, PathBuilder, Pixels, Window, canvas, fill, point, px};
use std::cell::Cell;
use std::rc::Rc;

/// What one frame needs to paint, gathered before the closure takes it.
pub struct BSpatialPainting {
    pub layout: BSpatialLayout,
    pub view: BNodeView,
    pub orbit: BOrbit,
    /// The card header's height in view units — the unit the relief is in.
    pub header: f32,
    pub ink: BSpatialInk,
}

/// One thing to paint, already on the canvas, with the depth it sorts by.
struct BDrawn {
    depth: f32,
    points: Vec<(f32, f32)>,
    color: BColor,
    /// `None` fills the ring; `Some(width)` strokes the polyline.
    stroke: Option<f32>,
}

impl BSpatialPainting {
    /// The faces of `solid` the eye sees, projected, in `base`'s hue.
    fn faces_of(&self, solid: &BSolid, base: BColor, camera: &BSpatialCamera, out: &mut Vec<BDrawn>) {
        for face in solid.faces() {
            if !camera.faces_eye(face.centre(), face.normal) {
                continue;
            }
            let Some(points) = camera.project_ring(&face.ring) else { continue };
            let depth = points.iter().map(|p| p.2).sum::<f32>() / points.len() as f32;
            let flat: Vec<(f32, f32)> = points.iter().map(|p| (p.0, p.1)).collect();
            out.push(BDrawn { depth, points: flat.clone(), color: BSpatialInk::lit(base, face.normal), stroke: None });
            // A selected solid carries its mark as a rim on the top, the
            // active one wider — Blender ranks active above selected.
            if face.top && solid.rank > 0 {
                let width = self.ink.rim_width * if solid.rank > 1 { 2.0 } else { 1.0 };
                out.push(BDrawn { depth: depth - 1e-3, points: flat, color: self.ink.selected, stroke: Some(width) });
            }
        }
    }

    /// Every tube, as two strokes — the outline under the core, as the flat
    /// noodle is drawn — thinned by distance.
    fn tubes(&self, camera: &BSpatialCamera, out: &mut Vec<BDrawn>) {
        for end in &self.layout.tubes {
            let Some(points) = camera.project_ring(&end.spine(self.header)) else { continue };
            let depth = points.iter().map(|p| p.2).sum::<f32>() / points.len() as f32;
            let flat: Vec<(f32, f32)> = points.iter().map(|p| (p.0, p.1)).collect();
            let width = (TUBE_WIDTH * camera.zoom * camera.dist / depth).max(0.5);
            out.push(BDrawn { depth, points: flat.clone(), color: self.ink.tube_shadow, stroke: Some(width) });
            out.push(BDrawn { depth: depth - 1e-3, points: flat, color: self.ink.tube, stroke: Some(width * super::super::paint::INNER_SCALE) });
        }
    }

    /// The floor grid: lines a power-of-two multiple of the card unit apart,
    /// chosen so they stay at least `MIN_CELL` pixels apart at the floor.
    fn grid(&self, camera: &BSpatialCamera, at: Bounds<Pixels>, window: &mut Window) {
        let Some((x0, y0, x1, y1)) = self.floor_rect() else { return };
        if camera.zoom <= 0.0 || self.header <= 0.0 {
            return;
        }
        let mut step = self.header * 4.0;
        while step * camera.zoom < MIN_CELL {
            step *= 2.0;
        }
        let color = to_hsla(self.ink.grid);
        let line = |a: [f32; 3], b: [f32; 3], window: &mut Window| {
            if let (Some(p), Some(q)) = (camera.project(a), camera.project(b)) {
                let mut path = PathBuilder::stroke(px(self.ink.grid_width));
                path.move_to(point(at.origin.x + px(p.0), at.origin.y + px(p.1)));
                path.line_to(point(at.origin.x + px(q.0), at.origin.y + px(q.1)));
                if let Ok(path) = path.build() {
                    window.paint_path(path, color);
                }
            }
        };
        let mut x = (x0 / step).floor() * step;
        while x <= x1 {
            line([x, y0, 0.0], [x, y1, 0.0], window);
            x += step;
        }
        let mut y = (y0 / step).floor() * step;
        while y <= y1 {
            line([x0, y, 0.0], [x1, y, 0.0], window);
            y += step;
        }
    }

    /// The floor to grid: the view's rectangle and the graph's, united, with
    /// a view's width of margin so the grid runs past what is seen.
    fn floor_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let v = &self.view;
        let graph = self.layout.floor().unwrap_or((v.xmin, v.ymin, v.xmax, v.ymax));
        let m = v.width();
        Some((graph.0.min(v.xmin) - m, graph.1.min(v.ymin) - m, graph.2.max(v.xmax) + m, graph.3.max(v.ymax) + m))
    }
}

/// The closest two grid lines may come on screen, in pixels.
const MIN_CELL: f32 = 48.0;

/// The canvas element, and the cell its paint pass publishes its bounds into
/// — the same arrangement the flat canvas and the animation window use, since
/// a mouse event carries window coordinates and the canvas's own rectangle is
/// not known until it paints.
pub fn element(painting: BSpatialPainting, ctx: &BRenderCtx) -> (gpui::Canvas<()>, Rc<Cell<Option<Bounds<Pixels>>>>) {
    let bounds_cell: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
    let publish = Rc::clone(&bounds_cell);
    let host = ctx.host;
    let element = canvas(
        |_, _, _| (),
        move |bounds, (), window, cx| {
            publish.set(Some(bounds));
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            // The size the next frame fits the flat view to (`view/store.rs`)
            // — the camera is built on that view, so both modes measure once.
            if view_store::measured(host, (w, h)) {
                window.request_animation_frame();
            }
            let camera = BSpatialCamera::of(&painting.view, painting.orbit, (w, h));
            window.paint_quad(fill(bounds, to_hsla(painting.ink.floor)));
            painting.grid(&camera, bounds, window);
            let mut drawn = Vec::new();
            for plate in &painting.layout.plates {
                painting.faces_of(plate, painting.ink.plate, &camera, &mut drawn);
            }
            for solid in &painting.layout.solids {
                let base = painting.ink.base_of(solid.class, solid.muted);
                painting.faces_of(solid, base, &camera, &mut drawn);
            }
            painting.tubes(&camera, &mut drawn);
            // Far first, so the near overdraws it.
            drawn.sort_by(|a, b| b.depth.total_cmp(&a.depth));
            let origin = |p: &(f32, f32)| point(bounds.origin.x + px(p.0), bounds.origin.y + px(p.1));
            for item in &drawn {
                let Some(first) = item.points.first() else { continue };
                let mut path = match item.stroke {
                    Some(width) => PathBuilder::stroke(px(width)),
                    None => PathBuilder::fill(),
                };
                path.move_to(origin(first));
                for p in &item.points[1..] {
                    path.line_to(origin(p));
                }
                if item.stroke.is_none() {
                    path.close();
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, to_hsla(item.color));
                }
            }
            // Names last, over everything: a title is a billboard, not a face.
            painting.titles(&camera, bounds, window, cx);
        },
    );
    (element, bounds_cell)
}
