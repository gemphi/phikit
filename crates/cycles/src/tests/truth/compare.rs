// SPDX-License-Identifier: Apache-2.0
//! Tests of `truth/compare.rs` and `truth/allowance.rs`: Blender's rule of
//! `oiiotool --fail T --failpercent P --diff` (`render_report.py:138-144`).
use crate::truth::*;

/// A grey RGBA image of `count` pixels (a row) with the given value.
fn grey(count: usize, value: f32) -> Image {
    let mut image = Image::new(count, 1, 4);
    image.pixel.iter_mut().for_each(|channel| *channel = value);
    image
}

/// `grey` with the red of the first `failing` pixels raised by `by`.
fn raised(count: usize, failing: usize, by: f32) -> Image {
    let mut image = grey(count, 0.0);
    for pixel in 0..failing {
        image.pixel[pixel * 4] = by;
    }
    image
}

#[test]
fn a_reference_against_itself_passes_with_no_error() {
    let image = raised(1000, 500, 0.7);
    let result = compare(&image, &image, Allowance::default()).unwrap();
    assert_eq!((result.failed_pixel, result.max_error, result.failed), (0, 0.0, false));
    assert_eq!(result.pixel, 1000);
}

#[test]
fn a_channel_over_the_threshold_fails_the_pixel_and_exactly_at_it_does_not() {
    let allowance = Allowance { threshold: 0.016, percent: 0.0 };
    let reference = grey(10, 0.0);
    // `--fail 0.016` is read as a float; the error is compared to it with `>`.
    let at = compare(&reference, &raised(10, 10, 0.016), allowance).unwrap();
    assert_eq!(at.failed_pixel, 0, "an error equal to the threshold passes");
    let over = compare(&reference, &raised(10, 10, f32::from_bits(0.016f32.to_bits() + 1)), allowance).unwrap();
    assert_eq!(over.failed_pixel, 10, "the next float up fails");
    assert!(over.failed, "with no pixels allowed one failing pixel fails the render");
}

#[test]
fn any_one_channel_fails_the_pixel_and_a_pixel_counts_once() {
    let mut render = grey(4, 0.0);
    // Pixel 0 differs in all four channels, pixel 1 in the alpha only.
    render.pixel[0..4].copy_from_slice(&[0.5; 4]);
    render.pixel[7] = 0.5;
    let result = compare(&grey(4, 0.0), &render, Allowance::default()).unwrap();
    assert_eq!(result.failed_pixel, 2);
    assert_eq!(result.max_error, 0.5);
}

#[test]
fn one_percent_of_the_pixels_may_fail_and_one_more_may_not() {
    // 10 000 pixels: `nfail > failpercent / 100 * npixels`, so 100 pass.
    let reference = grey(10_000, 0.0);
    let allowance = Allowance::default();
    let hundred = compare(&reference, &raised(10_000, 100, 0.5), allowance).unwrap();
    assert!(!hundred.failed && hundred.failed_percent == 1.0, "exactly 1 per cent passes");
    let more = compare(&reference, &raised(10_000, 101, 0.5), allowance).unwrap();
    assert!(more.failed, "101 of 10 000 fails");
}

#[test]
fn a_missing_channel_reads_as_zero() {
    // An RGB reference against an RGBA render: the render's alpha of 1
    // is an error against the 0 the missing channel reads as.
    let rgb = Image::new(2, 1, 3);
    let mut rgba = Image::new(2, 1, 4);
    rgba.pixel.iter_mut().skip(3).step_by(4).for_each(|alpha| *alpha = 1.0);
    let result = compare(&rgb, &rgba, Allowance::default()).unwrap();
    assert_eq!(result.failed_pixel, 2);
}

#[test]
fn images_of_two_sizes_cannot_be_compared() {
    let result = compare(&grey(3, 0.0), &grey(4, 0.0), Allowance::default());
    assert!(matches!(result, Err(TruthError::Size { reference: (3, 1), render: (4, 1) })));
}

#[test]
fn a_nan_against_a_number_fails_and_equal_infinities_do_not() {
    let mut a = grey(2, 0.0);
    let mut b = grey(2, 0.0);
    a.pixel[0] = f32::NAN;
    a.pixel[4] = f32::INFINITY;
    b.pixel[4] = f32::INFINITY;
    let result = compare(&a, &b, Allowance::default()).unwrap();
    assert_eq!(result.failed_pixel, 1);
}

#[test]
fn the_diff_image_is_sixteen_times_the_difference() {
    let reference = raised(2, 2, 0.25);
    let diff = diff_image(&reference, &grey(2, 0.0), &[0, 1, 2]).unwrap();
    assert_eq!(diff.channels, 3);
    assert_eq!(&diff.pixel[0..3], &[4.0, 0.0, 0.0]);
    let alpha = diff_image(&grey(1, 0.5), &grey(1, 0.25), &[3]).unwrap();
    assert_eq!(alpha.pixel, [4.0]);
}

#[test]
fn the_default_allowance_is_blenders() {
    // render_report.py:287-288
    assert_eq!(Allowance::default(), Allowance { threshold: 0.016, percent: 1.0 });
    assert_eq!(Allowance::of_category("bsdf", false), Allowance::default());
    assert_eq!(Allowance::of_category("shader", false), Allowance::default());
}

#[test]
fn a_category_gets_the_allowance_blender_sets_for_it() {
    let of = |name| Allowance::of_category(name, false);
    // cycles_render_tests.py:337-339
    for name in ["motion_blur", "integrator", "displacement"] {
        assert_eq!(of(name), Allowance { threshold: 0.032, percent: 1.0 }, "{name}");
    }
    // 347-348 and 349-351: `light` is set twice and the second line wins.
    assert_eq!(of("shadow_catcher"), Allowance { threshold: 0.038, percent: 1.0 });
    assert_eq!(of("light"), Allowance { threshold: 0.02, percent: 4.0 });
    assert_eq!(of("camera"), Allowance { threshold: 0.02, percent: 4.0 });
    // 352-354
    assert_eq!(of("volume"), Allowance { threshold: 0.048, percent: 3.0 });
    assert_eq!(of("openvdb"), Allowance { threshold: 0.048, percent: 3.0 });
    // 356-357
    assert_eq!(of("colorspace"), Allowance { threshold: 0.05, percent: 1.0 });
}

#[test]
fn the_osl_allowances_are_kept_for_when_osl_runs() {
    // 338 and 343-344
    assert_eq!(Allowance::of_category("shader", true).threshold, 0.032);
    assert_eq!(Allowance::of_category("hair", true).threshold, 0.032);
    assert_eq!(Allowance::of_category("principled_bsdf", true).threshold, 0.06);
    assert_eq!(Allowance::of_category("principled_bsdf", false).threshold, 0.016);
}

#[test]
fn an_allowance_changes_the_verdict_on_the_same_pair() {
    // 3 per cent of the pixels differ by 0.019: past `bsdf`'s 1 per cent
    // and 0.016, inside `light`'s 4 per cent but over its 0.02? No: 0.019
    // is under light's 0.02, so no pixel fails there at all.
    let reference = grey(1000, 0.0);
    let render = raised(1000, 30, 0.019);
    let bsdf = compare(&reference, &render, Allowance::of_category("bsdf", false)).unwrap();
    let light = compare(&reference, &render, Allowance::of_category("light", false)).unwrap();
    assert!(bsdf.failed);
    assert_eq!(bsdf.failed_pixel, 30);
    assert!(!light.failed);
    assert_eq!(light.failed_pixel, 0);
}
