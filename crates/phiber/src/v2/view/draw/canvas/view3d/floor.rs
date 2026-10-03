//! Blender's floor grid and X / Y / Z axis lines (`overlay_grid.cc`), drawn in 3D
//! on the Z = 0 plane through the viewport camera instead of as flat 2D
//! guides — the CPU painter's grid, matching the tracer's (`util.wgsl`).
//! Two levels a decade apart are drawn at once: the finer fades out as its
//! cells shrink on screen and the coarser takes over, so zooming subdivides
//! and merges continuously instead of popping. Segments behind a perspective
//! eye are dropped. Each part shows as the Overlays popover asks: the floor,
//! and each axis on its own (`show_floor`, `show_axis_x` / `_y` / `_z`); with
//! an axis off, its line is an ordinary floor line.

use super::extra::BProjectedLine;
use crate::v2::scene::camera::{BViewSetup, SCREEN};
use crate::v2::scene::{BOverlays, MESH_SCALE, project3_full};
use gpui::Hsla;

#[derive(Clone, Copy)]
pub(super) struct BFloorPalette {
    pub grid: Hsla,
    pub axis_x: Hsla,
    pub axis_y: Hsla,
    pub axis_z: Hsla,
}

/// Segments per grid line (keeps perspective lines straight enough to clip).
const SEGMENTS: usize = 8;

pub(super) fn lines(view: (f32, f32, (f32, f32), f32), setup: BViewSetup, pivot: [f32; 3], pal: BFloorPalette, shown: BOverlays) -> Vec<BProjectedLine> {
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
    let at = |p: [f32; 3]| project3_full(p[0] * MESH_SCALE, p[2] * MESH_SCALE, p[1] * MESH_SCALE, yaw, pitch, pan, zoom);
    let mut out = Vec::new();
    let mut push = |a: [f32; 3], b: [f32; 3], color: Hsla| {
        for k in 0..SEGMENTS {
            let t0 = k as f32 / SEGMENTS as f32;
            let t1 = (k + 1) as f32 / SEGMENTS as f32;
            let p = at(std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t0));
            let q = at(std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t1));
            if perspective && (p.2 <= near || q.2 <= near) {
                continue;
            }
            out.push(BProjectedLine { a: (p.0, p.1), b: (q.0, q.1), color, dash: false });
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
        if shown.floor && (gx.abs() > step * 1e-3 || !shown.axis_y) {
            push([gx, cy - half, 0.0], [gx, cy + half, 0.0], color);
        }
        if shown.floor && (gy.abs() > step * 1e-3 || !shown.axis_x) {
            push([cx - half, gy, 0.0], [cx + half, gy, 0.0], color);
        }
    }
    if shown.axis_x {
        push([cx - half, 0.0, 0.0], [cx + half, 0.0, 0.0], pal.axis_x);
    }
    if shown.axis_y {
        push([0.0, cy - half, 0.0], [0.0, cy + half, 0.0], pal.axis_y);
    }
    if shown.axis_z {
        push([0.0, 0.0, -half], [0.0, 0.0, half], pal.axis_z);
    }
    out
}
