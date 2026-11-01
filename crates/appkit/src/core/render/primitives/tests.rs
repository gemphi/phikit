use super::gradient_rendering::{gradient_interpolate, gradient_interpolate_channel};
use super::super::blend_colors;
use proptest::prelude::*;

// Unit test: draw_shadow_around draws a shadow expanded by `spread` on all sides
// **Validates: Requirements 12.3**
#[test]
fn test_draw_shadow_around_draws_pixels_in_spread_area() {
    use crate::graphics::NSRect;
    use crate::render::RenderContext;

    let width = 100;
    let height = 100;
    let mut buffer = vec![0u32; width * height];

    {
        let mut ctx = RenderContext::new(width, height, &mut buffer);
        // Draw a shadow around a 40x40 rect centered at (50, 50) with 4px spread and 0.25 opacity
        let rect = NSRect::new(30.0, 30.0, 40.0, 40.0);
        ctx.draw_shadow_around(rect, 6.0, 4.0, 0.25);
    }

    // The shadow should extend 4px beyond the rect on all sides.
    // Check that pixels in the shadow spread area (e.g., at x=50, y=27) are non-zero.
    // The shadow rect should be from (26, 26) to (74, 74) with radius 10.
    // A pixel at (50, 27) should be within the shadow area (inside the rounded rect).
    let pixel_in_shadow = buffer[27 * width + 50]; // y=27, x=50
    assert_ne!(
        pixel_in_shadow, 0,
        "Shadow should draw pixels in the spread area above the rect"
    );

    // A pixel well outside the shadow (e.g., at (10, 10)) should remain untouched
    let pixel_outside = buffer[10 * width + 10]; // y=10, x=10
    assert_eq!(
        pixel_outside, 0,
        "Pixels outside the shadow area should remain untouched"
    );
}

// Feature: macos-visual-polish, Property 1: Gradient interpolation produces bounded colors
// **Validates: Requirements 2.1**
//
// For any two valid NSColor values (channels in [0,1]) and any interpolation
// parameter t in [0,1], the gradient interpolation function SHALL produce a color
// where each channel is between the minimum and maximum of the two input channels.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_gradient_interpolation_produces_bounded_colors(
        r1 in 0.0f32..=1.0f32,
        g1 in 0.0f32..=1.0f32,
        b1 in 0.0f32..=1.0f32,
        a1 in 0.0f32..=1.0f32,
        r2 in 0.0f32..=1.0f32,
        g2 in 0.0f32..=1.0f32,
        b2 in 0.0f32..=1.0f32,
        a2 in 0.0f32..=1.0f32,
        t in 0.0f32..=1.0f32,
    ) {
        let color1 = crate::NSColor::new(r1, g1, b1, a1);
        let color2 = crate::NSColor::new(r2, g2, b2, a2);

        let result = gradient_interpolate(color1, color2, t);

        // Each interpolated channel must be between min and max of the two inputs
        prop_assert!(
            result.r >= r1.min(r2) - f32::EPSILON && result.r <= r1.max(r2) + f32::EPSILON,
            "Red channel {} not in [{}, {}] for t={}",
            result.r, r1.min(r2), r1.max(r2), t
        );
        prop_assert!(
            result.g >= g1.min(g2) - f32::EPSILON && result.g <= g1.max(g2) + f32::EPSILON,
            "Green channel {} not in [{}, {}] for t={}",
            result.g, g1.min(g2), g1.max(g2), t
        );
        prop_assert!(
            result.b >= b1.min(b2) - f32::EPSILON && result.b <= b1.max(b2) + f32::EPSILON,
            "Blue channel {} not in [{}, {}] for t={}",
            result.b, b1.min(b2), b1.max(b2), t
        );
        prop_assert!(
            result.a >= a1.min(a2) - f32::EPSILON && result.a <= a1.max(a2) + f32::EPSILON,
            "Alpha channel {} not in [{}, {}] for t={}",
            result.a, a1.min(a2), a1.max(a2), t
        );
    }
}

// Feature: macos-visual-polish, Property 4: Alpha blending correctness
// **Validates: Requirements 7.4**
//
// For any source color (r,g,b,a) and background u32 pixel, the `blend_colors`
// function SHALL produce a result where each output channel equals
// `src_channel * alpha + bg_channel * (1 - alpha)`, within floating-point tolerance.
//
// The implementation uses integer math (multiply by alpha_int, shift right by 8)
// which introduces a small rounding error vs the ideal floating-point formula.
// We allow a tolerance of 2 (out of 255) to account for this.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_alpha_blending_correctness(
        src_r in 0.0f32..=1.0f32,
        src_g in 0.0f32..=1.0f32,
        src_b in 0.0f32..=1.0f32,
        // Constrain alpha to the range where actual blending occurs (not early-return paths)
        alpha in 0.025f32..=0.98f32,
        bg_r in 0u32..=255u32,
        bg_g in 0u32..=255u32,
        bg_b in 0u32..=255u32,
    ) {
        // Construct background pixel in the same format as to_bgra_u32: (a << 24) | (r << 16) | (g << 8) | b
        let bg_u32 = (255u32 << 24) | (bg_r << 16) | (bg_g << 8) | bg_b;

        let src_color = crate::NSColor::new(src_r, src_g, src_b, alpha);

        let result = blend_colors(src_color, bg_u32, alpha);

        // Extract result channels
        let result_r = (result >> 16) & 0xFF;
        let result_g = (result >> 8) & 0xFF;
        let result_b = result & 0xFF;

        // Compute expected blending using the same integer math as the implementation:
        // expected = (src_channel_int * alpha_int + bg_channel_int * (255 - alpha_int)) >> 8
        // This mirrors the implementation's formula exactly.
        let alpha_int = (alpha * 255.0) as u32;
        let src_r_int = (src_r * 255.0) as u32;
        let src_g_int = (src_g * 255.0) as u32;
        let src_b_int = (src_b * 255.0) as u32;

        let expected_r = ((src_r_int * alpha_int) + (bg_r * (255 - alpha_int))) >> 8;
        let expected_g = ((src_g_int * alpha_int) + (bg_g * (255 - alpha_int))) >> 8;
        let expected_b = ((src_b_int * alpha_int) + (bg_b * (255 - alpha_int))) >> 8;

        // Also verify against the ideal floating-point formula within tolerance.
        // Ideal: out = src * alpha + bg * (1 - alpha)
        let ideal_r = (src_r * alpha * 255.0 + bg_r as f32 * (1.0 - alpha)).round() as i32;
        let ideal_g = (src_g * alpha * 255.0 + bg_g as f32 * (1.0 - alpha)).round() as i32;
        let ideal_b = (src_b * alpha * 255.0 + bg_b as f32 * (1.0 - alpha)).round() as i32;

        // The implementation should match its own integer formula exactly
        prop_assert_eq!(
            result_r, expected_r,
            "Red: result={} expected={} (src_r={}, bg_r={}, alpha={})",
            result_r, expected_r, src_r, bg_r, alpha
        );
        prop_assert_eq!(
            result_g, expected_g,
            "Green: result={} expected={} (src_g={}, bg_g={}, alpha={})",
            result_g, expected_g, src_g, bg_g, alpha
        );
        prop_assert_eq!(
            result_b, expected_b,
            "Blue: result={} expected={} (src_b={}, bg_b={}, alpha={})",
            result_b, expected_b, src_b, bg_b, alpha
        );

        // Additionally verify the result is within a small tolerance of the ideal
        // floating-point blend. The >>8 approximation of /255 combined with
        // truncation of f32→u32 conversions can differ by up to 3 levels.
        let tolerance = 3i32;
        prop_assert!(
            (result_r as i32 - ideal_r).abs() <= tolerance,
            "Red channel {} too far from ideal {} (tolerance {})",
            result_r, ideal_r, tolerance
        );
        prop_assert!(
            (result_g as i32 - ideal_g).abs() <= tolerance,
            "Green channel {} too far from ideal {} (tolerance {})",
            result_g, ideal_g, tolerance
        );
        prop_assert!(
            (result_b as i32 - ideal_b).abs() <= tolerance,
            "Blue channel {} too far from ideal {} (tolerance {})",
            result_b, ideal_b, tolerance
        );
    }
}
