use crate::plan::*;

fn panel() -> GlassRequest {
    GlassRequest {
        bounds: [100.0, 100.0, 100.0, 50.0],
        visible: [100.0, 100.0, 200.0, 150.0],
        radii: [8.0; 4],
        blur_radius: 24.0,
        saturation: 1.5,
        lens: 26.0,
        tint: [0.1, 0.1, 0.1, 0.28],
        edge: [1.0, 1.0, 1.0, 0.75],
        opacity: 1.0,
        scale_factor: 1.0,
    }
}

#[test]
fn the_capture_reaches_past_the_shape_and_stays_in_the_window() {
    assert_eq!(capture_region([100.0, 100.0, 200.0, 150.0], 10.0, 4, 1000, 1000), [88, 88, 212, 160]);
    assert_eq!(capture_region([0.0, 0.0, 998.0, 999.0], 10.0, 4, 1000, 1000), [0, 0, 1000, 1000]);
}

#[test]
fn a_wide_blur_shrinks_further() {
    assert_eq!(shrink_for(8.0), 2);
    assert_eq!(shrink_for(24.0), 4);
}

#[test]
fn the_block_is_144_bytes_in_the_order_the_shader_declares() {
    assert_eq!(std::mem::size_of::<GlassParam>(), 144);
    let block = GlassParam { rect: [1.0, 2.0, 3.0, 4.0], sigma: 5.0, stage: STAGE_GLASS, rim_band: 9.0, ..GlassParam::default() };
    let bytes = block.bytes();
    let float = |at: usize| f32::from_ne_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    assert_eq!([float(0), float(4), float(8), float(12)], [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(float(56), 5.0, "sigma sits after step");
    assert_eq!(u32::from_ne_bytes([bytes[60], bytes[61], bytes[62], bytes[63]]), STAGE_GLASS);
    assert_eq!(float(140), 9.0, "rim_band is the last word");
}

#[test]
fn a_plan_is_shrink_then_two_blurs_then_the_shape() {
    let plan = GlassPlan::of(&panel(), 1000, 800).expect("the panel shows");
    assert_eq!(plan.stage.map(|stage| stage.stage), [STAGE_SHRINK, STAGE_BLUR, STAGE_BLUR, STAGE_GLASS]);
    assert_eq!(plan.small, [500.0, 400.0]);
    let [across, down] = [plan.stage[1], plan.stage[2]];
    assert!(across.step[0] > 0.0 && across.step[1] == 0.0, "the first blur goes across");
    assert!(down.step[0] == 0.0 && down.step[1] > 0.0, "the second goes down");
    assert_eq!(plan.stage[3].rect, [100.0, 100.0, 100.0, 50.0], "the shape is drawn where it shows");
    assert_eq!(plan.stage[3].target_size, [1000.0, 800.0]);
}

#[test]
fn the_copy_holds_the_blur_and_the_lens_around_the_shape() {
    let plan = GlassPlan::of(&panel(), 1000, 800).expect("the panel shows");
    let [l, t, r, b] = plan.capture;
    // 24 px of blur reaches 36, and the lens is held to half the short side, 25.
    assert!(l <= 100 - 61 && t <= 100 - 61 && r >= 200 + 61 && b >= 150 + 61, "{:?}", plan.capture);
    assert!(l % 4 == 0 && t % 4 == 0 && r % 4 == 0 && b % 4 == 0, "on whole texels of the quarter-size copy");
}

#[test]
fn glass_that_does_not_show_has_no_plan() {
    let clipped = GlassRequest { visible: [100.0, 100.0, 100.0, 150.0], ..panel() };
    let faded = GlassRequest { opacity: 0.0, ..panel() };
    assert!(GlassPlan::of(&clipped, 1000, 800).is_none());
    assert!(GlassPlan::of(&faded, 1000, 800).is_none());
    assert!(GlassPlan::of(&panel(), 0, 800).is_none());
}

#[test]
fn a_denser_screen_scales_the_lengths_and_keeps_the_rim_inside_the_shape() {
    let dense = GlassRequest { scale_factor: 2.0, ..panel() };
    let plan = GlassPlan::of(&dense, 2000, 1600).expect("the panel shows");
    assert_eq!(plan.stage[3].lens, 25.0, "the lens stops at half the short side");
    assert_eq!(plan.stage[3].rim_band, 25.0, "and so does the rim");
    let one = GlassPlan::of(&panel(), 2000, 1600).expect("the panel shows");
    assert!(plan.stage[1].sigma > one.stage[1].sigma, "twice the pixels, a wider blur in them");
}
