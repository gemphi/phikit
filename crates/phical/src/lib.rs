//! # phi-cal — exact operators on the closed relation circle
//!
//! Phiano and Phinet use an *X as Y* paradigm: a word never means anything
//! other than what it relates to, in what conditions. Each relation is an
//! angle on a circle; this crate is the exact geometry of that circle and of
//! its closed extension, the Riemann sphere. Nothing here learns. Everything
//! here is invertible or has a stated error bound, and is tested.
//!
//! | module | what |
//! |---|---|
//! | [`sphere`] | stereographic lift of the evidenced circle to S², and back; the Fibonacci lattice; geodesics |
//! | [`mobius`] | Möbius maps as 2×2 complex matrices: rotation (bind), inversion, dilation (evidence), composition, inverse |
//! | [`q64`] | the C64/Q64 juncture: quantise a phase to one of 64 symbols and lift it back, error ≤ π/64 |
//! | [`winding`] | winding number of a path of phases — a sentence's topological invariant |
//! | [`heat`] | Laplace–Beltrami heat flow on the circle (exact in Fourier modes) and band-limited on the sphere |

pub mod heat;
pub mod mobius;
pub mod q64;
pub mod sphere;
pub mod winding;

pub use num_complex::Complex64 as c64;

/// Number of relation channels, and the size of the Q64 alphabet.
pub const CHANNELS: usize = 64;
/// 2π.
pub const TAU: f64 = std::f64::consts::TAU;

/// Signed wrap of an angle difference into (−π, π].
#[inline]
pub fn wrap(d: f64) -> f64 {
    let mut d = d.rem_euclid(TAU);
    if d > std::f64::consts::PI {
        d -= TAU;
    }
    d
}
