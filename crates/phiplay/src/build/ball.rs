//! The ball: a sphere of the sport's size in a cover cut the sport's way.
//!
//! Each cover is worked out on the sphere, then painted into an
//! equirectangular image (row 0 the north pole, +z), so the seams fall
//! where they fall on a real ball rather than where a texture's edges are:
//!
//! - a basketball's eight panels — two great circles crossing at right
//!   angles, and two curved seams each a circle about the meridian's axis
//!   that crosses only the equator;
//! - a volleyball's or netball's eighteen — the six faces of a cube pushed
//!   out onto the sphere, three strips to a face, the strips turned a
//!   quarter from one face to the next so they interlock;
//! - a soccer ball's thirty-two — twelve pentagons about an icosahedron's
//!   vertices and twenty hexagons about its faces.

use super::image::BImage;
use super::mesh::BMesh;
use super::part::{BMat, BPart};
use crate::sport::{BBallCut, BBallSpec};
use glam::{Mat4, Vec3};
use std::sync::Arc;

/// Seams are this wide either side, as an angle (radians): about 4 mm
/// on a 0.11 m ball.
const SEAM: f32 = 0.018;

/// The ball, at the origin.
pub fn ball(spec: &BBallSpec, name: &str) -> BPart {
    let image = Arc::new(cover(spec, 1024, 512));
    let mat = BMat::plain(format!("{name} Cover"), spec.panels[0], 0.5).with_image(image);
    BPart::new(name, BMesh::sphere(spec.radius, 48, 24), mat, Mat4::IDENTITY)
}

/// The cover, `w` × `h`, equirectangular.
pub fn cover(spec: &BBallSpec, w: u32, h: u32) -> BImage {
    let centres = soccer_centres();
    BImage::painted(w, h, |u, v| {
        let (theta, phi) = (std::f32::consts::PI * v, std::f32::consts::TAU * u);
        let d = Vec3::new(theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos());
        let (panel, seam) = match spec.cut {
            BBallCut::Basketball => basketball(d),
            BBallCut::Strips => strips(d),
            BBallCut::Soccer => soccer(d, &centres),
        };
        if seam { spec.seam } else { spec.panels[panel % spec.panels.len()] }
    })
}

/// Which of the eight panels `d` is on, and whether it is on a seam.
fn basketball(d: Vec3) -> (usize, bool) {
    let curved = |sign: f32| ((sign * d.x).clamp(-1.0, 1.0).acos() - 0.92).abs();
    let seam = d.z.abs() < SEAM || d.x.abs() < SEAM || curved(1.0) < SEAM || curved(-1.0) < SEAM;
    // Every panel is one colour on a basketball; the index only matters
    // for a two-tone cover.
    (0, seam)
}

/// A volleyball's strip: the cube face `d` pushes out from, the strip
/// across that face, and whether it lies on a seam.
fn strips(d: Vec3) -> (usize, bool) {
    let a = d.abs();
    let k = if a.x >= a.y && a.x >= a.z { 0 } else if a.y >= a.z { 1 } else { 2 };
    let comps = [d.x, d.y, d.z];
    let major = comps[k].abs();
    // The strips run across the axis after this face's on even faces and
    // the one after that on odd ones: the quarter turn that interlocks them.
    let face = 2 * k + usize::from(comps[k] < 0.0);
    let across = comps[(k + 1 + face % 2) % 3] / major; // −1 … 1 on the face
    let strip = (((across + 1.0) / 2.0 * 3.0).floor() as usize).min(2);
    let edge = {
        let others = [comps[(k + 1) % 3].abs(), comps[(k + 2) % 3].abs()];
        // Near the face's own edge: another face is nearly as major.
        others.iter().any(|o| (major - o) < SEAM * 1.5)
    };
    let between = [-1.0 / 3.0, 1.0 / 3.0].iter().any(|b| (across - b).abs() * major < SEAM);
    // The middle strip takes the first colour; the outer two the second and
    // third by face, so each face shows all three.
    let colour = if strip == 1 { 0 } else { 1 + (face + strip / 2) % 2 };
    (colour, edge || between)
}

/// The thirty-two face centres of a truncated icosahedron: the twelve
/// vertices of an icosahedron (pentagons) and its twenty face centres
/// (hexagons), all on the unit sphere — pentagons first.
fn soccer_centres() -> Vec<Vec3> {
    let g = (1.0 + 5f32.sqrt()) / 2.0;
    let mut v = Vec::new();
    for (a, b) in [(1.0, g), (-1.0, g), (1.0, -g), (-1.0, -g)] {
        v.push(Vec3::new(0.0, a, b));
        v.push(Vec3::new(a, b, 0.0));
        v.push(Vec3::new(b, 0.0, a));
    }
    let vertices: Vec<Vec3> = v.into_iter().map(Vec3::normalize).collect();
    // A face is three vertices mutually one edge apart (the edge is the
    // smallest distance between two vertices).
    let edge = 2.0 / (g * g + 1.0f32).sqrt() * 1.0001;
    let mut faces = Vec::new();
    for i in 0..12 {
        for j in i + 1..12 {
            for k in j + 1..12 {
                let near = |a: Vec3, b: Vec3| a.distance(b) < edge * 1.01;
                if near(vertices[i], vertices[j]) && near(vertices[j], vertices[k]) && near(vertices[i], vertices[k]) {
                    faces.push((vertices[i] + vertices[j] + vertices[k]).normalize());
                }
            }
        }
    }
    vertices.into_iter().chain(faces).collect()
}

/// The panel `d` is on — 0 for a pentagon, 1 for a hexagon — and whether it
/// is on a seam. A pentagon's centre is nearer its edge than a hexagon's
/// (its in-radius is about 0.84 of the hexagon's on a real ball), so the
/// nearest centre is weighed by that before it is chosen.
fn soccer(d: Vec3, centres: &[Vec3]) -> (usize, bool) {
    const PENTAGON: f32 = 0.84;
    let mut best = (f32::MAX, 0usize);
    let mut second = f32::MAX;
    for (i, c) in centres.iter().enumerate() {
        let angle = d.dot(*c).clamp(-1.0, 1.0).acos() / if i < 12 { PENTAGON } else { 1.0 };
        if angle < best.0 {
            second = best.0;
            best = (angle, i);
        } else if angle < second {
            second = angle;
        }
    }
    (usize::from(best.1 >= 12), second - best.0 < SEAM * 1.6)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Panels on a sphere, counted by flooding the non-seam pixels of a
    /// cover with its u wrap and its poles joined.
    fn panels(seam: impl Fn(Vec3) -> bool) -> usize {
        let (w, h) = (360usize, 180usize);
        let at = |x: usize, y: usize| {
            let (theta, phi) = (std::f32::consts::PI * (y as f32 + 0.5) / h as f32, std::f32::consts::TAU * (x as f32 + 0.5) / w as f32);
            Vec3::new(theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos())
        };
        let open: Vec<bool> = (0..w * h).map(|i| !seam(at(i % w, i / w))).collect();
        let mut seen = vec![false; w * h];
        let mut count = 0;
        for start in 0..w * h {
            if !open[start] || seen[start] {
                continue;
            }
            let mut stack = vec![start];
            seen[start] = true;
            let mut size = 0;
            while let Some(i) = stack.pop() {
                size += 1;
                let (x, y) = (i % w, i / w);
                let mut next = vec![y * w + (x + 1) % w, y * w + (x + w - 1) % w];
                if y > 0 { next.push((y - 1) * w + x) } else { next.push(y * w + (x + w / 2) % w) }
                if y + 1 < h { next.push((y + 1) * w + x) } else { next.push(y * w + (x + w / 2) % w) }
                for n in next {
                    if open[n] && !seen[n] {
                        seen[n] = true;
                        stack.push(n);
                    }
                }
            }
            if size > 40 {
                count += 1;
            }
        }
        count
    }

    #[test]
    fn each_cover_has_its_real_panel_count() {
        assert_eq!(panels(|d| basketball(d).1), 8, "a basketball");
        assert_eq!(panels(|d| strips(d).1), 18, "a volleyball");
        let centres = soccer_centres();
        assert_eq!(centres.len(), 32);
        assert_eq!(panels(|d| soccer(d, &centres).1), 32, "a soccer ball");
    }
}
