//! One seat, from its set: a hexagonal stem, a softly square pan, and the
//! back that names it. Built facing −y (the sitter looks along −y), the pan
//! at `PAN` above the step; the bowl turns each one to face the field.

use super::mesh::BMesh;
use crate::venue::seat::{BBack, BSeatSpec};
use glam::{Mat4, Vec3};
use std::f32::consts::{PI, TAU};

/// The pan's top above the step.
pub const PAN: f32 = 0.43;
/// How far apart seats stand along a row.
pub const PITCH: f32 = 0.55;

/// The seat as one mesh.
pub fn seat(spec: &BSeatSpec) -> BMesh {
    let mut m = BMesh::cylinder(0.035, 0.03, PAN - 0.05, 6);
    let bench = spec.back == BBack::None;
    // The pan: a softly square oval, or for a bench a plank the whole pitch.
    let (pw, pd) = if bench { (PITCH * 0.98, 0.36) } else { (0.42, 0.38) };
    let pan = BMesh::extruded(&superellipse(pw / 2.0, pd / 2.0, 3.0, 12), 0.05);
    // `extruded` stands an outline in x–z and pushes it along +y; turned a
    // quarter about x it lies flat, pushed up.
    m.merge(&pan, Mat4::from_translation(Vec3::new(0.0, 0.0, PAN - 0.05)) * Mat4::from_rotation_x(PI / 2.0));
    if !bench {
        let lean = Mat4::from_translation(Vec3::new(0.0, pd / 2.0 - 0.02, PAN)) * Mat4::from_rotation_x(-spec.lean.to_radians());
        let back = match spec.back {
            BBack::Shell { wrap } => shell(spec.width, spec.height, wrap),
            _ => BMesh::extruded(&outline(spec), 0.035),
        };
        m.merge(&back, lean);
    }
    m
}

/// The back's outline in x–z, from its foot at z = 0, counter-clockwise
/// seen from −y.
pub fn outline(spec: &BSeatSpec) -> Vec<[f32; 2]> {
    let (hw, h) = (spec.width / 2.0, spec.height);
    match spec.back {
        // Half a hexagon on its long side, its two top corners chamfered so
        // the cut reads crisp but not sharp.
        BBack::SemiHex { top } => {
            let c = 0.02;
            vec![[-hw, 0.0], [hw, 0.0], [hw * top + c, h - c * 1.7], [hw * top - c, h], [-hw * top + c, h], [-hw * top - c, h - c * 1.7]]
        }
        BBack::Egg { n, top } => egg(hw, h / 2.0, n, top, usize::from(spec.facets)).into_iter().map(|[x, z]| [x, z + h / 2.0]).collect(),
        BBack::Wing { sweep } => {
            // A delta: full at the foot, swept to a narrow tip.
            let lift = sweep.to_radians().tan() * 0.15;
            vec![[-hw, 0.0], [hw, 0.0], [hw * 0.55, h * 0.55], [0.07, h + lift], [-0.07, h + lift], [-hw * 0.55, h * 0.55]]
        }
        BBack::Pod { round: _ } => superellipse(hw, h / 2.0, 4.0, usize::from(spec.facets)).into_iter().map(|[x, z]| [x, z + h / 2.0]).collect(),
        BBack::Saddle => {
            let n = usize::from(spec.facets);
            let mut pts = vec![[-hw, 0.0], [hw, 0.0]];
            pts.extend((0..=n).map(|k| {
                let a = PI * k as f32 / n as f32;
                [hw * a.cos(), h * a.sin()]
            }));
            pts
        }
        BBack::Blade => vec![[-hw, 0.0], [hw, 0.0], [hw * 0.8, h], [-hw * 0.8, h]],
        BBack::Shell { .. } | BBack::None => Vec::new(),
    }
}

/// A superellipse of half-extents `a` × `b`, power `n`, in `points` from
/// the bottom, counter-clockwise.
pub fn superellipse(a: f32, b: f32, n: f32, points: usize) -> Vec<[f32; 2]> {
    egg(a, b, n, 1.0, points)
}

/// The phiegg: a superellipse whose upper half narrows to `top` of its
/// width (the fairing's taper).
pub fn egg(a: f32, b: f32, n: f32, top: f32, points: usize) -> Vec<[f32; 2]> {
    (0..points)
        .map(|k| {
            let t = TAU * k as f32 / points as f32 - PI / 2.0;
            let (c, s) = (t.cos(), t.sin());
            let up = s.signum() * s.abs().powf(2.0 / n);
            let taper = 1.0 - (1.0 - top) * up.max(0.0).powf(1.3);
            [c.signum() * c.abs().powf(2.0 / n) * a * taper, up * b]
        })
        .collect()
}

/// A bucket back: an arc of `wrap` degrees in plan, `width` across its
/// chord, standing `height` tall, 3 cm thick.
fn shell(width: f32, height: f32, wrap: f32) -> BMesh {
    let half = wrap.to_radians() / 2.0;
    let r = width / 2.0 / half.sin();
    let mut m = BMesh::default();
    let steps = 12;
    for side in [r, r - 0.03] {
        for k in 0..steps {
            let (a, b) = (-half + 2.0 * half * k as f32 / steps as f32, -half + 2.0 * half * (k + 1) as f32 / steps as f32);
            // Centred so the arc's middle is at y = 0, bulging toward +y.
            let p = |a: f32, z: f32| Vec3::new(side * a.sin(), side * a.cos() - r, z);
            if side == r {
                m.quad(p(a, 0.0), p(b, 0.0), p(b, height), p(a, height));
            } else {
                m.quad(p(b, 0.0), p(a, 0.0), p(a, height), p(b, height));
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::venue::seat::BSeat;

    #[test]
    fn every_seat_fits_its_pitch_and_its_back_is_its_own() {
        let mut outlines = Vec::new();
        for seat_kind in BSeat::ALL {
            let spec = seat_kind.spec();
            let m = seat(spec);
            assert!(m.triangle_count() > 20, "{}", spec.label);
            let (lo, hi) = m.positions.iter().fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))));
            assert!(hi.x - lo.x <= PITCH + 1e-3, "{}: {} wide", spec.label, hi.x - lo.x);
            assert!(lo.z >= -1e-4 && hi.z < 1.05, "{}: from the step to {}", spec.label, hi.z);
            if spec.back != BBack::None {
                assert!(hi.z > PAN + 0.1, "{}: a back above the pan", spec.label);
            }
            outlines.push(outline(spec));
        }
        // No two backs share an outline.
        for i in 0..outlines.len() {
            for j in i + 1..outlines.len() {
                assert!(outlines[i].is_empty() || outlines[i] != outlines[j]);
            }
        }
        // The semi-hex's top is half its foot.
        let semi = outline(BSeat::SemiHex.spec());
        let top = semi[3][0] - semi[4][0];
        assert!((top - BSeat::SemiHex.spec().width / 2.0 + 0.04).abs() < 0.01, "{top}");
    }
}
