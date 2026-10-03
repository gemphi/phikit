//! The Riemann sphere Ĉ = C ∪ {∞} ≅ S² as the closed home of one channel.
//!
//! A phasor `e^{iφ}` is on the equator; the evidence-scaled point `ρ e^{iφ}`
//! lifts to latitude by stereographic projection from the north pole, so
//! `ρ = 0` is the south pole (never seen under this relation) and `ρ → ∞`
//! the north pole. Both are "no relation"; the circle in between is where
//! a word actually stands.

use crate::c64;

/// A point on the unit sphere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct S2 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl S2 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let n = (x * x + y * y + z * z).sqrt().max(1e-300);
        Self { x: x / n, y: y / n, z: z / n }
    }

    /// Great-circle distance to `other`, in radians.
    pub fn geodesic(self, other: S2) -> f64 {
        let dot = (self.x * other.x + self.y * other.y + self.z * other.z).clamp(-1.0, 1.0);
        dot.acos()
    }

    /// Latitude in [−π/2, π/2]; the equator is 0.
    pub fn latitude(self) -> f64 {
        self.z.clamp(-1.0, 1.0).asin()
    }

    /// Longitude in [0, 2π).
    pub fn longitude(self) -> f64 {
        self.y.atan2(self.x).rem_euclid(crate::TAU)
    }
}

/// Stereographic lift from the plane to the sphere: `z ↦ (2x, 2y, |z|²−1)/(1+|z|²)`.
/// The unit circle maps to the equator, 0 to the south pole, ∞ to the north.
pub fn stereo(z: c64) -> S2 {
    let r2 = z.norm_sqr();
    let d = 1.0 + r2;
    S2 { x: 2.0 * z.re / d, y: 2.0 * z.im / d, z: (r2 - 1.0) / d }
}

/// Inverse of [`stereo`]. The north pole has no finite image and returns
/// `None`.
pub fn unstereo(p: S2) -> Option<c64> {
    let d = 1.0 - p.z;
    if d.abs() < 1e-12 {
        return None;
    }
    Some(c64::new(p.x / d, p.y / d))
}

/// Lift a phase with evidence: `ρ e^{iφ}` with `ρ = ln(1 + n)`, the reading
/// Phiano's lexicon uses. `n = 0` lands on the south pole; `n = e − 1 ≈ 1.7`
/// on the equator.
pub fn lift(phase: f64, evidence_count: u32) -> S2 {
    let rho = (1.0 + evidence_count as f64).ln();
    stereo(c64::from_polar(rho, phase))
}

/// The Fibonacci (golden-angle) lattice of `n` points on S² — the most even
/// deterministic spread available, and the grid the Q64 alphabet lives on
/// when a sector is read as a cap rather than an arc.
pub fn fibonacci_lattice(n: usize) -> Vec<S2> {
    let golden = std::f64::consts::PI * (3.0 - 5.0f64.sqrt());
    (0..n)
        .map(|i| {
            let z = 1.0 - 2.0 * (i as f64 + 0.5) / n as f64;
            let r = (1.0 - z * z).max(0.0).sqrt();
            let th = golden * i as f64;
            S2 { x: r * th.cos(), y: r * th.sin(), z }
        })
        .collect()
}

/// Index of the nearest lattice point.
pub fn nearest(lattice: &[S2], p: S2) -> usize {
    let mut best = (0usize, f64::INFINITY);
    for (i, q) in lattice.iter().enumerate() {
        let d = p.geodesic(*q);
        if d < best.1 {
            best = (i, d);
        }
    }
    best.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_round_trips_and_places_the_circle_on_the_equator() {
        for k in 0..64 {
            let z = c64::from_polar(1.0, crate::TAU * k as f64 / 64.0);
            let p = stereo(z);
            assert!(p.z.abs() < 1e-12, "unit circle is the equator");
            let back = unstereo(p).unwrap();
            assert!((back - z).norm() < 1e-12);
        }
        assert!((stereo(c64::new(0.0, 0.0)).z + 1.0).abs() < 1e-12, "0 is the south pole");
        assert!(unstereo(S2 { x: 0.0, y: 0.0, z: 1.0 }).is_none(), "∞ has no plane image");
    }

    #[test]
    fn evidence_is_latitude() {
        let none = lift(1.0, 0);
        let some = lift(1.0, 5);
        let lots = lift(1.0, 5000);
        assert!(none.latitude() < some.latitude() && some.latitude() < lots.latitude());
        assert!((none.latitude() + std::f64::consts::FRAC_PI_2).abs() < 1e-9, "never seen = south pole");
        assert!((some.longitude() - 1.0).abs() < 1e-9, "phase is longitude");
    }

    #[test]
    fn fibonacci_lattice_is_even() {
        let l = fibonacci_lattice(64);
        // Nearest-neighbour distances should all be within a factor of two.
        let mut nn = Vec::new();
        for (i, p) in l.iter().enumerate() {
            let d = l.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, q)| p.geodesic(*q)).fold(f64::INFINITY, f64::min);
            nn.push(d);
        }
        let (lo, hi) = nn.iter().fold((f64::INFINITY, 0.0f64), |(a, b), &d| (a.min(d), b.max(d)));
        assert!(hi / lo < 2.0, "lattice spacing ratio {}", hi / lo);
        assert_eq!(nearest(&l, l[17]), 17);
    }
}
