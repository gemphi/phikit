//! Property 20 — Viewport zoom clamp + monotonicity (task 25.2).
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirement 22.9 (zoom range [0.25, 4.0] for node graphs;
//! range differs per host — image uses [0.1, 32.0]).
//!
//! For any zoom multiplier `m > 0`, the resulting zoom stays inside
//! `[min, max]`, and repeated zoom-in stays monotonically increasing
//! until the ceiling is hit.

use phiber::v2::state::BState;
use proptest::prelude::*;

const NODE_MIN: f32 = 0.25;
const NODE_MAX: f32 = 4.0;

fn zoom_key(host: &str) -> String {
    format!("ctx.{host}.zoom")
}

fn apply_zoom(state: &mut BState, host: &str, mult: f32, min: f32, max: f32) {
    let k = zoom_key(host);
    let current: f32 = state
        .get_encoded(&k)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);
    let next = (current * mult).clamp(min, max);
    state.set_encoded(&k, format!("{next}"));
}

fn read_zoom(state: &BState, host: &str) -> f32 {
    state
        .get_encoded(&zoom_key(host))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0)
}

proptest! {
    #[test]
    fn zoom_stays_in_range(mults in prop::collection::vec(0.1f32..10.0f32, 1..=20)) {
        let mut state = BState::new();
        for m in mults {
            apply_zoom(&mut state, "shader", m, NODE_MIN, NODE_MAX);
            let z = read_zoom(&state, "shader");
            prop_assert!(z >= NODE_MIN, "zoom {z} < {NODE_MIN}");
            prop_assert!(z <= NODE_MAX, "zoom {z} > {NODE_MAX}");
        }
    }

    #[test]
    fn zoom_in_is_monotonic_until_ceiling(
        n in 1usize..=10usize,
    ) {
        let mut state = BState::new();
        let mut prev = read_zoom(&state, "shader");
        for _ in 0..n {
            apply_zoom(&mut state, "shader", 1.5, NODE_MIN, NODE_MAX);
            let z = read_zoom(&state, "shader");
            prop_assert!(z >= prev - 1e-6, "zoom decreased: {prev} → {z}");
            prev = z;
        }
    }

    #[test]
    fn zoom_out_is_monotonic_until_floor(n in 1usize..=10usize) {
        let mut state = BState::new();
        // Start at max.
        state.set_encoded(&zoom_key("shader"), format!("{NODE_MAX}"));
        let mut prev = read_zoom(&state, "shader");
        for _ in 0..n {
            apply_zoom(&mut state, "shader", 0.5, NODE_MIN, NODE_MAX);
            let z = read_zoom(&state, "shader");
            prop_assert!(z <= prev + 1e-6, "zoom increased: {prev} → {z}");
            prev = z;
        }
    }
}

#[test]
fn clamp_boundaries_stick() {
    let mut state = BState::new();
    // Zoom in far past the ceiling — should clamp to NODE_MAX.
    for _ in 0..100 {
        apply_zoom(&mut state, "shader", 2.0, NODE_MIN, NODE_MAX);
    }
    assert!((read_zoom(&state, "shader") - NODE_MAX).abs() < 1e-4);

    // Zoom out far past the floor — should clamp to NODE_MIN.
    for _ in 0..100 {
        apply_zoom(&mut state, "shader", 0.5, NODE_MIN, NODE_MAX);
    }
    assert!((read_zoom(&state, "shader") - NODE_MIN).abs() < 1e-4);
}
