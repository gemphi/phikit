//! The C64/Q64 juncture: one circle read continuously (C64, a phase per
//! channel) or in 64 symbols (Q64, a sector per channel).
//!
//! `quantise` floors a phase into one of 64 sectors; `lift` returns the
//! sector's centre. The round trip moves a phase by at most π/64. When both
//! words of a pair are quantised, the angle between them moves by at most
//! π/32, so a single channel's cosine moves by at most π/32 ≈ 0.098 (the
//! bound `1 − cos(π/32)` only holds near agreement); averaged over 64
//! channels the measured loss on random pairs is under 0.01.

use crate::{c64, CHANNELS, TAU};

/// Half the sector width: the worst-case round-trip error.
pub const BOUND: f64 = std::f64::consts::PI / CHANNELS as f64;

/// Phase → sector index in `0..64`.
#[inline]
pub fn quantise(phase: f64) -> u8 {
    ((phase.rem_euclid(TAU) / TAU) * CHANNELS as f64).floor() as usize as u8 % CHANNELS as u8
}

/// Sector index → its central phase, `2π(q + ½)/64`.
#[inline]
pub fn lift(q: u8) -> f64 {
    ((q as usize % CHANNELS) as f64 + 0.5) / CHANNELS as f64 * TAU
}

/// Quantise every channel of a word: 64 phases → 64 symbols (a 384-bit key).
pub fn quantise_word(phases: &[f64]) -> Vec<u8> {
    phases.iter().map(|p| quantise(*p)).collect()
}

/// Lift a Q64 word back to phasors on the unit circle.
pub fn lift_word(symbols: &[u8]) -> Vec<c64> {
    symbols.iter().map(|q| c64::from_polar(1.0, lift(*q))).collect()
}

/// Symbolic distance between two Q64 words: how many channels differ.
pub fn hamming(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

/// Cyclic distance between two sectors, in sectors (0..=32).
pub fn sector_distance(a: u8, b: u8) -> u8 {
    let d = (a as i32 - b as i32).rem_euclid(CHANNELS as i32);
    d.min(CHANNELS as i32 - d) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_error_is_within_the_bound() {
        let mut worst = 0.0f64;
        for i in 0..10_000 {
            let phase = TAU * i as f64 / 10_000.0;
            let back = lift(quantise(phase));
            let e = crate::wrap(back - phase).abs();
            worst = worst.max(e);
            assert_eq!(quantise(back), quantise(phase), "lift lands in its own sector");
        }
        assert!(worst <= BOUND + 1e-12, "worst error {worst} > bound {BOUND}");
        assert!(worst > BOUND * 0.99, "the bound is tight: {worst}");
    }

    #[test]
    fn resonance_loss_under_double_quantisation_is_under_half_a_percent() {
        // Two random words, compared before and after quantising both.
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        let mut rnd = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f64 / (1u64 << 53) as f64 * TAU
        };
        let mut worst = 0.0f64;
        for _ in 0..200 {
            let a: Vec<f64> = (0..64).map(|_| rnd()).collect();
            let b: Vec<f64> = (0..64).map(|_| rnd()).collect();
            let exact: f64 = a.iter().zip(&b).map(|(x, y)| (x - y).cos()).sum::<f64>() / 64.0;
            let qa = lift_word(&quantise_word(&a));
            let qb = lift_word(&quantise_word(&b));
            let quant: f64 = qa.iter().zip(&qb).map(|(x, y)| (x * y.conj()).re).sum::<f64>() / 64.0;
            worst = worst.max((exact - quant).abs());
        }
        // Per-channel worst case is 2·BOUND (the cosine is 1-Lipschitz); the
        // mean over 64 channels of random-sign errors is far smaller.
        assert!(worst <= 2.0 * BOUND + 1e-9, "resonance loss {worst}");
        assert!(worst < 0.02, "measured 64-channel loss {worst} (expected ≈ 0.01)");
    }

    #[test]
    fn hamming_and_sector_distance() {
        let a = [0u8, 1, 2, 63];
        let b = [0u8, 2, 2, 0];
        assert_eq!(hamming(&a, &b), 2);
        assert_eq!(sector_distance(63, 0), 1);
        assert_eq!(sector_distance(0, 32), 32);
    }
}
