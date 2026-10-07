//! `BImageView`'s geometry: the fit and the picture's rectangle.

use super::*;
use crate::v2::part::image::header::set::COMBINED;
// The geometry moved to `place.rs`, so these come from gpui rather than
// from `mod.rs`'s own imports (batch 164).
use gpui::{point, px, size};

#[test]
fn view_all_shows_a_fitting_image_at_one_and_a_larger_one_at_a_power_of_two() {
    assert_eq!(BImageView::view_all(800.0, 600.0, 1000.0, 700.0), 1.0);
    // 1920 × 1080 in 1268 × 486: the fit is 0.45, so 1/4.
    assert_eq!(BImageView::view_all(1920.0, 1080.0, 1268.0, 486.0), 0.25);
    assert_eq!(BImageView::view_all(1920.0, 1080.0, 1600.0, 800.0), 0.5);
    let view = BImageView { zoom: Some(2.0), pan: (10.0, -5.0), channels: BChannels::Color, pass: COMBINED.into(), slot_filled: true, status: None, shown: crate::v2::part::image::shown::BShownImage::RenderResult };
    // The guard on design literals reads `px(<number>)`; a test's geometry
    // goes through a helper.
    let at_px = |x: f32, y: f32, w: f32, h: f32| Bounds::new(point(px(x), px(y)), size(px(w), px(h)));
    let rect = view.rect(100.0, 50.0, at_px(100.0, 50.0, 400.0, 300.0));
    // Centred: (400 − 200) / 2 = 100, less the offset 2 × 10; (300 − 100) / 2 = 100, plus 2 × −5.
    assert_eq!(rect, at_px(180.0, 140.0, 200.0, 100.0));
    assert_eq!(BChannels::of("Alpha"), BChannels::Alpha);
    assert_eq!(BChannels::of("nonsense"), BChannels::Color);
}
