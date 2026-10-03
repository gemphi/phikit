//! Möbius maps `z ↦ (az + b)/(cz + d)`: the conformal symmetry group of the
//! Riemann sphere, and the exact algebra of one channel.
//!
//! Rotation is bind, its inverse is unbind, dilation is an evidence change,
//! inversion swaps the two "no relation" poles. They compose as 2×2 complex
//! matrices, so a chain of role rotations is one matrix product.

use crate::c64;

/// A Möbius map as its matrix `[[a, b], [c, d]]`, `ad − bc ≠ 0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mobius {
    pub a: c64,
    pub b: c64,
    pub c: c64,
    pub d: c64,
}

impl Mobius {
    pub const fn new(a: c64, b: c64, c: c64, d: c64) -> Self {
        Self { a, b, c, d }
    }

    /// `z ↦ z`.
    pub fn identity() -> Self {
        Self::new(c64::new(1.0, 0.0), c64::new(0.0, 0.0), c64::new(0.0, 0.0), c64::new(1.0, 0.0))
    }

    /// `z ↦ e^{iθ} z` — bind by a role angle.
    pub fn rotation(theta: f64) -> Self {
        Self::new(c64::from_polar(1.0, theta), c64::new(0.0, 0.0), c64::new(0.0, 0.0), c64::new(1.0, 0.0))
    }

    /// `z ↦ ρ z` — scale evidence.
    pub fn dilation(rho: f64) -> Self {
        Self::new(c64::new(rho, 0.0), c64::new(0.0, 0.0), c64::new(0.0, 0.0), c64::new(1.0, 0.0))
    }

    /// `z ↦ 1/z` — swap the poles, reflect the circle.
    pub fn inversion() -> Self {
        Self::new(c64::new(0.0, 0.0), c64::new(1.0, 0.0), c64::new(1.0, 0.0), c64::new(0.0, 0.0))
    }

    /// `z ↦ z + t`.
    pub fn translation(t: c64) -> Self {
        Self::new(c64::new(1.0, 0.0), t, c64::new(0.0, 0.0), c64::new(1.0, 0.0))
    }

    pub fn det(self) -> c64 {
        self.a * self.d - self.b * self.c
    }

    /// Apply to a point. `None` when `z` is the pole sent to ∞.
    pub fn apply(self, z: c64) -> Option<c64> {
        let den = self.c * z + self.d;
        if den.norm() < 1e-300 {
            return None;
        }
        Some((self.a * z + self.b) / den)
    }

    /// `self ∘ other`: apply `other` first, then `self`. Matrix product.
    pub fn then_after(self, other: Mobius) -> Self {
        Self::new(
            self.a * other.a + self.b * other.c,
            self.a * other.b + self.b * other.d,
            self.c * other.a + self.d * other.c,
            self.c * other.b + self.d * other.d,
        )
    }

    /// The inverse map.
    pub fn inverse(self) -> Self {
        Self::new(self.d, -self.b, -self.c, self.a)
    }

    /// Normalise so `det = 1` (the map is unchanged).
    pub fn normalised(self) -> Self {
        let s = self.det().sqrt();
        Self::new(self.a / s, self.b / s, self.c / s, self.d / s)
    }

    /// Derivative `dw/dz = (ad − bc)/(cz + d)²`.
    pub fn derivative(self, z: c64) -> c64 {
        let den = self.c * z + self.d;
        self.det() / (den * den)
    }

    /// Apply to every sample of a function on the circle.
    pub fn apply_all(self, f: &[c64]) -> Vec<c64> {
        f.iter().map(|z| self.apply(*z).unwrap_or(c64::new(f64::INFINITY, 0.0))).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_is_bind_and_its_inverse_is_unbind() {
        let z = c64::from_polar(1.0, 0.4);
        let r = Mobius::rotation(1.1);
        let bound = r.apply(z).unwrap();
        assert!((bound - c64::from_polar(1.0, 1.5)).norm() < 1e-12);
        let back = r.inverse().apply(bound).unwrap();
        assert!((back - z).norm() < 1e-12);
    }

    #[test]
    fn composition_is_matrix_product() {
        let m = Mobius::new(c64::new(0.9, 0.3), c64::new(0.2, -0.1), c64::new(0.1, 0.2), c64::new(1.0, 0.0));
        let n = Mobius::rotation(0.7).then_after(Mobius::dilation(1.5));
        let z = c64::new(0.3, -0.8);
        let direct = m.apply(n.apply(z).unwrap()).unwrap();
        let composed = m.then_after(n).apply(z).unwrap();
        assert!((direct - composed).norm() < 1e-12);
        let id = m.then_after(m.inverse()).normalised();
        assert!((id.a - c64::new(1.0, 0.0)).norm() < 1e-9 || (id.a + c64::new(1.0, 0.0)).norm() < 1e-9);
        assert!(id.b.norm() < 1e-9 && id.c.norm() < 1e-9);
    }

    #[test]
    fn inversion_swaps_the_poles_and_fixes_the_circle() {
        let inv = Mobius::inversion();
        assert!(inv.apply(c64::new(0.0, 0.0)).is_none(), "0 → ∞");
        let z = c64::from_polar(1.0, 0.9);
        let w = inv.apply(z).unwrap();
        assert!((w.norm() - 1.0).abs() < 1e-12, "circle stays on the circle");
        assert!((w - z.conj()).norm() < 1e-12, "as its reflection");
    }

    #[test]
    fn derivative_matches_finite_difference() {
        let m = Mobius::new(c64::new(0.9, 0.3), c64::new(0.2, -0.1), c64::new(0.1, 0.2), c64::new(1.0, 0.0));
        let z = c64::new(0.3, 0.5);
        let h = 1e-6;
        let fd = (m.apply(z + h).unwrap() - m.apply(z).unwrap()) / h;
        assert!((fd - m.derivative(z)).norm() < 1e-5);
    }
}
