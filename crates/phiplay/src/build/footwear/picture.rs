//! A design drawn small, by a z-buffer rasteriser — no renderer needed: the
//! six views the sketch sheets lay out for every design — side (outer),
//! side (inner), front, back, top and sole — each on its own or as one
//! strip (`sheet`). The Project Browser's card is the side (outer) view,
//! toe to the right, as the sketches draw every shoe. Hand it the left
//! shoe's parts (`left`); the side views are mirrored so they read as the
//! sheet's right shoe.

use super::super::image::BImage;
use super::super::part::BPart;
use glam::{Mat3, Vec3};
use std::f32::consts::{FRAC_PI_2, PI};

/// Where a picture is drawn from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BShot {
    SideOuter,
    SideInner,
    Front,
    Back,
    Top,
    Sole,
}

impl BShot {
    /// The sheet's views, in its order.
    pub const SHEET: [BShot; 6] = [Self::SideOuter, Self::SideInner, Self::Front, Self::Back, Self::Top, Self::Sole];

    pub fn label(self) -> &'static str {
        match self {
            Self::SideOuter => "Side (outer)",
            Self::SideInner => "Side (inner)",
            Self::Front => "Front",
            Self::Back => "Back",
            Self::Top => "Top",
            Self::Sole => "Sole",
        }
    }

    /// The turn into view space (the view looks down +y, +z up on the
    /// picture), and whether the picture is mirrored.
    fn view(self) -> (Mat3, bool) {
        match self {
            Self::SideOuter => (Mat3::from_rotation_z(-FRAC_PI_2), true),
            Self::SideInner => (Mat3::from_rotation_z(FRAC_PI_2), true),
            Self::Front => (Mat3::IDENTITY, false),
            Self::Back => (Mat3::from_rotation_z(PI), false),
            Self::Top => (Mat3::from_rotation_x(FRAC_PI_2), false),
            Self::Sole => (Mat3::from_rotation_x(-FRAC_PI_2), false),
        }
    }
}

/// The left shoe's parts of a pair.
pub fn left(parts: &[BPart]) -> Vec<BPart> {
    parts.iter().filter(|p| p.name.contains(" L ")).cloned().collect()
}

/// The metres a picture of `parts` is across, whichever the view: the shoe's
/// largest extent in any direction and a margin, so the sheet's six views are
/// drawn to one size and a pixel is `span / edge` metres in every one.
pub fn span(parts: &[BPart]) -> f32 {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for part in parts {
        for p in &part.mesh.positions {
            let q = part.at.transform_point3(Vec3::from(*p));
            lo = lo.min(q);
            hi = hi.max(q);
        }
    }
    (hi - lo).max_element().max(1e-4) * 1.08
}

/// `parts` drawn `edge` pixels square from `shot`, sRGB with alpha.
pub fn picture(parts: &[BPart], edge: u32, shot: BShot) -> BImage {
    let (w, h) = (edge as usize, edge as usize);
    let (view, mirror) = shot.view();
    let world = |p: [f32; 3], part: &BPart| view * part.at.transform_point3(Vec3::from(p));
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for part in parts {
        for p in &part.mesh.positions {
            let q = world(*p, part);
            lo = lo.min(q);
            hi = hi.max(q);
        }
    }
    let span = span(parts);
    let middle = (lo + hi) * 0.5;
    let px = |q: Vec3| {
        let x = ((q.x - middle.x) / span + 0.5) * w as f32;
        if mirror { w as f32 - x } else { x }
    };
    let py = |q: Vec3| (0.5 - (q.z - middle.z) / span) * h as f32;
    let light = Vec3::new(-0.45, -0.6, 0.66).normalize();
    let mut depth = vec![f32::MAX; w * h];
    let mut rgba = vec![0u8; w * h * 4];
    for part in parts {
        let base = Vec3::from(part.material.base.linear());
        let gloss = 1.0 - part.material.roughness;
        let metal = part.material.metallic;
        let normal = |i: usize| (view * Vec3::from(part.mesh.normals[i])).normalize_or_zero();
        for tri in part.mesh.indices.chunks(3) {
            let v: Vec<Vec3> = tri.iter().map(|&i| world(part.mesh.positions[i as usize], part)).collect();
            let s: Vec<(f32, f32)> = v.iter().map(|q| (px(*q), py(*q))).collect();
            let area = (s[1].0 - s[0].0) * (s[2].1 - s[0].1) - (s[2].0 - s[0].0) * (s[1].1 - s[0].1);
            if area.abs() < 1e-9 {
                continue;
            }
            let fold = |f: fn(f32, f32) -> f32, init: f32, k: fn(&(f32, f32)) -> f32| s.iter().map(k).fold(init, f);
            let (x0, x1) = (fold(f32::min, f32::MAX, |p| p.0).floor().max(0.0) as usize, fold(f32::max, f32::MIN, |p| p.0).ceil().min(w as f32 - 1.0) as usize);
            let (y0, y1) = (fold(f32::min, f32::MAX, |p| p.1).floor().max(0.0) as usize, fold(f32::max, f32::MIN, |p| p.1).ceil().min(h as f32 - 1.0) as usize);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                    let b0 = ((s[1].0 - fx) * (s[2].1 - fy) - (s[2].0 - fx) * (s[1].1 - fy)) / area;
                    let b1 = ((s[2].0 - fx) * (s[0].1 - fy) - (s[0].0 - fx) * (s[2].1 - fy)) / area;
                    let b2 = 1.0 - b0 - b1;
                    if b0 < 0.0 || b1 < 0.0 || b2 < 0.0 {
                        continue;
                    }
                    // Nearer is lower y in view space (the view looks down +y).
                    let d = b0 * v[0].y + b1 * v[1].y + b2 * v[2].y;
                    let k = y * w + x;
                    if d >= depth[k] {
                        continue;
                    }
                    depth[k] = d;
                    let mut n = (normal(tri[0] as usize) * b0 + normal(tri[1] as usize) * b1 + normal(tri[2] as usize) * b2).normalize_or_zero();
                    if n.y > 0.0 {
                        n = -n;
                    }
                    let diffuse = n.dot(light).max(0.0);
                    let half = (light + Vec3::new(0.0, -1.0, 0.0)).normalize();
                    let spec = n.dot(half).max(0.0).powf(8.0 + 60.0 * gloss) * (0.08 + 0.35 * gloss) * (1.0 + 2.0 * metal);
                    let rim = (1.0 - (-n.y).max(0.0)).powi(3) * 0.12;
                    let tint = if metal > 0.5 { base } else { Vec3::ONE };
                    let c = base * (0.22 + 0.85 * diffuse) * (1.0 - 0.6 * metal) + tint * (spec + rim);
                    let srgb = |x: f32| (if x <= 0.0031308 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 }).clamp(0.0, 1.0);
                    rgba[k * 4..k * 4 + 4].copy_from_slice(&[(srgb(c.x) * 255.0) as u8, (srgb(c.y) * 255.0) as u8, (srgb(c.z) * 255.0) as u8, 255]);
                }
            }
        }
    }
    BImage { width: edge, height: edge, rgba }
}

/// The six views side by side, each `edge` square, on nothing.
pub fn sheet(parts: &[BPart], edge: u32) -> BImage {
    let shots = BShot::SHEET;
    let (w, e) = (edge as usize * shots.len(), edge as usize);
    let mut rgba = vec![0u8; w * e * 4];
    for (i, shot) in shots.iter().enumerate() {
        let one = picture(parts, edge, *shot);
        for y in 0..e {
            let src = &one.rgba[y * e * 4..(y + 1) * e * 4];
            let at = (y * w + i * e) * 4;
            rgba[at..at + e * 4].copy_from_slice(src);
        }
    }
    BImage { width: w as u32, height: edge, rgba }
}
