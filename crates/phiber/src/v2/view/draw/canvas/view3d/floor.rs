//! Blender's floor grid and X / Y axis lines (`overlay_grid.cc`), drawn in 3D
//! on the Z = 0 plane through the viewport camera instead of as flat 2D
//! guides — the CPU painter's grid, matching the tracer's (`util.wgsl`).
//! Two levels a decade apart are drawn at once: the finer fades out as its
//! cells shrink on screen and the coarser takes over, so zooming subdivides
//! and merges continuously instead of popping. Segments behind a perspective
//! eye are dropped.

use super::extra::BProjectedLine;
use crate::v2::scene::camera::{BViewSetup, SCREEN};
use crate::v2::scene::{MESH_SCALE, project3_full};
use gpui::Hsla;

#[derive(Clone, Copy)]
pub(super) struct BFloorPalette {
    pub grid: Hsla,
    pub axis_x: Hsla,
    pub axis_y: Hsla,
}

/// Segments per grid line (keeps perspective lines straight enough to clip).
const SEGMENTS: usize = 8;

pub(super) fn lines(view: (f32, f32, (f32, f32), f32), setup: BViewSetup, pivot: [f32; 3], pal: BFloorPalette) -> Vec<BProjectedLine> {
    let perspective = setup.perspective;
    let (yaw, pitch, pan, zoom) = view;
    let width = 1.0 / (SCREEN * zoom * MESH_SCALE); // Blender units across the canvas
    // The fine level is the power of ten with at least ~12 cells across the
    // canvas; `f` is how far into that decade the view has zoomed out, and
    // is what fades the fine level toward the coarse one.
    let lg = (width / 12.0).max(1e-4).log10();
    let f = lg - lg.floor();
    let step = 10f32.powf(lg.floor());
    let coarse = step * 10.0;
    let fine_alpha = (1.0 - f) * (1.0 - f) * 0.6;
    let half = (width * if perspective { 3.0 } else { 1.0 } / step).ceil().min(200.0) * step;
    let (cx, cy) = ((pivot[0] / step).round() * step, (pivot[1] / step).round() * step);
    let near = 0.05 / (SCREEN * zoom * setup.fov) * 1.001;
    let at = |x: f32, y: f32| project3_full(x * MESH_SCALE, 0.0, y * MESH_SCALE, yaw, pitch, pan, zoom);
    let mut out = Vec::new();
    let mut push = |a: (f32, f32), b: (f32, f32), color: Hsla| {
        for k in 0..SEGMENTS {
            let t0 = k as f32 / SEGMENTS as f32;
            let t1 = (k + 1) as f32 / SEGMENTS as f32;
            let p = at(a.0 + (b.0 - a.0) * t0, a.1 + (b.1 - a.1) * t0);
            let q = at(a.0 + (b.0 - a.0) * t1, a.1 + (b.1 - a.1) * t1);
            if perspective && (p.2 <= near || q.2 <= near) {
                continue;
            }
            out.push(BProjectedLine { a: (p.0, p.1), b: (q.0, q.1), color });
        }
    };
    let faint = gpui::Hsla { a: pal.grid.a * fine_alpha, ..pal.grid };
    let n = (half / step) as i32;
    for i in -n..=n {
        let o = i as f32 * step;
        let (gx, gy) = (cx + o, cy + o);
        // A line on the coarse level is full strength; one only on the fine
        // level carries the fading alpha.
        let on_coarse = ((o / coarse).round() * coarse - o).abs() < step * 1e-3;
        let color = if on_coarse { pal.grid } else { faint };
        if gx.abs() > step * 1e-3 {
            push((gx, cy - half), (gx, cy + half), color);
        }
        if gy.abs() > step * 1e-3 {
            push((cx - half, gy), (cx + half, gy), color);
        }
    }
    push((cx - half, 0.0), (cx + half, 0.0), pal.axis_x);
    push((0.0, cy - half), (0.0, cy + half), pal.axis_y);
    out
}
