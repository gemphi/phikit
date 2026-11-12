// SPDX-License-Identifier: Apache-2.0
//! Tests of `truth/display.rs` and `display_view`: the colour management
//! between the film and the PNG.
use crate::session::{BufferParam, RenderBuffer};
use crate::truth::*;

fn buffer(rows: &[[f32; 4]], width: i32) -> RenderBuffer {
    let mut param = BufferParam::full(width, rows.len() as i32 / width);
    param.pass_stride = 4;
    let mut buffer = RenderBuffer::default();
    buffer.reset(&param);
    buffer.buffer = rows.iter().flatten().copied().collect();
    buffer
}

#[test]
fn the_standard_transform_is_the_srgb_curve() {
    let view = ViewSettings::default();
    assert_eq!(view.apply(0.0), 0.0);
    assert!((view.apply(0.5) - 0.735_357).abs() < 1e-5);
    assert!((view.apply(0.05) - 0.247_8).abs() < 1e-3);
    assert_eq!(view.apply(2.0), 1.0);
    assert!((view.apply(0.001) - 0.012_92).abs() < 1e-6);
}

#[test]
fn the_scene_s_names_pick_the_transform_and_say_when_it_is_not_ported() {
    let standard = ViewSettings::of("Standard", "None", 0.0, 1.0, 1.0);
    assert_eq!(standard.transform, ViewTransform::Standard);
    assert!(!standard.approximated());
    // A file that holds no transform reads as `Default`: the display's first view.
    assert_eq!(ViewSettings::of("Default", "None", 0.0, 1.0, 1.0).transform, ViewTransform::Standard);
    assert_eq!(ViewSettings::of("Raw", "None", 0.0, 1.0, 1.0).transform, ViewTransform::Raw);
    let agx = ViewSettings::of("AgX", "None", 0.0, 1.0, 1.0);
    assert_eq!(agx.transform, ViewTransform::Other("AgX".into()));
    assert!(agx.approximated());
    assert!(ViewSettings::of("Standard", "Very High Contrast", 0.0, 1.0, 1.0).approximated());
}

#[test]
fn exposure_in_stops_scales_the_light_before_the_curve() {
    let one_stop = ViewSettings::of("Standard", "None", 1.0, 1.0, 1.0);
    assert!((one_stop.apply(0.1) - ViewSettings::default().apply(0.2)).abs() < 1e-6);
    // The film's own exposure scales the pass the same way.
    let film = ViewSettings::of("Standard", "None", 0.0, 1.0, 2.0);
    assert!((film.apply(0.1) - ViewSettings::default().apply(0.2)).abs() < 1e-6);
}

#[test]
fn gamma_follows_the_curve_and_raw_has_no_curve() {
    let gamma = ViewSettings::of("Standard", "None", 0.0, 2.0, 1.0);
    assert!((gamma.apply(0.5) - 0.735_357f32.powf(0.5)).abs() < 1e-5);
    let raw = ViewSettings::of("Raw", "None", 0.0, 1.0, 1.0);
    assert_eq!((raw.apply(0.5), raw.apply(3.0), raw.apply(-1.0)), (0.5, 1.0, 0.0));
}

#[test]
fn a_cycles_buffer_is_read_bottom_row_first_and_alpha_as_transparency() {
    // Two rows of one pixel: the buffer's first row is the image's last.
    let film = buffer(&[[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 0.25]], 1);
    let view = ViewSettings::of("Raw", "None", 0.0, 1.0, 1.0);
    let image = display_view(&film, 1, true, &view);
    assert_eq!(image.pixel, [1.0, 1.0, 1.0, 0.75, 0.0, 0.0, 0.0, 1.0]);
    // Left as it is without the flag: `display` keeps the old reading.
    let plain = display(&film, 1, true);
    assert_eq!(plain.pixel[..3], [0.0; 3]);
    assert_eq!(plain.pixel[7], 0.25);
}

#[test]
fn the_sum_is_divided_by_the_sample_count() {
    let film = buffer(&[[2.0, 1.0, 0.5, 0.0]], 1);
    let image = display_view(&film, 4, false, &ViewSettings::of("Raw", "None", 0.0, 1.0, 1.0));
    assert_eq!(image.pixel, [0.5, 0.25, 0.125]);
}
