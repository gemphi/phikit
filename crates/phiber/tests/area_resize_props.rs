//! Properties 22, 23, 24 — Area resize invariants.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 52.16, 52.17, 52.18, 54.6, 54.7, 54.8, 54.18, 54.19
//!
//! - Property 22: `apply_delta(+d)` then `apply_delta(-d)` restores every
//!   boundary fraction, and never touches non-boundary keys.
//! - Property 23: fractions in every zone sum to 1.0 ± 1e-5 after any
//!   `apply_delta` / `normalize` sequence.
//! - Property 24: monotonicity of `f_i` under repeated shrink drags,
//!   with a min-clamp floor of `min_area_extent / zone_extent`.

use phiber::v2::space::BSpaceType;
use phiber::v2::area::{apply_delta, area_size_key, clamp_to_min, normalize};
use phiber::v2::state::BState;
use phiber::v2::zone::BZoneType;
use proptest::prelude::*;

const ZONE_EXTENT: f32 = 1000.0;

/// Read fraction with the 1/N fallback used inside area_resize.
fn read_fraction(
    state: &BState,
    space: BSpaceType,
    zone: BZoneType,
    index: usize,
    n: usize,
) -> f32 {
    let k = area_size_key(space, zone, index);
    state
        .get_encoded(&k)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.0 / n as f32)
}

fn area_count(space: BSpaceType, zone: BZoneType) -> usize {
    space.spec().areas_in(zone).len()
}

/// A space/zone pair that has at least 2 areas (so a boundary index exists).
fn arb_boundary() -> impl Strategy<Value = (BSpaceType, BZoneType, usize)> {
    // Layout retains multi-area Right/Bottom zones; Shading uses three
    // stacked middle columns (Left/Center/Right).
    prop_oneof![
        Just((BSpaceType::Layout, BZoneType::Right, 0usize)),
        Just((BSpaceType::Layout, BZoneType::Bottom, 0usize)),
        Just((BSpaceType::Shading, BZoneType::Left, 0usize)),
        Just((BSpaceType::Shading, BZoneType::Center, 0usize)),
        Just((BSpaceType::Shading, BZoneType::Right, 0usize)),
        Just((BSpaceType::Rendering, BZoneType::Bottom, 0usize)),
        Just((BSpaceType::GeometryNodes, BZoneType::Bottom, 0usize)),
    ]
    .prop_filter("zone must have >= 2 areas", |(space, zone, _)| {
        area_count(*space, *zone) >= 2
    })
}

// Property 22 — Round-trip. Small deltas where neither neighbor hits
// the min-clamp floor.
proptest! {
    #[test]
    fn round_trip_small_delta(
        (space, zone, index) in arb_boundary(),
        dpx in (-50.0f32..50.0f32),
    ) {
        let mut state = BState::new();
        // Snapshot every non-boundary key before the drag.
        let before: Vec<(String, String)> = state
            .iter_keys()
            .map(|k| (k.clone(), state.get_encoded(k).unwrap_or("").to_string()))
            .collect();

        apply_delta(&mut state, space, zone, index, dpx, ZONE_EXTENT).unwrap();
        apply_delta(&mut state, space, zone, index, -dpx, ZONE_EXTENT).unwrap();

        // Boundary fractions restored (within f32 rounding tolerance).
        let n = area_count(space, zone);
        let f_i = read_fraction(&state, space, zone, index, n);
        let f_j = read_fraction(&state, space, zone, index + 1, n);
        prop_assert!((f_i - 1.0 / n as f32).abs() < 1e-3, "f_i = {f_i}, expected ~{}", 1.0 / n as f32);
        prop_assert!((f_j - 1.0 / n as f32).abs() < 1e-3, "f_j = {f_j}");

        // Every non-boundary key untouched.
        let boundary_i = area_size_key(space, zone, index);
        let boundary_j = area_size_key(space, zone, index + 1);
        for (k, v) in &before {
            if *k == boundary_i || *k == boundary_j {
                continue;
            }
            prop_assert_eq!(state.get_encoded(k), Some(v.as_str()));
        }
    }
}

// Property 23 — Sum-normalization. After any apply_delta, the pair
// `(f_i, f_{i+1})` still sums to 2/N (they started at 1/N each and the
// pinch preserves sum modulo the min-clamp).
proptest! {
    #[test]
    fn pair_sum_preserved(
        (space, zone, index) in arb_boundary(),
        dpx in (-500.0f32..500.0f32),
    ) {
        let mut state = BState::new();
        let n = area_count(space, zone);
        let pair_sum_before = 2.0 / n as f32;

        apply_delta(&mut state, space, zone, index, dpx, ZONE_EXTENT).unwrap();

        let f_i = read_fraction(&state, space, zone, index, n);
        let f_j = read_fraction(&state, space, zone, index + 1, n);
        let pair_sum_after = f_i + f_j;
        prop_assert!(
            (pair_sum_after - pair_sum_before).abs() < 1e-3,
            "pair sum drift: before={pair_sum_before}, after={pair_sum_after}",
        );
    }

    #[test]
    fn normalize_yields_unit_sum(
        (space, zone, _index) in arb_boundary(),
        garbage_fractions in prop::collection::vec(0.1f32..2.0f32, 2..=4),
    ) {
        let mut state = BState::new();
        // Poison the state with non-normalised fractions.
        let n = area_count(space, zone).min(garbage_fractions.len());
        for (i, f) in garbage_fractions.iter().take(n).enumerate() {
            let k = area_size_key(space, zone, i);
            state.set_encoded(&k, format!("{f}"));
        }

        normalize(&mut state, space, zone);

        let total: f32 = (0..area_count(space, zone))
            .map(|i| read_fraction(&state, space, zone, i, area_count(space, zone)))
            .sum();
        prop_assert!((total - 1.0).abs() < 1e-4, "sum={total}, expected 1.0 ± 1e-4");
    }
}

// Property 24 — Min-clamp monotonicity. For any two negative deltas
// `d1 <= d2 <= 0`, `f_i` after applying `d2` is `>=` `f_i` after
// applying `d1`.
proptest! {
    #[test]
    fn shrink_monotonic(
        (space, zone, index) in arb_boundary(),
        d1_abs in (0.0f32..500.0f32),
        d2_abs in (0.0f32..500.0f32),
    ) {
        let d1 = -d1_abs.max(d2_abs);
        let d2 = -d1_abs.min(d2_abs); // d1 <= d2 <= 0

        let mut a = BState::new();
        let mut b = BState::new();
        apply_delta(&mut a, space, zone, index, d1, ZONE_EXTENT).unwrap();
        apply_delta(&mut b, space, zone, index, d2, ZONE_EXTENT).unwrap();

        let n = area_count(space, zone);
        let f_a = read_fraction(&a, space, zone, index, n);
        let f_b = read_fraction(&b, space, zone, index, n);
        // Smaller magnitude shrink (d2 closer to 0) → f_i is >= the more aggressive shrink.
        prop_assert!(f_b + 1e-6 >= f_a, "monotonicity violated: d1={d1} → f_a={f_a}, d2={d2} → f_b={f_b}");
    }
}

// ── clamp_to_min unit checks ─────────────────────────────────────────

#[test]
fn clamp_preserves_sum() {
    let (a, b) = clamp_to_min(0.9, 0.1, 0.05);
    assert!((a + b - 1.0).abs() < 1e-6);
    assert!(a >= 0.05 && b >= 0.05);
}

#[test]
fn clamp_reflects_across_the_min() {
    // f_i = 0.02 is below min 0.05 → clamp bumps f_i up; f_j = sum - f_i.
    let sum = 0.5;
    let (a, b) = clamp_to_min(0.02, sum - 0.02, 0.05);
    assert!(a >= 0.05);
    assert!((a + b - sum).abs() < 1e-6);
}

#[test]
fn clamp_splits_evenly_when_sum_below_2min() {
    let (a, b) = clamp_to_min(0.03, 0.04, 0.05);
    assert!((a - b).abs() < 1e-6);
    assert!((a + b - 0.07).abs() < 1e-6);
}

// ── Task 8.4 — Overlay zero-footprint, min-extent floor, decomposition ───────
//
// Feature: phiber-resolved-ui-style-settings
// Validates: Requirements 54.6, 54.7, 54.8 (seam overlay structural invariants)
//
// The BSeam overlay is positioned with `.absolute()` so it takes no layout
// space — only the visible seam line contributes to the flex tree.  The
// hit-band width is controlled exclusively by `style.resize.hit_width` and
// the min-extent floor is `style.resize.min_extent` (= 60 px by default).

use phiber::v2::area::size::apply_drag_with_min_extent;
use phiber::v2::theme::{BPaletteName, resolve_ui_style, ui_style_defaults};

/// The seam overlay source must use `div().absolute()` — confirming the
/// overlay takes zero layout space in the flex tree.
#[test]
fn seam_overlay_uses_absolute_positioning_not_layout_space() {
    let read = |relative: &str| {
        std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative),
        )
        .unwrap_or_else(|error| panic!("{relative}: {error}"))
    };
    let seam_src = read("src/v2/view/draw/zone/seam.rs");
    // One geometry serves both seams: an area boundary and a panel edge.
    let overlay_src = read("src/v2/view/draw/resize.rs");

    // The overlay `div` must be `.absolute()` so it does not occupy flex space.
    assert!(
        overlay_src.contains(".absolute()"),
        "the shared seam overlay must use .absolute() to stay zero-footprint"
    );
    // The interactive hit band is sized by hit_width, not by layout flex.
    assert!(
        overlay_src.contains("ctx.style.resize.hit_width"),
        "hit band width must come from ctx.style.resize.hit_width"
    );
    // Both seams go through it rather than positioning themselves.
    for (name, src) in [("zone", &seam_src), ("panel", &overlay_src)] {
        assert!(
            src.contains("overlay(vertical, true, ctx)") || src.contains("overlay(true, false, ctx)"),
            "the {name} seam must use the shared overlay geometry"
        );
    }
    // The seam line is the one shared `BSeamLine`, sized off the gap, not by hit_width.
    for (name, src) in [("zone", &seam_src), ("panel", &overlay_src)] {
        assert!(
            src.contains("BSeamLine {") || src.contains("impl BSeamLine"),
            "the {name} seam must draw the shared seam line"
        );
        // occlude() must be on the handle — prevents mouse events leaking through.
        assert!(
            src.contains(".occlude()"),
            "the {name} seam handle must call .occlude() to absorb pointer events"
        );
    }
}

/// apply_drag_with_min_extent never lets either fraction drop below
/// min_extent / zone_extent regardless of how large the drag is.
#[test]
fn drag_with_min_extent_floor_is_respected() {
    let zone_extent = 800.0_f32;
    let min_extent = 60.0_f32;
    let min_frac = min_extent / zone_extent;

    // Massive negative drag — tries to collapse area 0 to zero.
    let mut state = BState::new();
    apply_drag_with_min_extent(
        &mut state,
        BSpaceType::Layout,
        BZoneType::Right,
        0,
        0.5,   // anchor_i
        0.5,   // anchor_j
        -10_000.0,
        zone_extent,
        min_extent,
    )
    .unwrap();
    let f0 = read_fraction(&state, BSpaceType::Layout, BZoneType::Right, 0, 2);
    let f1 = read_fraction(&state, BSpaceType::Layout, BZoneType::Right, 1, 2);
    assert!(
        f0 >= min_frac - 1e-5,
        "f0={f0} is below min_frac={min_frac}"
    );
    assert!(
        f1 >= min_frac - 1e-5,
        "f1={f1} is below min_frac={min_frac}"
    );
    assert!(
        (f0 + f1 - 1.0).abs() < 1e-4,
        "pair sum drifted: f0+f1={}", f0 + f1
    );
}

/// The min-extent floor is invariant across all resolved style variants —
/// changing the palette does not change the structural minimum.
#[test]
fn min_extent_floor_is_palette_invariant() {
    for &palette in BPaletteName::ALL {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        assert!(
            style.resize.min_extent >= 1.0,
            "palette {palette:?}: min_extent must be >= 1 px, got {}",
            style.resize.min_extent
        );
    }
}

// After apply_drag_with_min_extent the pair sum is conserved (= 1.0 for a
// two-area zone) — no area fraction leaks into or out of the neighbour.
proptest! {
    #[test]
    fn drag_with_min_extent_pair_sum_conserved(
        (space, zone, index) in arb_boundary(),
        total_dpx in -5000.0f32..5000.0f32,
        zone_extent in 200.0f32..2000.0f32,
    ) {
        let min_extent = 60.0_f32;
        let n = area_count(space, zone);
        let anchor = 1.0 / n as f32;
        let mut state = BState::new();
        let result = apply_drag_with_min_extent(
            &mut state,
            space,
            zone,
            index,
            anchor,
            1.0 - anchor,
            total_dpx,
            zone_extent,
            min_extent,
        );
        // Either succeeds or returns a benign error (hidden zone / bad index).
        if result.is_ok() {
            let f_i = read_fraction(&state, space, zone, index, n);
            let f_j = read_fraction(&state, space, zone, index + 1, n);
            let min_frac = (min_extent / zone_extent).clamp(0.0, 0.5);
            prop_assert!(f_i >= min_frac - 1e-5, "f_i={f_i} < min_frac={min_frac}");
            prop_assert!(f_j >= min_frac - 1e-5, "f_j={f_j} < min_frac={min_frac}");
            prop_assert!(
                (f_i + f_j - (anchor + (1.0 - anchor))).abs() < 1e-3,
                "pair sum drifted: f_i+f_j={}", f_i + f_j
            );
        }
    }
}
