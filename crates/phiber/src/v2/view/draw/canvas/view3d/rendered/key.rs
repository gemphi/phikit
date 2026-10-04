//! Scene identity: when it changes, the render thread takes the scene
//! again — moving the tracer when only the pose changed (the same shape,
//! `Tracer::repose`), building it again otherwise.

use super::super::mesh::BMeshPalette;
use crate::v2::scene::BObject3D;
use gpui::Hsla;
use std::hash::{Hash, Hasher};

/// The scene's two identities.
pub struct BRayKey;

impl BRayKey {
    fn rgba(c: Hsla) -> [f32; 4] {
        let c = c.to_rgb();
        [c.r, c.g, c.b, c.a]
    }

    /// Identity of what the tracer must hold (objects, transforms, sizes,
    /// colours), hashed now; the tracer takes `of_list`.
    #[cfg(test)]
    pub fn of(scene: &[BObject3D], palette: &BMeshPalette) -> u64 {
        Self::with_palette(super::super::identity::BSceneId::hash(scene), palette)
    }

    /// `of` for a kept list: the list's number is kept with it
    /// (`BSceneId`), so a frame that shows the same list hashes nothing.
    pub fn of_list(scene: &std::rc::Rc<Vec<BObject3D>>, palette: &BMeshPalette) -> u64 {
        Self::with_palette(super::super::identity::BSceneId::of(scene), palette)
    }

    /// The objects' identity `scene` with the palette's.
    fn with_palette(scene: u64, palette: &BMeshPalette) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        scene.hash(&mut h);
        Self::palette(&mut h, palette);
        h.finish()
    }

    /// What of the palette the tracer holds: the colours, and which kind of
    /// scene it builds.
    fn palette(h: &mut impl Hasher, palette: &BMeshPalette) {
        for c in [palette.cube, palette.sphere, palette.gem, palette.loaded] {
            Self::rgba(c).map(f32::to_bits).hash(h);
        }
        // Solid builds from viewport colours with no images; Material builds
        // from Base Color with the atlas — the studio light and its
        // precomputed lighting in it (`scene::build`), where Rendered holds
        // the file's world. They are different scenes to the GPU, so
        // switching shading builds again rather than showing one mode's
        // data in the other.
        palette.shading.uses_viewport_color().hash(h);
        // MatCap's image goes up with the Solid scene (`scene::build`).
        palette.matcap.hash(h);
        palette.texture.hash(h);
        (palette.shading == crate::v2::scene::BShading::Material).hash(h);
    }

    /// The scene's shape: the same objects, meshes and colours, wherever
    /// they are. Two scenes of one shape differ in pose only — a mesh its
    /// rig deforms is a new mesh each frame (the same triangles, other
    /// vertices), so the mesh itself is pose, not shape.
    pub fn shape(scene: &[BObject3D], palette: &BMeshPalette) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for o in scene {
            match o {
                BObject3D::Primitive { name, shape, .. } => (name, *shape as u8).hash(&mut h),
                BObject3D::Loaded { name, mesh, materials, .. } => {
                    (name, mesh.vertices.len(), mesh.indices.len(), materials.len()).hash(&mut h);
                }
            }
        }
        Self::palette(&mut h, palette);
        h.finish()
    }
}
