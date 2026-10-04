//! Color ▸ Attribute's and ▸ Texture's plumbing: a mesh's corner colours
//! (`SceneCorners::colors`) reach the tracer's vertices as sRGB bytes
//! (`Scene::paint`), and Solid uploads a material's base-colour image only
//! under Texture.

use super::scene;
use crate::v2::scene::BShading;
use gpui::white;

#[test]
fn a_meshs_corner_colours_paint_the_tracers_vertices() {
    use crate::phiphx::scene::{SceneCorners, SceneMesh, SceneNormals};
    use crate::v2::scene::BObject3D;
    let grey = white();
    let palette = super::BMeshPalette { shading: BShading::Solid, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false };
    let tri = |colors: Vec<[f32; 3]>| BObject3D::Loaded {
        name: "Painted".into(),
        mesh: std::sync::Arc::new(SceneMesh {
            vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
            corners: SceneCorners { ids: Vec::new(), normals: Vec::new(), colors },
        }),
        location: [0.0; 3],
        scale: [1.0; 3],
        matrix_world: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
        materials: Vec::new(),
        material_indices: Default::default(),
        uvs: Default::default(),
        color: [1.0; 4],
        shadow: Default::default(),
    };
    let plain = scene::build(&[tri(Vec::new())], &palette);
    assert!(plain.colors.is_empty(), "no attribute, no colours uploaded");
    let painted = scene::build(&[tri(vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])], &palette);
    // sRGB bytes: linear 1 is 255, linear 0 is 0; alpha 255.
    assert_eq!(painted.colors, vec![0xff00_00ff, 0xff00_ff00, 0xffff_0000]);
}

#[test]
fn solid_uploads_the_base_image_under_texture_alone() {
    use crate::phiphx::phx::Phx;
    use crate::v2::scene::{scene_source, set_loaded_scene};
    use crate::v2::state::BState;
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/TEXTURED-PACKED.blend");
    let model = Phx::open(&path).expect("open");
    set_loaded_scene(Some(model));
    let objects = scene_source(&BState::default());
    let grey = white();
    let palette = |texture: bool| super::BMeshPalette { shading: BShading::Solid, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture };
    assert!(scene::build(&objects, &palette(false)).atlas.is_empty(), "Material: Solid reads no image");
    let textured = scene::build(&objects, &palette(true));
    assert!(!textured.atlas.is_empty(), "Texture: the packed image goes up");
    assert!(textured.atlas.bytes() < 8 << 20, "the base image alone, not the maps: {} bytes", textured.atlas.bytes());
    set_loaded_scene(None);
}

/// Viewport Display ▸ Shadow off (`OB_DRAW_NO_SHADOW_CAST`) stops the
/// object's shadow under Solid; the ray-traced modes read Visibility ▸ Ray
/// ▸ Shadow (`OB_HIDE_SHADOW`) instead.
#[test]
fn an_objects_shadow_toggles_follow_the_mode() {
    use crate::phiphx::blend::BlendShadowCast;
    use crate::phiphx::scene::{SceneCorners, SceneMesh, SceneNormals};
    use crate::v2::scene::BObject3D;
    let grey = white();
    let palette = |shading| super::BMeshPalette { shading, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false };
    let tri = |shadow: BlendShadowCast| BObject3D::Loaded {
        name: "Shy".into(),
        mesh: std::sync::Arc::new(SceneMesh {
            vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
            corners: SceneCorners::default(),
        }),
        location: [0.0; 3],
        scale: [1.0; 3],
        matrix_world: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
        materials: Vec::new(),
        material_indices: Default::default(),
        uvs: Default::default(),
        color: [1.0; 4],
        shadow,
    };
    let casts = |shading, shadow| scene::build(&[tri(shadow)], &palette(shading)).insts[0].casts_shadow;
    let display_off = BlendShadowCast { display: false, ray: true };
    let ray_off = BlendShadowCast { display: true, ray: false };
    assert!(casts(BShading::Solid, BlendShadowCast::default()));
    assert!(!casts(BShading::Solid, display_off), "Solid: Viewport Display's toggle");
    assert!(casts(BShading::Solid, ray_off));
    assert!(!casts(BShading::Wireframe, display_off));
    assert!(casts(BShading::Rendered, display_off), "Rendered: Ray Visibility's toggle");
    assert!(!casts(BShading::Rendered, ray_off));
    assert!(!casts(BShading::Material, ray_off));
}
