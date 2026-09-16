//! Area resize math — pure `BState` transforms.
//!
//! Feature: phiber-blender-space-parity
//! Requirements: 54.5–54.13, 54.15, 54.17
//!
//! The zone's variable-axis extent is split between `N` areas via
//! fractions in `(0.0, 1.0)` that sum to `1.0`. Each fraction is
//! persisted at
//!
//! ```text
//! ctx.area_size.<space_slug>.<zone_slug>.<index>
//! ```
//!
//! An absent key defaults to the equal split `1 / N`. Every function
//! here is a pure transform on `BState` — no gpui, no I/O, no theme
//! widgets — so the property tests in
//! `crates/phiber/tests/area_resize_props.rs` can exercise the
//! invariants in isolation.

use super::{BState, V_EMPTY};
use crate::v2::space::BSpaceType;
use crate::v2::theme::METRICS;
use crate::v2::zone::BZoneType;

/// Minimum fraction the renderer keeps an area at so every declared area
/// stays visible (`area_fraction` clamps reads to this floor). The drag math
/// enforces the same floor in `clamp_to_min`, so the persisted fractions and
/// the rendered fractions never disagree — a boundary can be dragged flush
/// but never past the visibility floor.
pub const MIN_VISIBLE_FRACTION: f32 = 0.001;

/// Error returned by [`apply_delta`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeError {
    /// The space has no areas in this zone.
    UnknownZone,
    /// The boundary index is not in `0..N-1`.
    UnknownIndex,
    /// The zone is currently collapsed (K_ZONE_<zone> == V_FALSE).
    ZoneHidden,
    /// The zone has fewer than 2 areas — nothing to resize.
    InsufficientAreas,
}

/// Encoded state key for area fraction `<space>.<zone>.<index>`.
pub fn area_size_key(space: BSpaceType, zone: BZoneType, index: usize) -> String {
    format!("ctx.area_size.{}.{}.{}", space.slug(), zone.slug(), index)
}

/// Encoded state key prefix for the entire `<space>.<zone>` fraction set.
pub fn area_size_prefix(space: BSpaceType, zone: BZoneType) -> String {
    format!("ctx.area_size.{}.{}.", space.slug(), zone.slug())
}

/// Encoded state key for the zone visibility flag.
pub fn zone_visibility_key(zone: BZoneType) -> String {
    format!("ctx.zone.{}", zone.slug())
}

// Drag triple keys (constant strings — no dynamic segments).
pub const K_DRAG_SPACE: &str = "ctx.area_resize.drag.space";
pub const K_DRAG_ZONE: &str = "ctx.area_resize.drag.zone";
pub const K_DRAG_INDEX: &str = "ctx.area_resize.drag.index";

/// Number of areas in `(space, zone)`, from the space's static spec.
fn area_count(space: BSpaceType, zone: BZoneType) -> usize {
    space.spec().areas_in(zone).len()
}

/// Read the fraction for area `index` in `(space, zone)`. Absent keys
/// default to the equal split `1 / N`.
fn read_fraction(
    state: &BState,
    space: BSpaceType,
    zone: BZoneType,
    index: usize,
    n: usize,
) -> f32 {
    let k = area_size_key(space, zone, index);
    match state.get_encoded(&k) {
        Some(s) => s.parse::<f32>().unwrap_or_else(|_| 1.0 / n as f32),
        None => 1.0 / n as f32,
    }
}

/// Fraction consumed by the generic zone renderer. Invalid or absent values
/// fall back to an equal split, keeping every declared area visible.
pub fn area_fraction(state: &BState, space: BSpaceType, zone: BZoneType, index: usize) -> f32 {
    let n = area_count(space, zone);
    if n == 0 || index >= n {
        return 0.0;
    }
    read_fraction(state, space, zone, index, n).clamp(MIN_VISIBLE_FRACTION, 1.0)
}

/// Write the fraction for area `index`, canonicalising through `Debug`
/// (equivalent to `{:?}`) so the string round-trips through `parse::<f32>`.
fn write_fraction(state: &mut BState, space: BSpaceType, zone: BZoneType, index: usize, f: f32) {
    let k = area_size_key(space, zone, index);
    state.set_encoded(&k, format!("{f}"));
}

/// Two-neighbor pinch preserving `f_i + f_j`. Each returned fraction is
/// at least `min_frac`. If the total `S < 2 * min_frac`, the pair is
/// split evenly.
pub fn clamp_to_min(f_i: f32, f_j: f32, min_frac: f32) -> (f32, f32) {
    // Never let either area fall below the renderer's visibility floor, even
    // when the caller's `min_frac` is 0 (min_extent = 0). Otherwise the drag
    // could persist a 0.0 fraction that `area_fraction` then clamps up to
    // MIN_VISIBLE_FRACTION on read, making the persisted split and the
    // rendered split disagree.
    let floor = min_frac.max(MIN_VISIBLE_FRACTION);
    let sum = f_i + f_j;
    if sum < 2.0 * floor {
        let half = sum / 2.0;
        return (half, half);
    }
    let clamped_i = f_i.max(floor).min(sum - floor);
    let clamped_j = sum - clamped_i;
    (clamped_i, clamped_j)
}

/// Read the current fraction pair `(f_i, f_{i+1})` at `index` (with the
/// `1/N` fallback), for seeding a drag anchor. Returns `None` when the
/// zone has fewer than two areas or `index` is out of range.
pub fn boundary_pair(
    state: &BState,
    space: BSpaceType,
    zone: BZoneType,
    index: usize,
) -> Option<(f32, f32)> {
    let n = area_count(space, zone);
    if n < 2 || index + 1 >= n {
        return None;
    }
    Some((
        read_fraction(state, space, zone, index, n),
        read_fraction(state, space, zone, index + 1, n),
    ))
}

/// Apply an *absolute* boundary move: given the fraction pair the drag
/// started from (`anchor_i`, `anchor_j`) and the total pixel delta since the
/// drag began, write the clamped `f_i` / `f_{i+1}`. This is idempotent in the
/// total delta — re-dispatching the same move (any number of times, in any
/// order) converges to the same fractions, which is what makes the
/// interactive drag robust against gpui firing multiple move events per
/// frame. Only the two boundary keys are written.
pub fn apply_drag(
    state: &mut BState,
    space: BSpaceType,
    zone: BZoneType,
    index: usize,
    anchor_i: f32,
    anchor_j: f32,
    total_dpx: f32,
    zone_extent: f32,
) -> Result<(), ResizeError> {
    apply_drag_with_min_extent(
        state,
        space,
        zone,
        index,
        anchor_i,
        anchor_j,
        total_dpx,
        zone_extent,
        METRICS.min_area_extent,
    )
}

/// Resolved-style variant used by rendered seams. Keeping the legacy wrapper
/// above preserves callers while the active drag path receives the minimum
/// extent captured in its render-pass payload.
#[allow(clippy::too_many_arguments)]
pub fn apply_drag_with_min_extent(
    state: &mut BState,
    space: BSpaceType,
    zone: BZoneType,
    index: usize,
    anchor_i: f32,
    anchor_j: f32,
    total_dpx: f32,
    zone_extent: f32,
    min_extent: f32,
) -> Result<(), ResizeError> {
    let n = area_count(space, zone);
    if n == 0 {
        return Err(ResizeError::UnknownZone);
    }
    if n < 2 {
        return Err(ResizeError::InsufficientAreas);
    }
    if index + 1 >= n {
        return Err(ResizeError::UnknownIndex);
    }
    if state.get_encoded(&zone_visibility_key(zone)) == Some("false") {
        return Err(ResizeError::ZoneHidden);
    }
    if zone_extent <= 0.0 {
        return Ok(());
    }
    let df = total_dpx / zone_extent;
    let min_frac = (min_extent / zone_extent).clamp(0.0, 0.5);
    let (f_i, f_j) = clamp_to_min(anchor_i + df, anchor_j - df, min_frac);
    write_fraction(state, space, zone, index, f_i);
    write_fraction(state, space, zone, index + 1, f_j);
    Ok(())
}

/// Apply a pixel delta at boundary `index` inside `(space, zone)`.
///
/// Converts `dpx` to a fractional delta `Δf = dpx / zone_extent`, applies
/// the min-clamp, and writes only `f_i` and `f_{i+1}` back. Every other
/// fraction is untouched — that is what makes Property 22 (round-trip)
/// hold on the non-boundary keys.
pub fn apply_delta(
    state: &mut BState,
    space: BSpaceType,
    zone: BZoneType,
    index: usize,
    dpx: f32,
    zone_extent: f32,
) -> Result<(), ResizeError> {
    let n = area_count(space, zone);
    if n == 0 {
        return Err(ResizeError::UnknownZone);
    }
    if n < 2 {
        return Err(ResizeError::InsufficientAreas);
    }
    if index + 1 >= n {
        return Err(ResizeError::UnknownIndex);
    }
    if state.get_encoded(&zone_visibility_key(zone)) == Some("false") {
        return Err(ResizeError::ZoneHidden);
    }
    if zone_extent <= 0.0 {
        // Zero-extent frame: no-op (Requirement 54.14).
        return Ok(());
    }

    let f_i = read_fraction(state, space, zone, index, n);
    let f_j = read_fraction(state, space, zone, index + 1, n);
    let df = dpx / zone_extent;

    let f_i_new = f_i + df;
    let f_j_new = f_j - df;

    let min_frac = (METRICS.min_area_extent / zone_extent).clamp(0.0, 0.5);
    let (f_i_clamped, f_j_clamped) = clamp_to_min(f_i_new, f_j_new, min_frac);

    write_fraction(state, space, zone, index, f_i_clamped);
    write_fraction(state, space, zone, index + 1, f_j_clamped);
    Ok(())
}

/// Re-scale every fraction in `(space, zone)` so they sum to `1.0`
/// within `1e-5`. If every key is absent, do nothing (the `1/N` fallback
/// already gives a normalised split). Called defensively after
/// [`apply_delta`] and lazily on first layout.
pub fn normalize(state: &mut BState, space: BSpaceType, zone: BZoneType) {
    let n = area_count(space, zone);
    if n < 2 {
        return;
    }
    let mut fractions: Vec<f32> = (0..n)
        .map(|i| read_fraction(state, space, zone, i, n))
        .collect();
    let sum: f32 = fractions.iter().sum();
    if (sum - 1.0).abs() < 1e-5 || sum <= 0.0 {
        return;
    }
    for f in fractions.iter_mut() {
        *f /= sum;
    }
    for (i, f) in fractions.iter().enumerate() {
        write_fraction(state, space, zone, i, *f);
    }
}

/// Release-time snap: if `f_index` is within `snap_frac` of `1/N`,
/// rewrite `f_index` to `1/N` and adjust `f_{index+1}` to preserve the
/// pair sum.
pub fn snap_to_equal(
    state: &mut BState,
    space: BSpaceType,
    zone: BZoneType,
    index: usize,
    snap_frac: f32,
) {
    let n = area_count(space, zone);
    if n < 2 || index + 1 >= n {
        return;
    }
    let f_i = read_fraction(state, space, zone, index, n);
    let equal = 1.0 / n as f32;
    if (f_i - equal).abs() > snap_frac {
        return;
    }
    let f_j = read_fraction(state, space, zone, index + 1, n);
    let sum = f_i + f_j;
    write_fraction(state, space, zone, index, equal);
    write_fraction(state, space, zone, index + 1, sum - equal);
}

/// Clear every `ctx.area_size.<space>.<zone>.*` key so the `1/N`
/// fallback applies (double-click reset — Requirement 54.13).
pub fn reset_equal(state: &mut BState, space: BSpaceType, zone: BZoneType) {
    let prefix = area_size_prefix(space, zone);
    let victims: Vec<String> = state
        .iter_keys()
        .filter(|k| k.starts_with(&prefix))
        .cloned()
        .collect();
    for k in victims {
        state.set_encoded(&k, V_EMPTY);
    }
}

#[cfg(test)]
mod resolved_min_extent_tests {
    use super::*;

    #[test]
    fn rendered_drag_uses_the_resolved_minimum_extent() {
        let mut state = BState::new();
        apply_drag_with_min_extent(
            &mut state,
            BSpaceType::Shading,
            BZoneType::Right,
            0,
            0.5,
            0.5,
            -1_000.0,
            400.0,
            120.0,
        )
        .unwrap();

        assert_eq!(
            area_fraction(&state, BSpaceType::Shading, BZoneType::Right, 0),
            0.3
        );
        assert_eq!(
            area_fraction(&state, BSpaceType::Shading, BZoneType::Right, 1),
            0.7
        );
    }
}
