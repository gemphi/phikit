//! `Phasor` — what `Transform` is to RealityKit.
//!
//! A RealityKit entity's state in the world is a position, a rotation and a
//! scale in ℝ³ (`reality-foundation/components.rs:12`). A word's state is its
//! phase on the relation circle: [`MAX_KINDS`](crate::relation::MAX_KINDS)
//! channels, each a complex phasor with an amplitude and the count of what was
//! seen under it. Phical stores no point — the number here is never a
//! position, always an angle and how much evidence stands behind it.

use crate::aspect::Aspect;
use crate::relation::{MAX_KINDS, Relation};

/// One word's angles, per relation channel.
#[derive(Clone, Debug)]
pub struct Phasor {
    theta: Vec<f64>,
    amplitude: Vec<f64>,
    evidence: Vec<u32>,
}

impl Aspect for Phasor {}

const TWO_PI: f64 = std::f64::consts::TAU;

impl Default for Phasor {
    fn default() -> Self {
        Self::new()
    }
}

impl Phasor {
    /// All channels silent: no angle claimed, no evidence behind any.
    pub fn new() -> Self {
        Self {
            theta: vec![0.0; MAX_KINDS],
            amplitude: vec![0.0; MAX_KINDS],
            evidence: vec![0; MAX_KINDS],
        }
    }

    /// Seeded deterministically from a spelling, so a fresh word starts
    /// somewhere of its own rather than on top of every other fresh word.
    pub fn seeded(spelling: &str) -> Self {
        let mut p = Self::new();
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in spelling.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        for k in 0..MAX_KINDS {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            p.theta[k] = (h >> 11) as f64 / (1u64 << 53) as f64 * TWO_PI;
        }
        p
    }

    pub fn theta(&self, kind: Relation) -> f64 {
        self.theta.get(kind.channel()).copied().unwrap_or(0.0)
    }

    pub fn amplitude(&self, kind: Relation) -> f64 {
        self.amplitude.get(kind.channel()).copied().unwrap_or(0.0)
    }

    pub fn evidence(&self, kind: Relation) -> u32 {
        self.evidence.get(kind.channel()).copied().unwrap_or(0)
    }

    /// Move this channel a step of `rate` toward `target`, the short way
    /// round, and count the observation. This is the whole learning rule:
    /// pairs of words moving under a condition.
    pub fn couple(&mut self, kind: Relation, target: f64, rate: f64) {
        let k = kind.channel();
        if k >= MAX_KINDS {
            return;
        }
        let d = (target - self.theta[k] + std::f64::consts::PI).rem_euclid(TWO_PI) - std::f64::consts::PI;
        self.theta[k] = (self.theta[k] + rate * d).rem_euclid(TWO_PI);
        self.evidence[k] = self.evidence[k].saturating_add(1);
        self.amplitude[k] = (self.evidence[k] as f64).ln_1p();
    }

    /// How closely two words agree on `kind`: 1 together, −1 opposed. `None`
    /// when either has no evidence there — silence is not agreement.
    pub fn agreement(&self, other: &Phasor, kind: Relation) -> Option<f64> {
        let k = kind.channel();
        if self.evidence.get(k).copied().unwrap_or(0) == 0
            || other.evidence.get(k).copied().unwrap_or(0) == 0
        {
            return None;
        }
        Some((self.theta[k] - other.theta[k]).cos())
    }

    /// Agreement across every channel both words have evidence on, weighted
    /// by how much evidence that is.
    pub fn resonance(&self, other: &Phasor) -> f64 {
        let (mut sum, mut weight) = (0.0, 0.0);
        for k in 0..MAX_KINDS {
            let kind = Relation(k as u8);
            if let Some(a) = self.agreement(other, kind) {
                let w = self.amplitude[k].min(other.amplitude[k]);
                sum += a * w;
                weight += w;
            }
        }
        if weight == 0.0 { 0.0 } else { sum / weight }
    }

    /// Channels this word has evidence on, heaviest first — its relation set,
    /// which is the honest answer to RealityKit's absent `ComponentSet`.
    pub fn kinds(&self) -> Vec<(Relation, u32)> {
        let mut v: Vec<(Relation, u32)> = (0..MAX_KINDS)
            .filter(|k| self.evidence[*k] > 0)
            .map(|k| (Relation(k as u8), self.evidence[k]))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coupling_moves_together_and_counts() {
        let kind = Relation(3);
        let mut a = Phasor::seeded("cat");
        let mut b = Phasor::seeded("mat");
        assert_eq!(a.agreement(&b, kind), None, "silence is not agreement");
        for _ in 0..40 {
            let (ta, tb) = (a.theta(kind), b.theta(kind));
            a.couple(kind, tb, 0.2);
            b.couple(kind, ta, 0.2);
        }
        assert!(a.agreement(&b, kind).unwrap() > 0.99);
        assert_eq!(a.evidence(kind), 40);
        assert!(a.resonance(&b) > 0.99);
        assert_eq!(a.kinds(), vec![(kind, 40)]);
    }
}
