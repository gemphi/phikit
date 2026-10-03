//! Workbench's Solid effects as a file's viewport keeps them
//! (`View3DShading.flag`'s `V3D_SHADING_OBJECT_OUTLINE` and
//! `V3D_SHADING_CAVITY`, `object_outline_color`, `cavity_type`, the four
//! factors, `show_shadows` and `shadow_intensity`): the factory startup's —
//! the outline on and black, Cavity off, its type Screen, every factor 1,
//! Shadow off at intensity 0.5.

use phiphx::blend::BlendFile;
use phiphx::scene::{SceneModel, SceneShadow, SceneSolidFx, SceneSolidShade, SceneSsao};
use std::path::PathBuf;

#[test]
fn the_startup_viewport_outlines_its_objects_and_has_no_cavity() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiber/assets/blend/startup-5.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path.to_string_lossy());
    // `Scene.display`'s SSAO: 16 samples, 0.2 units, attenuation 1.
    assert_eq!(model.render.ssao, SceneSsao { samples: 16, distance: 0.2, attenuation: 1.0 });
    // `Scene.display`'s shadow: the light along (1, 1, 1), Offset 0.1, Focus 0.
    let shadow = model.render.shadow;
    assert!(shadow.light_direction.iter().all(|c| (c - 0.577_350_3).abs() < 1e-6), "{shadow:?}");
    assert_eq!((shadow.shift, shadow.focus), (0.1, 0.0));
    assert_eq!(shadow, SceneShadow { light_direction: shadow.light_direction, ..SceneShadow::default() });
    let view = model.ui.view.expect("the Layout viewport");
    assert_eq!(view.solid_fx, Some(SceneSolidFx { outline: true, outline_color: [0.0; 3], cavity: false, cavity_type: "SCREEN", cavity_valley_factor: 1.0, cavity_ridge_factor: 1.0, curvature_ridge_factor: 1.0, curvature_valley_factor: 1.0, shadow: false, shadow_intensity: 0.5 }));
    // Studio lighting, the material colour, a 0.8 grey single colour, the
    // theme background — `View3DShading`'s defaults.
    assert_eq!(view.solid_shade, Some(SceneSolidShade { light: "STUDIO", color_type: "MATERIAL", single_color: [0.8; 3], background_type: "THEME", background_color: [0.05; 3] }));
}
