//! The trim, all of it lying on the form so nothing floats: the sketch's
//! lines and scalloped bands — the scallop one half-disc repeated along its
//! band, as an array repeats a shape — the scalloped rim, the back zip and
//! its teeth; and the extras: ribbons wound up the ankle and tied in a bow, a
//! thong, an ankle strap held from the heel.

use super::form::{FLOOR, LENGTH};
use super::sweep::{strip, tube};
use super::{BPiece, Last, ROUND};
use crate::build::mesh::BMesh;
use crate::build::part::BMat;
use crate::kit::footwear::{BBandKind, BExtra};
use glam::{Mat4, Vec2, Vec3};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// A curve round the form at `z(θ)`, `off` out: its points and its length.
fn round_curve(last: &Last, z: &dyn Fn(f32) -> f32, off: f32, n: usize) -> (Vec<f32>, Vec<Vec3>) {
    let thetas: Vec<f32> = (0..n).map(|j| TAU * j as f32 / n as f32).collect();
    let pts = thetas.iter().map(|&t| last.at(z(t), t, off)).collect();
    (thetas, pts)
}

/// Half-discs `r` across hanging below the curve `z(θ)`, one after another
/// all the way round — the drawings' scalloped edging.
fn scallops(last: &Last, z: &dyn Fn(f32) -> f32, off: f32, r: f32) -> BMesh {
    let (thetas, pts) = round_curve(last, z, off, ROUND * 6);
    let n = pts.len();
    let mut lengths = vec![0.0f32; n + 1];
    for i in 0..n {
        lengths[i + 1] = lengths[i] + pts[i].distance(pts[(i + 1) % n]);
    }
    let count = (lengths[n] / (2.0 * r)).round().max(3.0) as usize;
    let mut m = BMesh::default();
    for k in 0..count {
        let want = lengths[n] * (k as f32 + 0.5) / count as f32;
        let i = lengths.partition_point(|&l| l < want).saturating_sub(1).min(n - 1);
        let (t, p) = (thetas[i], pts[i]);
        let along = (pts[(i + 1) % n] - pts[(i + n - 1) % n]).normalize_or_zero();
        let out = last.out(z(t), t, off);
        let mut down = out.cross(along).normalize_or_zero();
        if down.z > 0.0 {
            down = -down;
        }
        let lift = out * 0.0009;
        let centre = m.vertex(p + lift, out, Vec2::splat(0.5));
        for a in 0..=10 {
            let a = PI * a as f32 / 10.0;
            m.vertex(p + lift + (along * a.cos() + down * a.sin()) * r, out, Vec2::new(0.5 + 0.5 * a.cos(), 0.5 + 0.5 * a.sin()));
        }
        for a in 0..10u32 {
            let (x, y) = (centre + 1 + a, centre + 2 + a);
            if (Vec3::from(m.positions[x as usize]) - p).cross(Vec3::from(m.positions[y as usize]) - p).dot(out) > 0.0 {
                m.triangle(centre, x, y);
            } else {
                m.triangle(centre, y, x);
            }
        }
    }
    m
}

/// A stitched line round the form at `z(θ)`.
fn line(last: &Last, z: &dyn Fn(f32) -> f32, off: f32, r: f32) -> BMesh {
    let (_, pts) = round_curve(last, z, off, ROUND * 3);
    tube(&pts, true, 6, |_| r)
}

pub(super) fn trim(last: &Last) -> Vec<BPiece> {
    let spec = last.spec;
    let trim_mat = BMat::plain("Trim", spec.colours.trim, 0.35);
    let mut out: Vec<BPiece> = Vec::new();
    // The bands under the rim, the sketch's lines.
    let ceiling = last.lowest_top() - 0.018;
    for (n, band) in spec.bands.iter().enumerate() {
        if !spec.open && LENGTH * band.back.max(band.front) < ceiling {
            let z = move |t: f32| last.band(band.back, band.front, t) + if band.kind == BBandKind::Wave { 0.009 * (4.0 * t).sin() } else { 0.0 };
            let mut m = line(last, &z, 0.0012, if band.kind == BBandKind::Scallop { 0.0016 } else { 0.0009 });
            if band.kind == BBandKind::Scallop {
                m.merge(&scallops(last, &z, 0.0, 0.0052), Mat4::IDENTITY);
            }
            out.push((format!("Band {}", n + 1), m, trim_mat.clone()));
        }
    }
    if !spec.open {
        if spec.piping {
            let rim = |t: f32| last.top(t) - 0.0025;
            let mut m = line(last, &rim, 0.0012, 0.0017);
            m.merge(&scallops(last, &rim, 0.0, 0.0055), Mat4::IDENTITY);
            out.push(("Rim Scallops".into(), m, trim_mat.clone()));
        }
        if spec.foxing {
            let fox = |t: f32| last.fox(t);
            let mut m = line(last, &fox, 0.0018, 0.0012);
            m.merge(&scallops(last, &fox, 0.0009, 0.0045), Mat4::IDENTITY);
            out.push(("Foxing Edge".into(), m, trim_mat.clone()));
        }
        if spec.zip {
            out.extend(zip(last));
        }
    }
    match spec.extra {
        BExtra::None => {}
        BExtra::Ribbons => out.extend(ribbons(last)),
        BExtra::AnkleStrap => out.extend(ankle_strap(last)),
        BExtra::Thong => out.extend(thong(last)),
    }
    out
}

/// Up the back: the tape, its teeth either side, the pull at the top.
fn zip(last: &Last) -> Vec<BPiece> {
    let (z0, z1) = (0.045, last.top(PI) - 0.008);
    let n = ((z1 - z0) / 0.004) as usize;
    let path: Vec<Vec3> = (0..=n).map(|k| last.at(z0 + (z1 - z0) * k as f32 / n as f32, PI, 0.0009)).collect();
    let across: Vec<Vec3> = (0..=n).map(|k| last.round(z0 + (z1 - z0) * k as f32 / n as f32, PI, 0.0)).collect();
    let outs: Vec<Vec3> = (0..=n).map(|k| last.out(z0 + (z1 - z0) * k as f32 / n as f32, PI, 0.0)).collect();
    let tape = strip(&path, &across, &outs, 0.009, false);
    let mut teeth = BMesh::default();
    let tooth = BMesh::cuboid(Vec3::new(0.0035, 0.0022, 0.0018));
    for (k, p) in path.iter().enumerate() {
        let side = if k % 2 == 0 { 0.0011 } else { -0.0011 };
        let at = *p + across[k] * side + outs[k] * 0.0012;
        teeth.merge(&tooth, Mat4::from_translation(at));
    }
    let top = *path.last().unwrap_or(&Vec3::ZERO) + outs[n] * 0.003;
    teeth.merge(&BMesh::cuboid(Vec3::new(0.0055, 0.002, 0.016)), Mat4::from_translation(top - Vec3::Z * 0.008));
    let metal = BMat::metal("Zip", last.spec.colours.trim, 0.3);
    vec![("Zip Tape".into(), tape, BMat::plain("Zip Tape", last.spec.colours.panel, 0.5)), ("Zip".into(), teeth, metal)]
}

/// Two satin ribbons from the slipper's sides wound up the ankle, crossing
/// front and back, tied in a bow at the back.
fn ribbons(last: &Last) -> Vec<BPiece> {
    let mat = BMat::plain("Ribbon", last.spec.colours.trim, 0.22);
    // From the sides of the opening, across the front of the ankle and
    // round it once more, the two ends meeting at the back in the bow.
    let (z0, z1) = (last.top(FRAC_PI_2) + 0.002, last.form.ankle + 0.06);
    let mut out = Vec::new();
    for (k, (start, turn)) in [(FRAC_PI_2, -1.0f32), (-FRAC_PI_2, 1.0)].into_iter().enumerate() {
        // On the ankle, not the boot the form is: a little inside its face.
        let off = -0.019 + 0.0016 * k as f32;
        let samples = 160;
        let (mut path, mut across, mut outs) = (Vec::new(), Vec::new(), Vec::new());
        for i in 0..=samples {
            let t = i as f32 / samples as f32;
            let theta = start + turn * t * (TAU + 3.0 * FRAC_PI_2);
            let z = z0 + (z1 - z0) * t;
            path.push(last.at(z, theta, off));
            across.push(last.up(z, theta, off));
            outs.push(last.out(z, theta, off));
        }
        out.push((format!("Ribbon {}", k + 1), strip(&path, &across, &outs, 0.0105, false), mat.clone()));
    }
    let knot = last.at(z1, PI, -0.015);
    let back = last.form.fwd.extend(0.0) * -1.0;
    out.push(("Bow".into(), bow(knot, back, last.form.side.extend(0.0)), mat));
    out
}

/// A bow at `knot` facing `face`: the knot, two loops out along `side`, two
/// tails hanging.
pub(super) fn bow(knot: Vec3, face: Vec3, side: Vec3) -> BMesh {
    let mut m = BMesh::sphere(0.0062, 14, 10);
    m = super::moved(m, &|p| p + knot + face * 0.002);
    for sigma in [1.0f32, -1.0] {
        let loop_path: Vec<Vec3> = (0..48).map(|k| {
            let phi = TAU * k as f32 / 48.0;
            knot + side * (sigma * 0.024 * (1.0 - phi.cos())) + Vec3::Z * (0.011 * phi.sin()) + face * (0.004 * (1.0 - phi.cos()))
        }).collect();
        let outs: Vec<Vec3> = (0..48).map(|k| (loop_path[(k + 1) % 48] - loop_path[(k + 47) % 48]).cross(face).normalize_or_zero()).collect();
        let across: Vec<Vec3> = loop_path.iter().map(|_| face).collect();
        m.merge(&strip(&loop_path, &across, &outs, 0.0095, true), Mat4::IDENTITY);
        let tail: Vec<Vec3> = (0..=12).map(|k| {
            let t = k as f32 / 12.0;
            knot + side * (sigma * (0.004 + 0.016 * t)) - Vec3::Z * (0.07 * t) + face * (0.003 + 0.004 * t)
        }).collect();
        let across: Vec<Vec3> = tail.iter().map(|_| side).collect();
        let outs: Vec<Vec3> = tail.iter().map(|_| face).collect();
        m.merge(&strip(&tail, &across, &outs, 0.0095, false), Mat4::IDENTITY);
    }
    m
}

/// A strap round the ankle, a strap up from the heel to hold it, a buckle.
fn ankle_strap(last: &Last) -> Vec<BPiece> {
    let spec = last.spec;
    let mat = BMat::plain("Ankle Strap", spec.colours.upper, 0.42);
    let z0 = last.form.ankle + 0.03;
    let off = -0.01;
    let n = ROUND * 2;
    let thetas: Vec<f32> = (0..n).map(|j| TAU * j as f32 / n as f32).collect();
    let path: Vec<Vec3> = thetas.iter().map(|&t| last.at(z0, t, off)).collect();
    let across: Vec<Vec3> = thetas.iter().map(|&t| last.up(z0, t, off)).collect();
    let outs: Vec<Vec3> = thetas.iter().map(|&t| last.out(z0, t, off)).collect();
    let mut out = vec![("Ankle Strap".to_string(), strip(&path, &across, &outs, 0.009, true), mat.clone())];
    let rim = last.top(PI) - 0.004;
    let back: Vec<Vec3> = (0..=10).map(|k| { let z = rim + (z0 - rim) * k as f32 / 10.0; let off = -0.01 * k as f32 / 10.0; last.at(z, PI, off + 0.0009) }).collect();
    let across: Vec<Vec3> = back.iter().map(|_| last.form.side.extend(0.0)).collect();
    let outs: Vec<Vec3> = back.iter().map(|_| (-last.form.fwd).extend(0.0)).collect();
    out.push(("Heel Strap".into(), strip(&back, &across, &outs, 0.011, false), mat));
    let buckle = last.at(z0, FRAC_PI_2, off + 0.003);
    out.push(("Buckle".into(), super::moved(BMesh::cuboid(Vec3::new(0.004, 0.012, 0.013)), &|p| p + buckle), BMat::metal("Buckle", spec.colours.trim, 0.25)));
    out
}

/// A thong from between the toes over the foot to either side of the sole.
fn thong(last: &Last) -> Vec<BPiece> {
    let foot = last.form.centre(FLOOR);
    let at_floor = |t: f32, share: f32| (foot + last.form.dir(t) * last.form.radius(FLOOR, t) * share).extend(FLOOR + 0.001);
    let gap = at_floor(-0.3, 0.72);
    let mat = BMat::plain("Thong", last.spec.colours.upper, 0.4);
    let mut out = Vec::new();
    for side in [1.45f32, -1.5] {
        let over = last.at(0.042, side.signum() * 0.6, -0.004);
        let anchor = at_floor(side, 1.0);
        let path = spline(&[gap, gap + Vec3::Z * 0.016, over, anchor], 40);
        out.push((if side > 0.0 { "Thong Out" } else { "Thong In" }.to_string(), tube(&path, false, 10, |_| 0.0042), mat.clone()));
    }
    out.push(("Toe Post".into(), super::moved(BMesh::sphere(0.0055, 12, 8), &|p| p + gap + Vec3::Z * 0.016), mat));
    out
}

/// A Catmull-Rom curve through `points`, `samples` long.
fn spline(points: &[Vec3], samples: usize) -> Vec<Vec3> {
    let n = points.len();
    let at = |i: isize| points[i.clamp(0, n as isize - 1) as usize];
    (0..=samples)
        .map(|k| {
            let x = k as f32 / samples as f32 * (n - 1) as f32;
            let i = (x.floor() as isize).min(n as isize - 2);
            let t = x - i as f32;
            let (p0, p1, p2, p3) = (at(i - 1), at(i), at(i + 1), at(i + 2));
            0.5 * ((2.0 * p1) + (p2 - p0) * t + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t + (3.0 * p1 - p0 - 3.0 * p2 + p3) * t * t * t)
        })
        .collect()
}
