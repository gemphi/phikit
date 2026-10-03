//! Winding number of a path of phases: how many times a sentence goes round
//! the circle. For a closed path it is an integer and a topological
//! invariant; for an open path it is a real number, reported as such.

use crate::{wrap, TAU};

/// Sum of wrapped steps, divided by 2π.
pub fn winding(phases: &[f64]) -> f64 {
    phases.windows(2).map(|w| wrap(w[1] - w[0])).sum::<f64>() / TAU
}

/// Winding of the path closed back to its start — always (near) an integer.
pub fn winding_closed(phases: &[f64]) -> i64 {
    if phases.is_empty() {
        return 0;
    }
    let mut p = phases.to_vec();
    p.push(phases[0]);
    winding(&p).round() as i64
}

/// Total variation of the path: the length walked along the circle.
pub fn arc_length(phases: &[f64]) -> f64 {
    phases.windows(2).map(|w| wrap(w[1] - w[0]).abs()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_turn_winds_once_and_a_wobble_winds_zero() {
        let turn: Vec<f64> = (0..=16).map(|i| TAU * i as f64 / 16.0).collect();
        assert!((winding(&turn) - 1.0).abs() < 1e-9);
        assert_eq!(winding_closed(&turn[..16]), 1);
        let wobble = [0.0, 0.3, -0.2, 0.1, 0.0];
        assert!(winding(&wobble).abs() < 1e-9);
        assert_eq!(winding_closed(&wobble), 0);
        assert!((arc_length(&wobble) - 1.2).abs() < 1e-9);
    }
}
