//! The sole and the heel. The sole is the form's footprint widened by the
//! welt and thickened down, its wall cupping up the back of the heel and
//! over the toe as the sketches draw it, two wavy grooves running round
//! it; underneath, the arch lifts off the floor (`BForm::underside`) and the
//! ground is met by the design's tread under the heel and across the ball, a
//! shade lighter than the sole. The tread is what tells one design of a set
//! from the next, as the drawings' heel lines do: the arcs nested round the
//! back of the heel and round the toe on the sole's wall, the pattern on
//! the ground (`BTread`) and the block heel's tiers go together. The heel goes under the raised sole: a block in tiers, a
//! stiletto, or the sole itself as a wedge.

use super::form::{FLOOR, LENGTH, SOLE, smoothstep};
use super::sweep::{cap, grid, tube};
use super::{BPiece, Last, ROUND};
use crate::build::mesh::BMesh;
use crate::build::part::BMat;
use crate::kit::footwear::{BHeel, BTread};
use glam::{Mat4, Vec2, Vec3};
use std::f32::consts::{PI, TAU};

/// How far up the upper the wall climbs toward `θ`: well up the back of the
/// heel, a little over the toe.
fn cup(theta: f32, open: bool, satin: bool) -> f32 {
    if open {
        return 0.0;
    }
    let c = theta.cos();
    // A satin shoe's sole is a thin pad: it barely climbs.
    let climb = if satin { 0.3 } else { 1.0 };
    0.003 + climb * (0.026 * (-c).max(0.0).powf(1.6) + 0.007 * c.max(0.0).powi(5))
}

/// The radii, metres, of the arcs the sole's wall carries round the back
/// of the heel and round the toe, each centred where the sole meets the
/// ground there — the drawings' heel lines, nested one inside the other.
fn arcs(tread: BTread) -> &'static [f32] {
    match tread {
        BTread::Waves => &[0.009, 0.016, 0.023],
        BTread::Hexagon => &[0.011, 0.021],
        BTread::Rings => &[0.007, 0.0125, 0.018, 0.0235],
        BTread::Ripple => &[0.008, 0.0135, 0.019, 0.0245, 0.03],
    }
}

/// The wall's stand off the upper `s` of the way up it, `u` foot lengths
/// down the foot and `rise` metres over the sole's bottom: a bulge low
/// down, cut by the arcs round the heel and the toe (the waves' arcs
/// ripple).
fn welt(tread: BTread, s: f32, u: f32, rise: f32) -> f32 {
    let groove = |d: f32| arcs(tread).iter().map(|r| if (d - r).abs() < 0.0011 { -0.0011 } else { 0.0 }).sum::<f32>();
    let (heel, toe) = ((u * LENGTH).hypot(rise), ((1.0 - u) * LENGTH).hypot(rise));
    let wave = |d: f32, a: f32| if tread == BTread::Waves { d + 0.0012 * (8.0 * a).sin() } else { d };
    let cut = groove(wave(heel, rise.atan2(u * LENGTH))) + groove(wave(toe, rise.atan2((1.0 - u) * LENGTH)));
    0.0024 + 0.0012 * (PI * s.min(0.6) / 0.6).sin() + cut.max(-0.0011)
}

/// The tread's lines on the ground — waves across the sole, rings round the
/// heel and the ball, or arcs swung from behind the heel — as polylines in
/// plan; the hexagon tread is pads, not lines.
fn ground_lines(tread: BTread, lo: Vec2, hi: Vec2, heel: Vec2, ball: Vec2, fwd: Vec2) -> Vec<Vec<Vec2>> {
    let circle = |c: Vec2, r: f32, a0: f32, a1: f32| -> Vec<Vec2> { (0..=64).map(|k| { let a = a0 + (a1 - a0) * k as f32 / 64.0; c + Vec2::new(a.cos(), a.sin()) * r }).collect() };
    match tread {
        BTread::Hexagon => Vec::new(),
        BTread::Waves => {
            let mut out = Vec::new();
            let mut y = lo.y;
            while y < hi.y {
                out.push((0..=48).map(|k| { let x = lo.x + (hi.x - lo.x) * k as f32 / 48.0; Vec2::new(x, y + 0.0022 * (TAU * x / 0.022).sin()) }).collect());
                y += 0.0072;
            }
            out
        }
        BTread::Rings => {
            let mut out: Vec<Vec<Vec2>> = (1..=5).map(|k| circle(heel, 0.0065 * k as f32, 0.0, TAU)).collect();
            out.extend((1..=8).map(|k| circle(ball, 0.0065 * k as f32, 0.0, TAU)));
            out
        }
        BTread::Ripple => {
            // Swung from just behind the heel: tight round it, opening out
            // down the sole.
            let behind = heel - fwd * 0.018;
            let a = fwd.y.atan2(fwd.x);
            (1..=36).map(|k| circle(behind, 0.0068 * k as f32, a - 1.35, a + 1.35)).collect()
        }
    }
}

pub(super) fn sole(last: &Last, sole_t: f32) -> Vec<BPiece> {
    let spec = last.spec;
    let open = spec.open;
    let foot = last.form.centre(FLOOR);
    let thetas: Vec<f32> = (0..ROUND).map(|j| TAU * j as f32 / ROUND as f32).collect();
    let print: Vec<Vec3> = thetas.iter().map(|&t| last.at(FLOOR, t, if open { 0.004 } else { 0.0 })).collect();
    let outward = |p: Vec3| (p.truncate() - foot).normalize_or_zero().extend(0.0);
    // The bottom is flat at the heel and the ball and lifted under the arch
    // (`underside`), never past the footbed.
    let arch = |u: f32| last.form.underside(u) * smoothstep(0.12, 0.22, u) * (1.0 - smoothstep(0.86, 0.96, u));
    let bottom_z = |p: Vec3| FLOOR - sole_t + arch(last.form.along(p).clamp(0.0, 1.0)).min(sole_t - 0.0025);
    // Down the wall the heel and the toe round in as the drawing's sole does
    // — a rocker, not a slab: a point of the footprint `u` along the foot
    // slides toward the middle as the drawing's outline does below its sole
    // line, from where it stands at the top of the sole to its bottom.
    let (b0, f0) = last.form.ends(SOLE);
    let (b1, f1) = last.form.ends(0.3 * SOLE);
    let rounded = |p: Vec3, s: f32| {
        let u = last.form.along(p);
        let (b, f) = (b1 + (b0 - b1) * s.sqrt(), f1 + (f0 - f1) * s.sqrt());
        let to = b + (u - b0) * (f - b) / (f0 - b0);
        p + (last.form.fwd * ((to - u) * LENGTH)).extend(0.0)
    };
    // Enough rows up the wall for its arcs to read.
    let rows: Vec<Vec<Vec3>> = (0..=40)
        .map(|k| {
            let s = k as f32 / 40.0;
            thetas
                .iter()
                .zip(&print)
                .map(|(&t, &p)| {
                    let zt = FLOOR + cup(t, open, spec.satin);
                    let zb = bottom_z(p).min(zt - 0.003);
                    let z = zb + (zt - zb) * s;
                    let base = if z <= FLOOR { rounded(p, ((z - zb) / (FLOOR - zb)).clamp(0.0, 1.0)).truncate().extend(z) } else { last.at(z, t, 0.0) };
                    base + outward(p) * welt(spec.tread, s, last.form.along(p), z - zb)
                })
                .collect()
        })
        .collect();
    let mut wall = grid(&rows, true);
    let centre = foot.extend(FLOOR - sole_t + 0.002);
    wall.merge(&cap(&rows[0], centre, -Vec3::Z), Mat4::IDENTITY);
    // The tread where the sole meets the ground — under the heel and across
    // the ball: low six-sided pads, or raised lines half sunk in the sole.
    let mut tread = BMesh::default();
    const CELL: f32 = 0.0068;
    let (dx, dy) = (CELL * 3f32.sqrt(), CELL * 1.5);
    let outline: Vec<Vec2> = rows[0].iter().map(|p| foot + (p.truncate() - foot) * 0.9).collect();
    let inside = |q: Vec2| {
        let mut odd = false;
        for i in 0..outline.len() {
            let (a, b) = (outline[i], outline[(i + 1) % outline.len()]);
            if (a.y > q.y) != (b.y > q.y) && q.x < a.x + (q.y - a.y) / (b.y - a.y) * (b.x - a.x) {
                odd = !odd;
            }
        }
        odd
    };
    let (lo, hi) = outline.iter().fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    let ground = |q: Vec2| inside(q) && !(0.3..0.56).contains(&last.form.along(q.extend(0.0)));
    let middle = |from: f32, to: f32| {
        let ps: Vec<Vec2> = outline.iter().copied().filter(|q| (from..to).contains(&last.form.along(q.extend(0.0)))).collect();
        ps.iter().copied().sum::<Vec2>() / ps.len().max(1) as f32
    };
    for line in ground_lines(spec.tread, lo, hi, middle(0.08, 0.22), middle(0.66, 0.8), last.form.fwd) {
        for run in line.split(|q| !ground(*q)).filter(|run| run.len() >= 3) {
            let path: Vec<Vec3> = run.iter().map(|q| q.extend(bottom_z(q.extend(0.0)) - 0.0004)).collect();
            tread.merge(&tube(&path, false, 8, |_| 0.0011), Mat4::IDENTITY);
        }
    }
    let mut row = 0;
    let mut y = if spec.tread == BTread::Hexagon { lo.y } else { hi.y };
    while y < hi.y {
        let mut x = lo.x + if row % 2 == 1 { dx / 2.0 } else { 0.0 };
        while x < hi.x {
            let c = Vec2::new(x, y);
            if ground(c) {
                let z = bottom_z(c.extend(0.0));
                let ring: Vec<Vec3> = (0..6).map(|k| { let a = std::f32::consts::FRAC_PI_6 + TAU * k as f32 / 6.0; (c + Vec2::new(a.cos(), a.sin()) * (CELL - 0.0011)).extend(z) }).collect();
                let low: Vec<Vec3> = ring.iter().map(|p| *p - Vec3::Z * 0.0014).collect();
                tread.merge(&grid(&[low.clone(), ring], true), Mat4::IDENTITY);
                tread.merge(&cap(&low, c.extend(z - 0.0014), -Vec3::Z), Mat4::IDENTITY);
            }
            x += dx;
        }
        y += dy;
        row += 1;
    }
    let footbed: Vec<Vec3> = print.iter().map(|p| (foot + (p.truncate() - foot) * 0.97).extend(FLOOR + 0.0006)).collect();
    let bed = if open { BMat::plain("Footbed", spec.colours.sole.mix(crate::colour::WHITE, 0.25), 0.8) } else { BMat::plain("Insole", spec.colours.lining, 0.8) };
    let sole_mat = BMat::plain("Sole", spec.colours.sole, 0.6);
    let mut out: Vec<BPiece> = vec![
        ("Sole".into(), wall, sole_mat.clone()),
        ("Tread".into(), tread, BMat::plain("Tread", spec.colours.sole.mix(crate::colour::WHITE, 0.12), 0.8)),
        ("Insole".into(), cap(&footbed, foot.extend(FLOOR + 0.0006), Vec3::Z), bed),
    ];
    if let Some(heel) = heel(last, &rows[0], sole_t) {
        out.push(("Heel".into(), heel, BMat::plain("Heel", spec.colours.sole, 0.5)));
    }
    out
}

/// The heel under the raised sole, already where it stands.
fn heel(last: &Last, bottom: &[Vec3], sole_t: f32) -> Option<BMesh> {
    let spec = last.spec;
    if spec.heel.height() <= 0.006 {
        return None;
    }
    let reach = if matches!(spec.heel, BHeel::Wedge(_)) { 0.64 } else { 0.3 };
    let raise = |p: Vec3| last.lift(p) + Vec3::Z * (sole_t - FLOOR);
    let raised: Vec<Vec3> = bottom.iter().copied().filter(|p| last.form.along(*p) < reach).map(raise).collect();
    if raised.len() < 3 {
        return None;
    }
    let middle = raised.iter().copied().sum::<Vec3>() / raised.len() as f32;
    let m = match spec.heel {
        BHeel::Stiletto(h) => {
            let c = middle.truncate() + last.form.fwd * 0.006;
            let stem: Vec<Vec3> = (0..=10).map(|k| c.extend((h + 0.004) * (1.0 - k as f32 / 10.0))).collect();
            let mut m = tube(&stem, false, 18, |s| 0.0045 + 0.0145 * (1.0 - s).powf(2.2));
            m.merge(&tube(&[c.extend(0.0), c.extend(0.007)], false, 18, |_| 0.0048), Mat4::IDENTITY);
            m
        }
        BHeel::Wedge(_) => {
            let rows: Vec<Vec<Vec3>> = (0..=6).map(|k| { let t = k as f32 / 6.0; raised.iter().map(|p| p.truncate().extend((p.z + 0.002) * (1.0 - t))).collect() }).collect();
            let mut m = grid(&rows, true);
            m.merge(&cap(&rows[6], middle.truncate().extend(0.0), -Vec3::Z), Mat4::IDENTITY);
            m
        }
        BHeel::Block(_) | BHeel::Flat(_) => {
            let tiered = matches!(spec.heel, BHeel::Block(_));
            // The block's tiers: one groove for each of the wall's arcs.
            let grooves = if tiered { arcs(spec.tread).len() } else { 0 };
            let mut marks = vec![(0.0, 0.0)];
            for g in 1..=grooves {
                let t = g as f32 / (grooves + 1) as f32;
                marks.extend([(t - 0.015, 0.0), (t, 0.0014), (t + 0.015, 0.0)]);
            }
            marks.push((1.0, 0.0));
            let rows: Vec<Vec<Vec3>> = marks.iter().map(|&(t, inset)| {
                let narrow = if tiered { 1.0 - 0.12 * t } else { 1.0 };
                raised.iter().map(|p| {
                    let d = (p.truncate() - middle.truncate()) * narrow;
                    let len = d.length();
                    (middle.truncate() + d * ((len - inset) / len.max(1e-5))).extend((p.z + 0.002) * (1.0 - t))
                }).collect()
            }).collect();
            let mut m = grid(&rows, true);
            m.merge(&cap(rows.last()?, middle.truncate().extend(0.0), -Vec3::Z), Mat4::IDENTITY);
            m
        }
    };
    Some(m)
}
