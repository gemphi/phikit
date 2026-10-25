//! A design (`kit::footwear::BFootwearSpec`) made into a pair, step by
//! step — the order the docs walk through:
//!
//! 1. **The form.** The user's tall boot read off her sketch sheet — side,
//!    front and sole — sliced from the floor up (`form::BForm`).
//! 2. **The cut.** The form cut off at the design's rim (`Last::top`), the
//!    toe drawn out as its toe says (`BToe::stretch`).
//! 3. **The upper and the lining.** The form's surface is the upper; the
//!    lining a leather's thickness inside; the two joined over the rim. A
//!    quilted upper is puffed in softened hexagons (`quilt`).
//! 4. **The panels.** The foxing round the foot under the sketch's vamp line,
//!    a toe cap over the toe — each laid over the upper.
//! 5. **The sole.** The form's footprint widened by the welt, its wall
//!    cupping up the back of the heel and over the toe as the drawings show,
//!    the tread's lines round it and its pattern on the ground (`BTread`:
//!    waves, hexagons, rings or ripples — what tells a set's four designs
//!    apart), the arch lifted off the floor (`BForm::underside`).
//! 6. **The heel.** Everything raised by the heel's height behind the arch
//!    (`Last::lift`), and the heel built under it (`sole`).
//! 7. **The trim** (`trim`): the sketch's lines and scalloped bands — the
//!    scallops one shape repeated along the band — the scalloped rim, the back
//!    zip and its teeth; the extras: ribbons and their bow, a thong, an ankle
//!    strap — each lying on the form, so nothing floats.
//! 8. **The pair.** Whatever stands lowest stood on the floor; the left
//!    shoe mirrored for the right.
//!
//! Every piece is a `BPart` with its own material; the left shoe stands
//! where the players' left foot does.

mod form;
pub mod picture;
mod sketch;
mod sole;
mod sweep;
mod trim;

use super::mesh::BMesh;
use super::part::{BMat, BPart};
use crate::kit::footwear::{BFootwearSpec, BReach, BTopline};
use form::{BForm, FLOOR, LENGTH, SOLE, smoothstep};
use glam::{Mat4, Vec3};
use std::f32::consts::TAU;
use sweep::{grid, mirrored, sheet};

/// Directions round the shoe.
const ROUND: usize = 112;
/// The leather between the upper's face and the lining's.
const LEATHER: f32 = 0.0035;

/// The rows' spacing: tight round the foot, looser up the shaft.
fn warp(z: f32) -> f32 {
    if z < 0.12 { z / 0.004 } else { 30.0 + (z - 0.12) / 0.006 }
}

fn unwarp(u: f32) -> f32 {
    if u < 30.0 { u * 0.004 } else { 0.12 + (u - 30.0) * 0.006 }
}

/// How far toward the front a direction faces: 0 at the heel, 1 at the toe.
fn frontness(theta: f32) -> f32 {
    (1.0 + theta.cos()) / 2.0
}

pub(crate) struct Last<'a> {
    pub form: BForm,
    pub spec: &'a BFootwearSpec,
    /// A low shoe's rim at `ROUND` directions, metres (`Last::cut`).
    cuts: Vec<f32>,
}

/// A low shoe's rim line down the foot, foot lengths off the sketch's floor
/// at `u` foot lengths from the heel: the heel counter's height at the back,
/// the collar's at the side of the foot, the throat's over the instep —
/// the drawings' line, which runs along the foot, not round the ankle.
fn rim_line(u: f32, front: f32, side: f32, back: f32) -> f32 {
    if u < 0.3 {
        back + (side - back) * smoothstep(0.0, 0.3, u)
    } else {
        side + (front - side) * smoothstep(0.3, 0.55, u)
    }
}

impl<'a> Last<'a> {
    fn new(form: BForm, spec: &'a BFootwearSpec) -> Self {
        let mut last = Last { form, spec, cuts: Vec::new() };
        if let BReach::Low { front, side, back } = spec.reach {
            last.cuts = (0..ROUND).map(|j| last.cut(TAU * j as f32 / ROUND as f32, front, side, back)).collect();
        }
        last
    }

    /// Where a low shoe's rim crosses the column of the form toward `θ`:
    /// climbing it, the first height that stands over the rim line at the
    /// point of the foot the column has reached.
    fn cut(&self, theta: f32, front: f32, side: f32, back: f32) -> f32 {
        let over = |z: f32| z - LENGTH * (rim_line(self.form.along(self.at(z, theta, 0.0)), front, side, back) - SOLE);
        let (mut lo, mut z) = (FLOOR, FLOOR + 0.002);
        while z < 0.2 && over(z) < 0.0 {
            lo = z;
            z += 0.002;
        }
        let (a, b) = (over(lo), over(z));
        let cut = if b > a { lo + (z - lo) * (-a / (b - a)).clamp(0.0, 1.0) } else { z };
        cut.min(self.form.top())
    }

    /// The rim's height toward `θ`, metres.
    pub fn top(&self, theta: f32) -> f32 {
        let base = match self.spec.reach {
            BReach::Rim { back, front } => back + (front - back) * frontness(theta).powf(1.3) - SOLE,
            BReach::Low { .. } => {
                let x = theta.rem_euclid(TAU) / TAU * ROUND as f32;
                let i = x.floor() as usize % ROUND;
                let cut = self.cuts[i] + (self.cuts[(i + 1) % ROUND] - self.cuts[i]) * x.fract();
                cut / LENGTH
            }
        };
        let c = theta.cos();
        LENGTH
            * (base
                + match self.spec.topline {
                    BTopline::Straight => 0.0,
                    BTopline::Peak(p) => p * c.max(0.0).powi(2),
                    BTopline::Notch(n) => -n * c.abs().powi(6),
                    BTopline::Wave(w) => -w * (0.5 + 0.5 * (6.0 * theta).cos()),
                })
    }

    pub fn lowest_top(&self) -> f32 {
        (0..ROUND).map(|j| self.top(TAU * j as f32 / ROUND as f32)).fold(f32::MAX, f32::min)
    }

    /// A line round the shaft from `back` to `front` (foot lengths off the
    /// sketch's floor), metres off the top of the sole.
    pub fn band(&self, back: f32, front: f32, theta: f32) -> f32 {
        LENGTH * (back - SOLE + (front - back) * frontness(theta))
    }

    /// The form's reach toward `θ` at `z`, the toe drawn out.
    fn reach(&self, z: f32, theta: f32) -> f32 {
        let front = theta.cos().max(0.0).powi(6);
        let toe = 1.0 + self.spec.toe.stretch() * front * (1.0 - smoothstep(0.025, 0.075, z));
        self.form.radius(z, theta) * toe
    }

    /// The point `off` outside the form toward `θ` at `z`, before the heel
    /// raises it.
    pub fn at(&self, z: f32, theta: f32, off: f32) -> Vec3 {
        (self.form.centre(z) + self.form.dir(theta) * (self.reach(z, theta) + off)).extend(z)
    }

    /// Up the surface at a point.
    pub fn up(&self, z: f32, theta: f32, off: f32) -> Vec3 {
        (self.at(z + 0.003, theta, off) - self.at(z - 0.003, theta, off)).normalize_or_zero()
    }

    /// Round the surface at a point, the way `θ` grows.
    pub fn round(&self, z: f32, theta: f32, off: f32) -> Vec3 {
        (self.at(z, theta + 0.02, off) - self.at(z, theta - 0.02, off)).normalize_or_zero()
    }

    /// Out of the surface at a point.
    pub fn out(&self, z: f32, theta: f32, off: f32) -> Vec3 {
        self.round(z, theta, off).cross(self.up(z, theta, off)).normalize_or_zero()
    }

    /// The heel's rise at a point: all of it behind the arch and up the leg,
    /// none from the ball of the foot forward.
    pub fn lift(&self, p: Vec3) -> Vec3 {
        let h = self.spec.heel.height();
        let behind = 1.0 - smoothstep(0.32, 0.68, self.form.along(p));
        let up = smoothstep(0.05, 0.14, p.z);
        p + Vec3::Z * h * behind.max(up)
    }

    /// The softened hexagon quilting's puff at a point of the shaft: the
    /// cells' middles stand out, their seams sink, nothing at the foxing or
    /// within a few centimetres of the rim.
    fn quilt(&self, z: f32, theta: f32) -> f32 {
        if !self.spec.quilt {
            return 0.0;
        }
        const CELL: f32 = 0.03;
        let (s, t) = (theta * 0.055, z);
        // The nearest of the lattice's centres, from the three about the point.
        let (q, r) = ((3f32.sqrt() / 3.0 * s - t / 3.0) / CELL, (2.0 / 3.0 * t) / CELL);
        let (mut best, x0, y0) = (f32::MAX, q.floor() as i32, r.floor() as i32);
        for (dq, dr) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let (cq, cr) = ((x0 + dq) as f32, (y0 + dr) as f32);
            let cs = CELL * 3f32.sqrt() * (cq + cr / 2.0);
            let ct = CELL * 1.5 * cr;
            best = best.min(((s - cs).powi(2) + (t - ct).powi(2)).sqrt());
        }
        let edge = (best / (CELL * 0.9)).min(1.0);
        let fade = smoothstep(self.fox(theta) + 0.01, self.fox(theta) + 0.03, z) * (1.0 - smoothstep(self.top(theta) - 0.05, self.top(theta) - 0.025, z));
        0.0042 * (edge * std::f32::consts::FRAC_PI_2).cos().max(0.0).powf(1.5) * fade
    }

    /// The sketch's vamp line: the top of the foxing round the foot.
    pub fn fox(&self, theta: f32) -> f32 {
        self.band(0.3, 0.13, theta)
    }

    /// Rows round the form from the floor to the rim, `off` out, the
    /// quilting on the face when `face`.
    fn rows(&self, off: f32, face: bool) -> Vec<Vec<Vec3>> {
        let thetas: Vec<f32> = (0..ROUND).map(|j| TAU * j as f32 / ROUND as f32).collect();
        let tops: Vec<f32> = thetas.iter().map(|&t| warp(self.top(t))).collect();
        let m = tops.iter().fold(0.0f32, |a, &b| a.max(b)).ceil() as usize;
        let lo = warp(FLOOR);
        (0..=m)
            .map(|k| {
                thetas
                    .iter()
                    .zip(&tops)
                    .map(|(&t, &u)| {
                        let z = unwarp(lo + (u - lo) * k as f32 / m as f32);
                        self.at(z, t, off + if face { self.quilt(z, t) } else { 0.0 })
                    })
                    .collect()
            })
            .collect()
    }
}

/// One part of a shoe before it is placed.
pub(crate) type BPiece = (String, BMesh, BMat);

/// The parts of one design's left shoe.
fn left(spec: &BFootwearSpec) -> Vec<BPiece> {
    // The slip-on and the ballet flat are drawn on lasts of their own; every
    // other design is the tall boot cut off.
    let outline = match spec.set {
        crate::kit::footwear::BFootwearSet::SlipOn => &sketch::SLIP,
        crate::kit::footwear::BFootwearSet::Ballet => &sketch::FLAT,
        _ => &sketch::TALL,
    };
    let probe = Last::new(BForm::sketch(outline, 0.12), spec);
    let reach_top = (0..ROUND).map(|j| probe.top(TAU * j as f32 / ROUND as f32)).fold(0.0f32, f32::max).max(0.26);
    let last = Last::new(BForm::sketch(outline, reach_top + 0.02), spec);
    let roughness = if spec.satin { 0.24 } else { 0.42 };
    let colours = spec.colours;
    // The sole as thick as the drawings draw it (`SOLE`), so a shoe stands
    // as tall as drawn; an open sandal's footbed and a first-batch satin
    // slipper's leather sole are their own.
    let sole_t = if spec.open {
        0.018
    } else if spec.satin && spec.set.collection() == crate::kit::footwear::BCollection::FirstBatch {
        0.005
    } else {
        SOLE * LENGTH
    };
    let raise = |p: Vec3| last.lift(p) + Vec3::Z * (sole_t - FLOOR);
    let mut out: Vec<BPiece> = Vec::new();
    let mut put = |name: &str, mesh: BMesh, mat: BMat, raised: bool| {
        if !mesh.indices.is_empty() {
            out.push((name.to_string(), if raised { moved(mesh, &raise) } else { mesh }, mat));
        }
    };

    // 3. The upper, the lining, the rim.
    if !spec.open {
        let outer = last.rows(0.0, true);
        let inner = last.rows(-LEATHER, false);
        put("Upper", grid(&outer, true), BMat::plain("Upper", colours.upper, roughness), true);
        put("Lining", grid(&inner, false), BMat::plain("Lining", colours.lining, 0.75), true);
        if let (Some(i), Some(o)) = (inner.last(), outer.last()) {
            put("Rim", grid(&[i.clone(), o.clone()], false), BMat::plain("Rim", colours.upper, roughness), true);
        }
        // 4. The foxing round the foot, the toe cap over the toe.
        let panel = BMat::plain("Panel", colours.panel, roughness);
        if spec.foxing {
            let rows: Vec<Vec<Vec3>> = (0..=16).map(|k| (0..ROUND).map(|j| { let t = TAU * j as f32 / ROUND as f32; last.at(FLOOR + (last.fox(t) - FLOOR) * k as f32 / 16.0, t, 0.0008) }).collect()).collect();
            put("Foxing", grid(&rows, true), panel.clone(), true);
        }
        if spec.cap {
            let (span, cols, n) = (1.35f32, 41usize, 14usize);
            let edge = |t: f32| 0.016 + 0.034 * (t / span * std::f32::consts::FRAC_PI_2).cos().max(0.0).powf(0.8);
            let rows: Vec<Vec<Vec3>> = (0..=n).map(|k| (0..cols).map(|c| { let t = -span + 2.0 * span * c as f32 / (cols - 1) as f32; last.at(FLOOR + (edge(t) - FLOOR) * k as f32 / n as f32, t, 0.0011) }).collect()).collect();
            put("Toe Cap", sheet(&rows, true, false), panel, true);
        }
    }

    // 5–6. The sole and the heel.
    for (name, mesh, mat) in sole::sole(&last, sole_t) {
        put(&name, mesh, mat, name != "Heel");
    }

    // 7. The trim and the extra.
    for (name, mesh, mat) in trim::trim(&last) {
        put(&name, mesh, mat, true);
    }

    // Whatever stands lowest — a hexagon pad, a heel's tip — stands on the
    // floor.
    let low = out.iter().flat_map(|(_, m, _)| m.positions.iter()).map(|p| p[2]).fold(f32::MAX, f32::min);
    if low.is_finite() {
        for (_, m, _) in &mut out {
            *m = moved(std::mem::take(m), &|p| p - Vec3::Z * low);
        }
    }

    out
}

/// `m` with every point moved by `f`, its normals left as they were (the
/// moves here bend a shoe by a few degrees at most, or turn it whole).
pub(crate) fn moved(mut m: BMesh, f: &dyn Fn(Vec3) -> Vec3) -> BMesh {
    let turn = f(Vec3::X) - f(Vec3::ZERO);
    let rigid = (turn - Vec3::X).length() > 1e-3 && (turn.length() - 1.0).abs() < 1e-3;
    for p in &mut m.positions {
        *p = f(Vec3::from(*p)).into();
    }
    if rigid {
        // A whole turn turns the normals with it.
        let (o, x, y, z) = (f(Vec3::ZERO), f(Vec3::X), f(Vec3::Y), f(Vec3::Z));
        let r = glam::Mat3::from_cols(x - o, y - o, z - o);
        for n in &mut m.normals {
            *n = (r * Vec3::from(*n)).into();
        }
    }
    m
}

/// The pair, as she stands: each part of the left shoe and its mirror for
/// the right, named `<design> <side> <part>`.
pub fn pair(spec: &BFootwearSpec) -> Vec<BPart> {
    let parts = left(spec);
    let mut out = Vec::with_capacity(parts.len() * 2);
    for (side, mirror) in [("L", false), ("R", true)] {
        for (name, mesh, mat) in &parts {
            let mesh = if mirror { mirrored(mesh) } else { mesh.clone() };
            let material = BMat { name: format!("{} {}", spec.label, mat.name), ..mat.clone() };
            out.push(BPart::new(format!("{} {side} {name}", spec.label), mesh, material, Mat4::IDENTITY));
        }
    }
    out
}

#[cfg(test)]
mod tests;
