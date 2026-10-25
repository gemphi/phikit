//! A hairstyle grown on her head from its numbers (`BHairSpec`): a shell
//! cut from her scalp and stood off it — thick and rough for an afro,
//! close for a buzz, shaved at the sides for a fade — and what stands off
//! it or hangs from it: puffs and buns as balls, a ponytail as a tapering
//! tube, braids, locs and twists as many thin strands below a parting,
//! cornrows as ridges laid front to back.
//!
//! Every part is weighted wholly to the head bone, so the style turns with
//! her head and never with anything else.

use super::body::{BBody, BPiece};
use super::kit::BGarment;
use super::part::BMat;
use crate::colour::Rgb;
use crate::look::hair::{BHairSpec, BStrands, BVolume};
use glam::Vec3;
use std::f32::consts::{PI, TAU};

const HEAD: &str = "DEF-spine.006";

/// Her head as a ball: its centre, radius, and the crown and nape heights.
#[derive(Clone, Copy, Debug)]
pub struct BHead {
    pub centre: Vec3,
    pub radius: f32,
    pub top: f32,
    pub nape: f32,
}

impl BHead {
    pub fn of(body: &BBody) -> Option<BHead> {
        let g = body.groups.iter().position(|n| n == HEAD)? as u32;
        let on: Vec<Vec3> = body
            .positions
            .iter()
            .zip(&body.weights)
            .filter(|(_, w)| w.iter().any(|&(h, x)| h == g && x > 0.8))
            .map(|(p, _)| Vec3::from(*p))
            .collect();
        if on.is_empty() {
            return None;
        }
        let top = on.iter().map(|p| p.z).fold(f32::MIN, f32::max);
        let nape = body.head(HEAD)?.z;
        // The cranium: the head's upper part, whose mean is the ball's centre.
        let upper: Vec<Vec3> = on.iter().copied().filter(|p| p.z > nape + 0.55 * (top - nape)).collect();
        let centre = upper.iter().sum::<Vec3>() / upper.len().max(1) as f32;
        let centre = Vec3::new(centre.x, centre.y, top - (top - nape) * 0.45);
        let radius = on.iter().filter(|p| p.z > centre.z).map(|p| p.distance(centre)).sum::<f32>() / on.iter().filter(|p| p.z > centre.z).count().max(1) as f32;
        Some(BHead { centre, radius, top, nape })
    }
}

/// The hairstyle, in `colour`.
pub fn hair(body: &BBody, spec: &BHairSpec, colour: Rgb) -> Option<BGarment> {
    let head = BHead::of(body)?;
    let g = body.groups.iter().position(|n| n == HEAD)? as u32;
    let height = head.top - head.nape;
    // The hairline: high across the forehead, down to `reach` of the way
    // from the crown to the nape at the back.
    let low_back = head.top - spec.reach * (head.top - head.nape + 0.02);
    let low_front = head.nape + 0.72 * height;
    let allowed = |c: Vec3| {
        let front = ((head.centre.y - c.y) / head.radius).clamp(0.0, 1.0);
        c.z > low_back + (low_front - low_back) * front.powf(0.6)
    };
    let noise = |p: Vec3| ((p.x * 61.0).sin() * (p.y * 53.0).cos() * (p.z * 47.0).sin()).abs();
    let faded = spec.faded_sides;
    let thickness = move |p: Vec3| {
        let side = faded && (p.x.abs() > 0.045) && p.z < head.top - 0.04;
        let base = if side { 0.003 } else { spec.shell };
        base * (1.0 + spec.curl * 0.5 * noise(p)) + 0.002
    };
    let cut = body.bisect(2, low_back);
    let mut piece = cut.piece(|face, c| cut.share(face, &[HEAD]) > 0.6 && allowed(c), thickness, |p| [p.x * 4.0 + 0.5, p.z * 4.0], |_| 0);
    let pinned = vec![(g, 1.0)];
    match spec.volume {
        BVolume::None => {}
        BVolume::Puff { radius, at, pair } => {
            let crown = |side: f32| {
                // Up the back of the head from the nape (0) to the top (1).
                let a = PI * (0.5 + 0.5 * at);
                let dir = Vec3::new(side * 0.6, -a.cos() * 0.9, a.sin()).normalize();
                head.centre + dir * (head.radius + radius * 0.55)
            };
            for side in if pair { vec![-1.0, 1.0] } else { vec![0.0] } {
                add(&mut piece, &ball(crown(side), radius, spec.curl), &pinned);
            }
        }
    }
    match spec.strands {
        BStrands::None => {}
        BStrands::Tail { length, width } => {
            let root = head.centre + Vec3::new(0.0, head.radius * 0.95, head.radius * 0.25);
            add(&mut piece, &tube(root, Vec3::new(0.0, 0.45, -1.0).normalize(), length, width, width * 0.35, false), &pinned);
        }
        BStrands::Hanging { count, length, width, braid } => {
            for k in 0..count {
                // Round the back and sides below the parting, not over the face.
                let a = PI * (-0.15 + 1.3 * k as f32 / f32::from(count - 1).max(1.0));
                let out = Vec3::new(a.cos(), a.sin().abs() * 0.9 + 0.1, 0.0).normalize();
                let root = head.centre + out * head.radius * 1.02 + Vec3::Z * head.radius * (0.15 + 0.2 * (k % 3) as f32 / 3.0);
                let down = (Vec3::new(out.x * 0.25, out.y * 0.35, -1.0)).normalize();
                add(&mut piece, &tube(root, down, length * (0.85 + 0.15 * ((k * 7) % 5) as f32 / 4.0), width, width * 0.8, braid), &pinned);
            }
        }
        BStrands::Rows { count } => {
            for i in 0..count {
                let x = head.radius * 0.8 * (2.0 * i as f32 / f32::from(count - 1).max(1.0) - 1.0);
                let lane = (head.radius.powi(2) - x * x).max(0.0).sqrt() + 0.004;
                // Over the head in the lane's plane: from the forehead (her
                // front is −y) over the crown to the nape.
                let points: Vec<Vec3> = (0..=16)
                    .map(|k| {
                        let a = 0.3 + (1.05 * PI - 0.3) * k as f32 / 16.0;
                        head.centre + Vec3::new(x, -a.cos() * lane, a.sin() * lane)
                    })
                    .collect();
                add(&mut piece, &path_tube(&points, 0.007), &pinned);
            }
        }
    }
    Some(BGarment { name: "Hair".into(), piece, materials: vec![BMat::plain("Hair", colour, 0.55)] })
}

/// Appends `mesh`'s polygons to `piece`, every vertex pinned with `weights`.
fn add(piece: &mut BPiece, mesh: &(Vec<Vec3>, Vec<Vec<u32>>), weights: &[(u32, f32)]) {
    let base = piece.positions.len() as u32;
    for p in &mesh.0 {
        piece.positions.push((*p).into());
        piece.uvs.push([0.5, 0.5]);
        piece.weights.push(weights.to_vec());
    }
    for f in &mesh.1 {
        piece.faces.push(f.iter().map(|v| v + base).collect());
        piece.face_materials.push(0);
    }
}

/// A rough ball: a UV sphere roughened by `curl`.
fn ball(centre: Vec3, radius: f32, curl: f32) -> (Vec<Vec3>, Vec<Vec<u32>>) {
    let (seg, rings) = (16u32, 10u32);
    let mut verts = Vec::new();
    for r in 0..=rings {
        let th = PI * r as f32 / rings as f32;
        for s in 0..seg {
            let ph = TAU * s as f32 / seg as f32;
            let n = Vec3::new(th.sin() * ph.cos(), th.sin() * ph.sin(), th.cos());
            let bump = 1.0 + curl * 0.08 * ((s * 7 + r * 3) % 5) as f32 / 4.0;
            verts.push(centre + n * radius * bump);
        }
    }
    let mut faces = Vec::new();
    for r in 0..rings {
        for s in 0..seg {
            let (a, b) = (r * seg + s, r * seg + (s + 1) % seg);
            faces.push(vec![a, b, b + seg, a + seg]);
        }
    }
    (verts, faces)
}

/// A tube from `root` along `dir`, `length` long, its radius going from
/// `r0` to `r1`; plaited (`braid`) as a chain of swelling links.
fn tube(root: Vec3, dir: Vec3, length: f32, r0: f32, r1: f32, braid: bool) -> (Vec<Vec3>, Vec<Vec<u32>>) {
    let steps = 12;
    let points: Vec<Vec3> = (0..=steps).map(|k| root + dir * length * k as f32 / steps as f32).collect();
    let radii: Vec<f32> = (0..=steps)
        .map(|k| {
            let t = k as f32 / steps as f32;
            let r = r0 + (r1 - r0) * t;
            if braid { r * (0.8 + 0.35 * (t * steps as f32 * PI).sin().abs()) } else { r }
        })
        .collect();
    sweep(&points, &radii)
}

/// A tube of one radius along a path of points.
fn path_tube(points: &[Vec3], r: f32) -> (Vec<Vec3>, Vec<Vec<u32>>) {
    sweep(points, &vec![r; points.len()])
}

/// A six-sided tube swept along `points`, capped at neither end.
fn sweep(points: &[Vec3], radii: &[f32]) -> (Vec<Vec3>, Vec<Vec<u32>>) {
    let sides = 6u32;
    let mut verts = Vec::new();
    for (k, p) in points.iter().enumerate() {
        let d = if k + 1 < points.len() { points[k + 1] - *p } else { *p - points[k - 1] }.normalize_or(Vec3::Z);
        let a = d.any_orthonormal_vector();
        let b = d.cross(a);
        for s in 0..sides {
            let ang = TAU * s as f32 / sides as f32;
            verts.push(*p + (a * ang.cos() + b * ang.sin()) * radii[k]);
        }
    }
    let mut faces = Vec::new();
    for k in 0..points.len() as u32 - 1 {
        for s in 0..sides {
            let (a, b) = (k * sides + s, k * sides + (s + 1) % sides);
            faces.push(vec![a, b, b + sides, a + sides]);
        }
    }
    (verts, faces)
}
