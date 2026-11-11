use super::*;

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5)
}

/// Pure red is hue 0 at full saturation; HSL lightness is half where
/// HSV value is one. Each separation combines back to the colour.
#[test]
fn hsv_and_hsl_round_trip_through_cycles_formulas() {
    assert!(close(separate_color(COMBSEP_HSV, [1.0, 0.0, 0.0]), [0.0, 1.0, 1.0]));
    assert!(close(separate_color(COMBSEP_HSL, [1.0, 0.0, 0.0]), [0.0, 1.0, 0.5]));
    assert!(close(separate_color(COMBSEP_RGB, [0.2, 0.6, 0.4]), [0.2, 0.6, 0.4]));
    for c in [[0.2, 0.6, 0.4], [0.9, 0.1, 0.5], [0.3, 0.3, 0.3], [0.0, 0.7, 0.7]] {
        for mode in [COMBSEP_RGB, COMBSEP_HSV, COMBSEP_HSL] {
            assert!(close(combine_color(mode, separate_color(mode, c)), c), "{c:?} in mode {mode}");
        }
    }
    // Blender's teal (0, 0.7, 0.7): hue ½, HSL saturation 1 and
    // lightness 0.35.
    assert!(close(rgb_to_hsl([0.0, 0.7, 0.7]), [0.5, 1.0, 0.35]), "{:?}", rgb_to_hsl([0.0, 0.7, 0.7]));
}
