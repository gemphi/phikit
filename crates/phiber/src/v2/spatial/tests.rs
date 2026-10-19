use super::*;

#[test]
fn ladder_is_blenders_pixel_sizes() {
    // `rna_scene.cc:6922` — 1, 2, 4, 8.
    let pixels: Vec<u32> = TIER.iter().map(|t| t.pixel).collect();
    assert_eq!(pixels, vec![1, 2, 4, 8]);
}

#[test]
fn over_budget_steps_down_one_rung_at_a_time() {
    let budget = BUDGET_MS;
    assert_eq!(BSpatialTier::step(0, budget * 10.0, budget), 1);
    assert_eq!(BSpatialTier::step(1, budget * 10.0, budget), 2);
    // and never past the coarsest
    assert_eq!(BSpatialTier::step(3, budget * 10.0, budget), 3);
}

#[test]
fn climbs_only_with_room_for_the_tier_above() {
    let budget = BUDGET_MS;
    // Inside budget but not by half: stay put, or the view oscillates.
    assert_eq!(BSpatialTier::step(2, budget * 0.9, budget), 2);
    assert_eq!(BSpatialTier::step(2, budget * 0.4, budget), 1);
    assert_eq!(BSpatialTier::step(0, 0.1, budget), 0);
}

#[test]
fn a_frame_that_is_not_a_number_moves_nothing() {
    assert_eq!(BSpatialTier::step(1, f32::NAN, BUDGET_MS), 1);
    assert_eq!(BSpatialTier::step(1, 5.0, 0.0), 1);
}

#[test]
fn span_never_collapses_to_zero() {
    for t in BSpatialTier::every() {
        assert!(t.span(1) >= 1, "{} collapsed a 1px axis", t.name);
        assert_eq!(t.span(64), 64 / t.pixel);
    }
}

#[test]
fn readback_is_the_one_cost_that_can_go_to_zero() {
    let frame = BSpatialCost::of(1920, 1080, 4, false);
    assert_eq!(frame.pixels, 1920 * 1080);
    assert_eq!(frame.rays, 1920 * 1080 * 4);
    assert!(frame.crosses_bus());
    assert_eq!(frame.readback, 1920 * 1080 * cost::PIXEL_BYTES);

    let kept = frame.on_gpu();
    assert!(!kept.crosses_bus());
    // The paths are unchanged: nothing was given up to stop the copy.
    assert_eq!(kept.rays, frame.rays);
}

#[test]
fn reuse_is_none_until_something_was_recorded() {
    let mut view = BSpatial::default();
    assert_eq!(view.reuse(), None);
    view.record();
    for _ in 0..30 {
        view.frame(1.0);
    }
    assert_eq!(view.reuse(), Some(30.0));
}

#[test]
fn a_slow_view_walks_down_and_back_up() {
    let mut view = BSpatial::default();
    for _ in 0..8 {
        view.frame(BUDGET_MS * 4.0);
    }
    assert_eq!(view.at().name, "Eighth");
    assert!(!view.in_budget());
    for _ in 0..8 {
        view.frame(1.0);
    }
    assert_eq!(view.at().name, "Full");
    assert!(view.in_budget());
}

#[test]
fn a_coarser_tier_asks_for_fewer_paths() {
    let view = BSpatial::default();
    let fine = view.cost(1000, 1000, true);
    let mut coarse = view;
    coarse.tier = 3;
    assert!(coarse.cost(1000, 1000, true).rays < fine.rays);
}
