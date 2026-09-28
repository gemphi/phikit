//! #26: Material Preview paints image textures (packed and `//`-relative);
//! Solid is the theme's one shade.

use phiber::phiphx::Phx;
use phiber::v2::scene::{BShading, current_scene};
use phiber::v2::state::BState;

#[test]
fn material_preview_samples_textures() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures/TEXTURED-PACKED.blend");
    let model = Phx::open(std::path::Path::new(path)).expect("fixture loads");
    let scene = current_scene(&BState::default(), Some(&model));
    let find = |n: &str| scene.iter().find(|o| matches!(o, phiber::v2::scene::BObject3D::Loaded { name, .. } if name == n)).expect(n);

    let plane = find("PackedPlane");
    let mut seen: Vec<[f32; 4]> = (0..2).filter_map(|t| plane.triangle_color(t, BShading::Material)).collect();
    seen.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // Quad split (0,1,2)+(0,2,3): centroids land in the green (bottom-right)
    // and blue (top-left) quadrants of the packed 2×2 image.
    assert_eq!(seen, vec![[0.0, 0.0, 1.0, 1.0], [0.0, 1.0, 0.0, 1.0]]);

    let cube = find("ExternalCube");
    assert_eq!(cube.triangle_color(0, BShading::Rendered), Some([1.0, 0.0, 1.0, 1.0]));
    assert_eq!(cube.triangle_color(0, BShading::Solid), None, "Solid has no colour of the object's own: the theme's one shade");
    assert_ne!(cube.triangle_color(0, BShading::Wireframe), Some([1.0, 0.0, 1.0, 1.0]), "Wireframe ignores the texture");
}
