//! Scene identity: when this changes, the tracer rebuilds.

use super::super::mesh::BMeshPalette;
use crate::v2::scene::BObject3D;
use gpui::Hsla;
use std::hash::{Hash, Hasher};

fn rgba(c: Hsla) -> [f32; 4] {
    let c = c.to_rgb();
    [c.r, c.g, c.b, c.a]
}

/// Identity of what the tracer must build (objects, transforms, sizes, colours).
pub fn key(scene: &[BObject3D], palette: &BMeshPalette) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for o in scene {
        match o {
            BObject3D::Primitive { name, shape, x, y } => (name, *shape as u8, x.to_bits(), y.to_bits()).hash(&mut h),
            BObject3D::Loaded { name, mesh, matrix_world, materials, .. } => {
                (name, mesh.vertices.len(), mesh.indices.len(), materials.len()).hash(&mut h);
                matrix_world.iter().flatten().for_each(|v| v.to_bits().hash(&mut h));
            }
        }
    }
    for c in [palette.cube, palette.sphere, palette.gem, palette.loaded] {
        rgba(c).map(f32::to_bits).hash(&mut h);
    }
    // Solid builds from viewport colours with no images; Material builds from
    // Base Color with the atlas. They are different scenes to the GPU, so
    // switching shading rebuilds rather than showing one mode's data in the
    // other.
    palette.shading.uses_viewport_color().hash(&mut h);
    h.finish()
}

