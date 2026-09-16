//! Feature: phiber-resolved-ui-style-settings, Property 12
//! **Validates: Requirements 7.8, 11.4**

use phiber::v2::state::key::ui_style as style_key;
use phiber::v2::{
    space::BSpaceType,
    state::{
        BState, area_fraction,
        area_resize::apply_drag_with_min_extent,
        area_size_key, boundary_pair, normalize,
        zone_size::{apply_zone_drag_with_min_extent, zone_size_key},
    },
    theme::{BPaletteName, BResolvedUiStyle, resolve_ui_style, ui_style_defaults},
    zone::BZoneType,
};
use proptest::prelude::*;

#[derive(Clone, Copy, Debug)]
struct ResizePresentation {
    visible_seam: f32,
    hit_width: f32,
    gap: f32,
    inset: f32,
}

fn arb_distinct_palettes() -> impl Strategy<Value = (BPaletteName, BPaletteName)> {
    prop_oneof![
        Just((BPaletteName::AppleDark, BPaletteName::BlenderDark)),
        Just((BPaletteName::AppleDark, BPaletteName::TokyoNight)),
        Just((BPaletteName::BlenderDark, BPaletteName::TokyoNight)),
    ]
}

fn arb_presentation_pair() -> impl Strategy<Value = (ResizePresentation, ResizePresentation)> {
    (
        0.0f32..64.0,
        0.0f32..64.0,
        1.0f32..64.0,
        0.0f32..128.0,
        -128.0f32..128.0,
        0.0f32..128.0,
        -128.0f32..128.0,
    )
        .prop_map(
            |(seam, extra, seam_delta, gap_a, inset_a, gap_b, inset_b)| {
                let seam_b = seam + seam_delta;
                (
                    ResizePresentation {
                        visible_seam: seam,
                        hit_width: seam + extra,
                        gap: gap_a,
                        inset: inset_a,
                    },
                    ResizePresentation {
                        visible_seam: seam_b,
                        hit_width: seam_b + extra,
                        gap: gap_b,
                        inset: inset_b,
                    },
                )
            },
        )
}

fn resolved_style(
    palette: BPaletteName,
    presentation: ResizePresentation,
    min_extent: f32,
) -> BResolvedUiStyle {
    let mut state = BState::new();
    for (key, value) in [
        (&style_key::RESIZE_VISIBLE_SEAM, presentation.visible_seam),
        (&style_key::RESIZE_HIT_WIDTH, presentation.hit_width),
        (&style_key::RESIZE_GAP, presentation.gap),
        (&style_key::RESIZE_INSET, presentation.inset),
        (&style_key::RESIZE_MIN_EXTENT, min_extent),
    ] {
        state.set(key, value.to_string());
    }
    resolve_ui_style(ui_style_defaults(palette), &state)
}

fn area_count(space: BSpaceType, zone: BZoneType) -> usize {
    space.spec().areas_in(zone).len()
}

fn arb_area_state() -> impl Strategy<Value = (BSpaceType, BZoneType, usize, Vec<u16>)> {
    let valid_boundaries: Vec<_> = BSpaceType::EVERY
        .iter()
        .copied()
        .flat_map(|space| {
            BZoneType::ALL.iter().copied().flat_map(move |zone| {
                let count = area_count(space, zone);
                (0..count.saturating_sub(1)).map(move |index| (space, zone, index))
            })
        })
        .collect();

    prop::sample::select(valid_boundaries)
        .prop_flat_map(|(space, zone, index)| {
            let count = area_count(space, zone);
            (
                Just((space, zone, index)),
                prop::collection::vec(1u16..=1_000, count),
            )
        })
        .prop_map(|((space, zone, index), weights)| (space, zone, index, weights))
}

fn initial_area_state(space: BSpaceType, zone: BZoneType, weights: &[u16]) -> BState {
    let mut state = BState::new();
    let total: f32 = weights.iter().map(|weight| *weight as f32).sum();
    for (index, weight) in weights.iter().enumerate() {
        state.set_encoded(
            &area_size_key(space, zone, index),
            (*weight as f32 / total).to_string(),
        );
    }
    state.set_encoded("ctx.test.resize_unrelated", "preserved");
    state
}

fn state_snapshot(state: &BState) -> Vec<(String, String)> {
    let mut snapshot: Vec<_> = state
        .iter_keys()
        .map(|key| {
            (
                key.clone(),
                state.get_encoded(key).unwrap_or_default().to_string(),
            )
        })
        .collect();
    snapshot.sort_unstable();
    snapshot
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 12: Resize state behavior is style-independent.
    ///
    /// Visual palette and hit-target presentation vary independently. The
    /// state transform receives only the resolved minimum extent required by
    /// the resize contract, so area fractions and zone sizes remain identical.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 12
    /// **Validates: Requirements 7.8, 11.4**
    #[test]
    fn visual_style_does_not_change_area_or_zone_resize_state(
        (palette_a, palette_b) in arb_distinct_palettes(),
        (presentation_a, presentation_b) in arb_presentation_pair(),
        (area_space, area_zone, area_index, weights) in arb_area_state(),
        zone_space in prop::sample::select(BSpaceType::EVERY.to_vec()),
        zone in prop_oneof![Just(BZoneType::Left), Just(BZoneType::Right), Just(BZoneType::Bottom)],
        area_delta in -4_096.0f32..4_096.0,
        zone_delta in -4_096.0f32..4_096.0,
        zone_sign in prop_oneof![Just(-1.0f32), Just(1.0f32)],
        extent in 128.0f32..4_096.0,
        zone_anchor in 1.0f32..4_096.0,
        min_extent in 0.0f32..=512.0,
    ) {
        let style_a = resolved_style(palette_a, presentation_a, min_extent);
        let style_b = resolved_style(palette_b, presentation_b, min_extent);

        prop_assert_ne!(style_a.colors, style_b.colors);
        prop_assert_ne!(style_a.resize.visible_seam, style_b.resize.visible_seam);
        prop_assert_eq!(style_a.resize.min_extent, style_b.resize.min_extent);

        let initial_area = initial_area_state(area_space, area_zone, &weights);
        let (anchor_i, anchor_j) = boundary_pair(
            &initial_area,
            area_space,
            area_zone,
            area_index,
        ).expect("generated area state has a valid boundary");
        let pair_sum_before = anchor_i + anchor_j;
        let mut area_a = initial_area.clone();
        let mut area_b = initial_area;

        apply_drag_with_min_extent(
            &mut area_a, area_space, area_zone, area_index,
            anchor_i, anchor_j, area_delta, extent, style_a.resize.min_extent,
        ).expect("generated area drag is accepted");
        apply_drag_with_min_extent(
            &mut area_b, area_space, area_zone, area_index,
            anchor_i, anchor_j, area_delta, extent, style_b.resize.min_extent,
        ).expect("generated area drag is accepted");
        normalize(&mut area_a, area_space, area_zone);
        normalize(&mut area_b, area_space, area_zone);

        prop_assert_eq!(state_snapshot(&area_a), state_snapshot(&area_b));
        let pair_sum_after = area_fraction(&area_a, area_space, area_zone, area_index)
            + area_fraction(&area_a, area_space, area_zone, area_index + 1);
        prop_assert!((pair_sum_after - pair_sum_before).abs() < 1e-4);
        prop_assert_eq!(area_a.get_encoded("ctx.test.resize_unrelated"), Some("preserved"));

        let zone_key = zone_size_key(zone_space, zone);
        let mut zone_a = BState::new();
        zone_a.set_encoded(&zone_key, zone_anchor.to_string());
        zone_a.set_encoded("ctx.test.resize_unrelated", "preserved");
        let mut zone_b = zone_a.clone();

        let changed_a = apply_zone_drag_with_min_extent(
            &mut zone_a, zone_space, zone, zone_anchor, zone_delta,
            zone_sign, extent, style_a.resize.min_extent,
        );
        let changed_b = apply_zone_drag_with_min_extent(
            &mut zone_b, zone_space, zone, zone_anchor, zone_delta,
            zone_sign, extent, style_b.resize.min_extent,
        );

        prop_assert!(changed_a && changed_b);
        prop_assert_eq!(state_snapshot(&zone_a), state_snapshot(&zone_b));
        prop_assert_eq!(zone_a.get_encoded("ctx.test.resize_unrelated"), Some("preserved"));
    }
}
