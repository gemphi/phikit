use crate::*;

#[test]
fn an_effect_tints_with_its_own_colour_before_the_theme_s() {
    let panel = [0.2, 0.2, 0.2, 1.0];
    let plain = GlassEffect::default().material(panel, None, [1.0; 3], [0.0; 3]);
    let tinted = GlassEffect { tint: Some([0.9, 0.1, 0.1, 1.0]), ..GlassEffect::default() }.material(panel, None, [1.0; 3], [0.0; 3]);
    assert_eq!(plain.tint, [0.2, 0.2, 0.2]);
    assert_eq!(tinted.tint, [0.9, 0.1, 0.1]);
}

#[test]
fn the_corner_radius_stops_at_half_the_short_side() {
    let shape = GlassEffect { corner_radius: 80.0, ..GlassEffect::default() }.shape(0.0, 0.0, 200.0, 100.0);
    assert_eq!(shape.corner_radius, 50.0);
}
