//! The form a shoe is lofted round, read off the user's sketch sheets
//! (`sketch`): a drawing's side view gives how far back and forward it
//! reaches at every height (`BOutline` — the tall boot's for every boot,
//! the slip-on's and the ballet flat's own for them), the tall boot's front
//! view how wide,
//! its sole how the foot spreads from heel to toe. Sliced every few
//! millimetres, each slice is kept as its middle and its reach in every
//! direction round it, so a shoe's surface can be asked for anywhere; every
//! boot is the tall boot cut off at its own rim (the sheet's "cut-off
//! series").
//!
//! Directions are measured from the toe: `θ = 0` points down the foot, `π/2`
//! out to the side (the left foot's outside), `π` to the heel. The left foot
//! stands at `x = FOOT_X`, its heel's back at `y = HEEL_Y`, its toe toward
//! −y, as the players' figure stands.

use super::sketch::{BOutline, FRONT, SAMPLES, TALL, TREAD};
use glam::{Vec2, Vec3};
use std::f32::consts::TAU;

/// The foot's length, metres: a women's 39.
pub const LENGTH: f32 = 0.25;
/// How thick the sketch draws its sole, foot lengths: the form starts above
/// it, the shoe's own sole (`sole`) under it.
pub const SOLE: f32 = 0.1;
/// The slices' spacing.
pub const DZ: f32 = 0.004;
/// The lowest slice: the top of the sole, where the upper begins.
pub const FLOOR: f32 = 0.003;
/// Where the left foot stands.
const FOOT_X: f32 = 0.11;
const HEEL_Y: f32 = 0.06;
/// Directions round each slice.
const BINS: usize = 96;
/// Points along each slice's outline.
const OUTLINE: usize = 160;

fn table<T: Copy>(t: &[T; SAMPLES], x: f32, mix: impl Fn(T, T, f32) -> T) -> T {
    let k = (x.clamp(0.0, 1.0) * (SAMPLES - 1) as f32).min((SAMPLES - 1) as f32 - 1e-4);
    let i = k.floor() as usize;
    mix(t[i], t[(i + 1).min(SAMPLES - 1)], k - i as f32)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub(super) fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The sole's half width and its middle's offset at `u` foot lengths from
/// the heel, metres.
pub fn tread(u: f32) -> (f32, f32) {
    let (w, c) = table(&TREAD, u, |a, b, t| (lerp(a.0, b.0, t), lerp(a.1, b.1, t)));
    (w * LENGTH, c * LENGTH)
}

pub struct BForm {
    /// The drawing it was lofted from.
    outline: &'static BOutline,
    centres: Vec<Vec2>,
    radii: Vec<[f32; BINS]>,
    /// Down the foot, and out to its side, in the floor's plane.
    pub fwd: Vec2,
    pub side: Vec2,
    /// The ankle bone's height.
    pub ankle: f32,
    /// Where the heel's back stands.
    pub heel: Vec2,
}

impl BForm {
    /// The drawing's shoe, sliced up to `top` metres: its side view from
    /// `outline`, its width from the tall boot's front and sole views.
    pub fn sketch(outline: &'static BOutline, top: f32) -> BForm {
        let (side_view, height) = (&outline.side, outline.height);
        let (fwd, side) = (Vec2::new(0.0, -1.0), Vec2::new(1.0, 0.0));
        let heel = Vec2::new(FOOT_X, HEEL_Y);
        // How high the boot stands over each point down the foot, foot
        // lengths: the side view's top edge there.
        let crown: Vec<f32> = (0..=64)
            .map(|k| {
                let u = k as f32 / 64.0;
                (0..SAMPLES).rev().find(|&i| side_view[i].0 <= u && u <= side_view[i].1).map_or(0.0, |i| height * i as f32 / (SAMPLES - 1) as f32)
            })
            .collect();
        let crown_at = |u: f32| {
            let x = (u.clamp(0.0, 1.0) * 64.0).min(63.999);
            let i = x.floor() as usize;
            lerp(crown[i], crown[i + 1], x - i as f32)
        };
        let levels = ((top + 4.0 * DZ - FLOOR) / DZ).ceil().max(2.0) as usize;
        let mut centres = Vec::with_capacity(levels);
        let mut radii = Vec::with_capacity(levels);
        for i in 0..levels {
            let z = FLOOR + i as f32 * DZ;
            let h = z / LENGTH + SOLE;
            // Above a low shoe's drawing stands the tall boot's leg: the
            // ankle its ribbons wind round.
            let (view, over) = if h > height { (&TALL.side, TALL.height) } else { (side_view, height) };
            let (back, front) = table(view, (h / over).min(1.0), |a, b, t| (lerp(a.0, b.0, t), lerp(a.1, b.1, t)));
            let wide = table(&FRONT, (h / TALL.height).min(1.0), lerp) * LENGTH;
            // Low down the slice is the foot, its width the sole's from heel
            // to toe; up the leg it is an ellipse the front view's width.
            let foot = 1.0 - smoothstep(0.06, 0.14, z);
            let mut pts = Vec::with_capacity(OUTLINE * 2);
            for k in 0..=OUTLINE {
                let s = k as f32 / OUTLINE as f32;
                let t = (1.0 - (std::f32::consts::PI * s).cos()) / 2.0;
                let u = lerp(back, front, t);
                let round = (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0).sqrt();
                let (tw, tc) = tread(u);
                // Over the foot the section is an arch, not a slab: full
                // width low down, rounding in to nothing at the boot's top
                // over that point — the toe box and the instep round over
                // as the drawings do.
                let rise = ((z / LENGTH) / (crown_at(u) - SOLE).max(0.02)).clamp(0.0, 1.0);
                let arch = (1.0 - rise.powf(2.2)).max(0.0).powf(0.5);
                let w = lerp(round * wide, tw.min(wide) * round.powf(0.35) * arch, foot);
                let c = tc * foot;
                let along = heel + fwd * (u * LENGTH);
                pts.push(along + side * (c + w));
                pts.push(along + side * (c - w));
            }
            let (lo, hi) = pts.iter().fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            let centre = (lo + hi) * 0.5;
            let mut r = [-1.0f32; BINS];
            for p in &pts {
                let v = *p - centre;
                let angle = v.dot(side).atan2(v.dot(fwd));
                let bin = (((angle / TAU) * BINS as f32).round() as isize).rem_euclid(BINS as isize) as usize;
                r[bin] = r[bin].max(v.length());
            }
            fill(&mut r);
            centres.push(centre);
            radii.push(r);
        }
        BForm { outline, centres, radii, fwd, side, ankle: 0.085, heel }
    }

    /// The drawing's top, metres off the top of the sole: nothing of the
    /// shoe is cut above it.
    pub fn top(&self) -> f32 {
        (self.outline.height - SOLE) * LENGTH
    }

    /// How far back and forward the drawing reaches `h` foot lengths off its
    /// floor — below `SOLE`, the sole rounding in at the heel and the toe.
    pub fn ends(&self, h: f32) -> (f32, f32) {
        table(&self.outline.side, h / self.outline.height, |a, b, t| (lerp(a.0, b.0, t), lerp(a.1, b.1, t)))
    }

    /// How high the drawing's underside is off the floor `u` foot lengths
    /// from the heel, metres.
    pub fn underside(&self, u: f32) -> f32 {
        table(&self.outline.underside, u, lerp) * LENGTH
    }

    fn level(&self, z: f32) -> (usize, usize, f32) {
        let x = ((z - FLOOR) / DZ).clamp(0.0, (self.centres.len() - 1) as f32);
        let i = x.floor() as usize;
        let j = (i + 1).min(self.centres.len() - 1);
        (i, j, x - i as f32)
    }

    pub fn centre(&self, z: f32) -> Vec2 {
        let (i, j, t) = self.level(z);
        self.centres[i].lerp(self.centres[j], t)
    }

    /// How far the form reaches from its centre at height `z` toward `θ`.
    pub fn radius(&self, z: f32, theta: f32) -> f32 {
        let (i, j, t) = self.level(z);
        let x = (theta / TAU).rem_euclid(1.0) * BINS as f32;
        let (a, s) = (x.floor() as usize % BINS, x.fract());
        let b = (a + 1) % BINS;
        let at = |r: &[f32; BINS]| r[a] * (1.0 - s) + r[b] * s;
        at(&self.radii[i]) * (1.0 - t) + at(&self.radii[j]) * t
    }

    /// The direction `θ` in the floor's plane.
    pub fn dir(&self, theta: f32) -> Vec2 {
        self.fwd * theta.cos() + self.side * theta.sin()
    }

    /// How far down the foot a point is, in foot lengths: 0 at the back of
    /// the heel, 1 at the toe's tip.
    pub fn along(&self, p: Vec3) -> f32 {
        (p.truncate() - self.heel).dot(self.fwd) / LENGTH
    }
}

/// Empty directions filled from their nearest filled neighbours either way.
fn fill(r: &mut [f32; BINS]) {
    let filled: Vec<usize> = (0..BINS).filter(|&b| r[b] >= 0.0).collect();
    if filled.is_empty() {
        r.fill(0.05);
        return;
    }
    let copy = *r;
    for b in 0..BINS {
        if copy[b] >= 0.0 {
            continue;
        }
        let next = filled.iter().map(|&f| (f + BINS - b) % BINS).min().unwrap_or(0);
        let prev = filled.iter().map(|&f| (b + BINS - f) % BINS).min().unwrap_or(0);
        let (a, c) = (copy[(b + BINS - prev) % BINS], copy[(b + next) % BINS]);
        let t = prev as f32 / (prev + next).max(1) as f32;
        r[b] = a * (1.0 - t) + c * t;
    }
}
