//! X-ray as a file's viewport keeps it (`View3DShading.flag`'s
//! `V3D_SHADING_XRAY` / `_XRAY_WIREFRAME`, `xray_alpha`, `xray_alpha_wire`):
//! the factory startup's (off in Solid at 0.5, on in Wireframe at 0), and a
//! file Blender saved with every value changed (`fixtures/make_xray.py`).

use phiphx::blend::BlendFile;
use phiphx::scene::{SceneModel, SceneXray};
use std::path::PathBuf;

fn xray(path: &str) -> Option<SceneXray> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy()).ui.view.and_then(|v| v.xray)
}

#[test]
fn the_viewport_keeps_its_xray() {
    assert_eq!(xray("../phiber/assets/blend/startup-5.blend"), Some(SceneXray { solid: false, alpha: 0.5, wireframe: true, alpha_wireframe: 0.0 }));
    let saved = xray("tests/fixtures/xray.blend").expect("the Layout viewport");
    assert!(saved.solid && !saved.wireframe, "{saved:?}");
    assert!((saved.alpha - 0.3).abs() < 1e-6 && (saved.alpha_wireframe - 0.6).abs() < 1e-6, "{saved:?}");
}
