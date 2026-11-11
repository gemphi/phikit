use super::*;

#[test]
fn bright_contrast_and_gamma_follow_the_glsl() {
    assert_eq!(bright_contrast([0.5, 0.25, 1.0, 0.7], 0.1, 0.4), [0.5 * 1.4 - 0.1, 0.25 * 1.4 - 0.1, 1.4 - 0.1, 0.7]);
    assert_eq!(bright_contrast([0.0, 0.0, 0.0, 1.0], -1.0, 0.0), [0.0, 0.0, 0.0, 1.0], "clamped at zero");
    let g = gamma([0.25, -0.5, 0.0, 1.0], 0.5);
    assert!((g[0] - 0.5).abs() < 1e-6);
    assert_eq!((g[1], g[2], g[3]), (-0.5, 0.0, 1.0), "only positive channels are raised");
}

#[test]
fn math_is_blenders_table() {
    assert_eq!(math(3, 1.0, 0.0, 0.0), Some(0.0), "safe divide");
    assert_eq!(math(10, -2.0, 3.0, 0.0), Some(-8.0), "compatible pow keeps a negative base");
    assert_eq!(math(17, 7.5, 2.0, 0.0), Some(1.5));
    assert_eq!(math(33, 7.3, 2.0, 0.0), Some(6.0), "snap");
    assert_eq!(math(34, 5.5, 0.0, 2.0), Some(1.5), "wrap into [0, 2)");
    assert_eq!(math(35, 1.0, 1.0004, 0.001), Some(1.0), "compare within epsilon");
    assert_eq!(math(36, 2.0, 3.0, 4.0), Some(10.0), "multiply add");
    assert_eq!(math(37, 3.0, 2.0, 0.0), Some(1.0), "ping-pong");
    assert_eq!(math(40, -1.0, 3.0, 0.0), Some(2.0), "floored modulo");
    assert_eq!(math(41, 1.0, 1.0, 1.0), None, "an unknown operation folds to nothing");
    assert_eq!(clamp(1, 0.5, 1.0, 0.0), 0.5, "range clamp accepts swapped bounds");
    assert_eq!(clamp(0, 0.5, 1.0, 0.0), 0.0, "min/max clamp applies min then max");
}
