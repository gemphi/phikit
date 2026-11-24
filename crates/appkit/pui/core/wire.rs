//! A model as wire, as a modelling tool's three side views have it: every
//! edge of every triangle drawn as a thin line in one ink on the scene's
//! ground, nothing hidden, so its make is seen through it. An edge two
//! triangles share is drawn once. Lines are drawn larger and averaged
//! down as the theme says, as the shaded pictures are.

use std::collections::HashSet;

use super::lens::PCamera;
use super::orbit::{PPiece, ground, smooth};
use super::raster::PRaster;
use crate::theme::PTheme;

/// A place in space as an edge's end is known by, so a shared edge is
/// known once.
type Key = [i64; 3];

fn key(v: [f32; 3]) -> Key {
    v.map(|c| (c * 1.0e5).round() as i64)
}

/// The pieces' edges drawn into a picture of that size, in an ink: its
/// width, height and points.
pub fn wire(piece: &[PPiece], camera: &PCamera, size: (u32, u32), ink: [u8; 4], theme: &dyn PTheme) -> (u32, u32, Vec<u8>) {
    let by = smooth(theme);
    let (w, h) = (size.0.max(1) * by, size.1.max(1) * by);
    let mut raster = PRaster::new(w, h, ground(theme));
    let lens = camera.lens();
    let mut seen: HashSet<(Key, Key)> = HashSet::new();
    for tri in piece.iter().flat_map(|p| p.triangle.iter()) {
        for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let (ka, kb) = (key(a), key(b));
            if !seen.insert((ka.min(kb), ka.max(kb))) {
                continue;
            }
            let (p, q) = (lens.project(a, w as f32, h as f32), lens.project(b, w as f32, h as f32));
            if p[2] > camera.near && q[2] > camera.near {
                line(&mut raster, (p[0], p[1]), (q[0], q[1]), ink, by as f32);
            }
        }
    }
    raster.smooth(by)
}

/// A line from one place to another, as thick as `thick` points, in an ink.
fn line(raster: &mut PRaster, from: (f32, f32), to: (f32, f32), ink: [u8; 4], thick: f32) {
    let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs()).ceil().max(1.0) as usize;
    let half = (thick / 2.0).max(0.5);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let (x, y) = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
        let (x0, x1, y0, y1) = ((x - half).floor() as i64, (x + half).ceil() as i64, (y - half).floor() as i64, (y + half).ceil() as i64);
        for py in y0.max(0)..y1.min(raster.height as i64) {
            for px in x0.max(0)..x1.min(raster.width as i64) {
                let at = (py as usize * raster.width as usize + px as usize) * 4;
                raster.pixel[at..at + 4].copy_from_slice(&ink);
            }
        }
    }
}
