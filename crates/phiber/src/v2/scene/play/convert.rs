//! phiplay's generated geometry as Phiber's scene: a mesh's positions,
//! smooth normals and UVs; a material's Principled values with its image as
//! the base colour (and the emission, when it glows); a part as an object
//! standing where the part says.

use crate::phiphx::scene::{SceneBsdf, SceneCorners, SceneMap, SceneMaps, SceneMaterial, SceneMesh, SceneNormals, SceneObject, SceneTexture};
use phiplay::build::{BImage, BMat, BMesh, BPart};
use std::sync::Arc;

pub fn mesh(m: &BMesh) -> SceneMesh {
    SceneMesh { vertices: m.positions.clone(), indices: m.indices.clone(), normals: SceneNormals::PerVertex(m.normals.clone()), corners: SceneCorners::default() }
}

/// An empty mesh: what an object is made with before its shared mesh is
/// put in.
fn placeholder() -> SceneMesh {
    SceneMesh { vertices: Vec::new(), indices: Vec::new(), normals: SceneNormals::PerVertex(Vec::new()), corners: SceneCorners::default() }
}

pub fn texture(image: &BImage) -> SceneTexture {
    SceneTexture { width: image.width, height: image.height, rgba: Arc::new(image.rgba.clone()) }
}

pub fn material(m: &BMat) -> SceneMaterial {
    let [r, g, b] = m.base.linear();
    let base = [r, g, b, 1.0];
    let image = m.image.as_deref().map(texture);
    let mut bsdf = SceneBsdf { base, roughness: m.roughness, metallic: m.metallic, ..Default::default() };
    if m.strength > 0.0 {
        bsdf.emission = m.emission.linear();
        bsdf.emission_strength = m.strength;
    }
    // A glowing image lights in its own colours, as the LED boards do.
    let emit = image.clone().filter(|_| m.strength > 0.0).map(|image| SceneMap { key: m.name.clone(), image: Some(image) });
    SceneMaterial {
        name: m.name.clone(),
        // Textured: the image carries the colour, the tint over it is white.
        base_color: if image.is_some() { [1.0; 4] } else { base },
        viewport_color: base,
        roughness: m.roughness,
        metallic: m.metallic,
        base_color_texture: image.as_ref().map(|_| m.name.clone()),
        texture: image,
        bsdf,
        maps: SceneMaps { emit, ..Default::default() },
        ..SceneMaterial::neutral()
    }
}

/// A part as an object of its own, at the part's matrix.
pub fn object(part: &BPart) -> SceneObject {
    shared(&part.name, Arc::new(mesh(&part.mesh)), Arc::new(part.mesh.uvs.clone()), material(&part.material), part.at)
}

/// An object on a mesh it shares with others (a fan, a digit's faces).
pub fn shared(name: &str, mesh: Arc<SceneMesh>, uvs: Arc<Vec<[f32; 2]>>, material: SceneMaterial, at: glam::Mat4) -> SceneObject {
    let (scale, _, location) = at.to_scale_rotation_translation();
    SceneObject {
        mesh,
        uvs,
        materials: vec![material],
        matrix_world: at.to_cols_array_2d(),
        location: location.into(),
        scale: scale.into(),
        ..SceneObject::identity(name, placeholder())
    }
}
