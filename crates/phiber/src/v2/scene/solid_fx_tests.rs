//! `BSolidFx` against the startup file's values and a file's: the outline,
//! Cavity and its words, the SSAO settings, Shadow and its words.

use super::*;

#[test]
fn the_factory_outline_is_on_and_a_files_cavity_is_read_back() {
    let mut s = BState::new();
    let f = BSolidFx::now(&s);
    assert_eq!(f, BSolidFx::FACTORY);
    assert_eq!(f.fx_words(1.0), [1.0, 0.0, 0.0, 1.0], "the outline alone; no curvature while Cavity is off");
    assert_eq!(f.outline_words(), [0.0, 0.0, 0.0, 1.0]);
    let file = SceneSolidFx { outline: false, outline_color: [1.0, 0.5, 0.0], cavity: true, cavity_type: "BOTH", cavity_valley_factor: 1.0, cavity_ridge_factor: 1.0, curvature_ridge_factor: 2.0, curvature_valley_factor: 0.5, shadow: true, shadow_intensity: 0.8, backface_culling: true, specular: false };
    BSolidFx::seed(&mut s, &file);
    let f = BSolidFx::now(&s);
    assert!(f.backface_culling && !f.specular, "the file's culling and no specular");
    assert_eq!(f.shade_words([2, 4, 0, 0]), [2, 4, 1, 1]);
    assert_eq!(BSolidFx::FACTORY.shade_words([0, 0, 0, 0]), [0, 0, 0, 0], "the factory draws as a zeroed camera");
    assert!(!f.outline && f.cavity && f.curvature());
    assert_eq!(f.cavity_type, "BOTH");
    let w = f.fx_words(2.0);
    assert_eq!(w[0], 0.0);
    assert!((w[1] - 0.125).abs() < 1e-6 && (w[2] - 2.8).abs() < 1e-5, "`0.5 / 2²`, `0.7 / 0.5²`: {w:?}");
    assert_eq!(w[3], 2.0, "`U.pixelsize`");
    assert!((f.outline_words()[0] - 1.0).abs() < 1e-6 && (f.outline_words()[1] - 0.2140).abs() < 1e-3, "sRGB to linear");
    s.set(View3dKey::Shade("cavity_type"), "WORLD");
    let f = BSolidFx::now(&s);
    assert!(!f.curvature(), "SSAO alone is not the curvature effect");
    assert!(f.ssao());
    assert_eq!(f.fx_words(1.0), [0.0, 0.0, 0.0, 1.0], "no curvature words");
    assert_eq!(f.ssao_words(8), [16, 8, 0, 0], "the startup file's 16 samples over the 8 anti-aliasing samples");
    assert_eq!(f.cavity_words(), [1.0, 1.0, 1.0, 0.2]);
    BSolidFx::seed_ssao(&mut s, &crate::phiphx::scene::SceneSsao { samples: 32, distance: 0.5, attenuation: 2.0 });
    let f = BSolidFx::now(&s);
    assert_eq!((f.ssao_samples, f.ssao_distance, f.ssao_attenuation), (32, 0.5, 2.0));
    assert_eq!(f.cavity_words(), [1.0, 1.0, 2.0, 0.5]);
    s.set(View3dKey::Shade("show_cavity"), V_FALSE);
    assert_eq!(BSolidFx::now(&s).ssao_words(8)[0], 0, "off: no samples");
    s.set(View3dKey::Shade("show_cavity"), V_TRUE);
    s.set(View3dKey::Shade("cavity_type"), "SCREEN");
    assert_eq!(BSolidFx::now(&s).ssao_words(8)[0], 0, "Screen alone is not SSAO");
}

#[test]
fn the_shadow_words_turn_the_panels_direction_as_workbench_does() {
    let mut s = BState::new();
    let f = BSolidFx::now(&s);
    assert!(!f.shadow && f.shadow_intensity == 0.5, "the startup file: off, half");
    assert_eq!(f.shadow_words()[3], 0.0, "off: no intensity");
    let w = f.shadow_fade_words();
    assert!((w[0] - 0.1).abs() < 1e-6 && (w[1] - (1.0 - 0.0001 * 0.9)).abs() < 1e-6, "{w:?}");
    BSolidFx::seed(&mut s, &SceneSolidFx { shadow: true, shadow_intensity: 0.8, ..SceneSolidFx { outline: true, outline_color: [0.0; 3], cavity: false, cavity_type: "SCREEN", cavity_valley_factor: 1.0, cavity_ridge_factor: 1.0, curvature_ridge_factor: 1.0, curvature_valley_factor: 1.0, shadow: false, shadow_intensity: 0.5, backface_culling: false, specular: true } });
    BSolidFx::seed_shadow(&mut s, &crate::phiphx::scene::SceneShadow { light_direction: [0.0, 1.0, 0.0], shift: 0.3, focus: 0.5 });
    let f = BSolidFx::now(&s);
    // The panel's +y is the world's −z: light travelling straight down.
    let w = f.shadow_words();
    assert!((w[0]).abs() < 1e-6 && (w[1]).abs() < 1e-6 && (w[2] + 1.0).abs() < 1e-6 && w[3] == 0.8, "{w:?}");
    let w = f.shadow_fade_words();
    assert!((w[0] - 0.3).abs() < 1e-6 && (w[1] - 0.65).abs() < 1e-6, "`1 − 0.5 · 0.7`: {w:?}");
    s.set(View3dKey::Shade("shadow_shift"), "1");
    assert!(BSolidFx::now(&s).shadow_fade_words()[1] > 1.0, "a shift of 1 leaves the step a width");
}
