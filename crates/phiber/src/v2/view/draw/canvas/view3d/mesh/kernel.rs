//! Pure projection + shading helpers for the view3d mesh renderer.
//!
//! These small, stateless kernels are exercised by `BMesh::build` for every
//! primitive and every loaded object. Splitting them out keeps `mesh.rs`
//! itself focused on the two `BMesh` builders and under the 200-line focused
//! renderer cap.

use super::BTriangle;
use crate::phiphx::scene::SceneNormals;
use crate::v2::scene::{MESH_SCALE, project3_full};
use gpui::{Hsla, hsla};

/// Project a primitive-local vertex onto the screen, honouring yaw/pitch/pan
/// and zoom. `(x, y)` is the object's ground position, `vertex` is unit-local.
pub(super) fn primitive_vertex(
    vertex: (f32, f32, f32),
    x: f32,
    y: f32,
    yaw: f32,
    pitch: f32,
    pan: (f32, f32),
    zoom: f32,
) -> (f32, f32, f32) {
    project3_full(
        x + vertex.0 * MESH_SCALE,
        vertex.1 * MESH_SCALE,
        y + vertex.2 * MESH_SCALE,
        yaw,
        pitch,
        pan,
        zoom,
    )
}

/// Assemble a `BTriangle` from three projected `(x, y, depth)` corners, the
/// normal at each corner, and the surface colour before lighting.
pub(super) fn triangle_from(
    points: [(f32, f32, f32); 3],
    normals: [(f32, f32, f32); 3],
    base: Hsla,
    surface: super::BSurface,
    texture: Option<super::BTexture>,
) -> BTriangle {
    BTriangle {
        points,
        normals,
        base,
        roughness: surface.roughness,
        metallic: surface.metallic,
        texture,
    }
}

/// The normal at one corner of a loaded mesh's face. Per-vertex normals are
/// looked up per corner, so the rasteriser has three to interpolate; per-face
/// normals give the same answer for all three, which is a flat face and
/// correct. Falls back to +Y when the buffer is short.
pub(super) fn loaded_normal(
    normals: &SceneNormals,
    vertex: usize,
    face: usize,
) -> (f32, f32, f32) {
    let normal = match normals {
        SceneNormals::PerVertex(normals) => normals.get(vertex),
        SceneNormals::PerFace(normals) => normals.get(face),
    };
    normal
        .map(|normal| (normal[0], normal[1], normal[2]))
        .unwrap_or((0.0, 1.0, 0.0))
}

/// Shade `base` by Blender's Solid-mode studio rig, which is fixed to the
/// camera: `normal` is projector-space and is rotated into view space first,
/// so a surface facing the viewer is lit from any orbit angle.
///
/// The rig is `get_world_lighting` (`mesh::studio`), ported from
/// `BKE_studiolight_default` and `workbench_world_light_lib.glsl`: four
/// directional lights with wrapped terminators, plus Blinn specular split by
/// the material's roughness and metallic. It replaced a single hand-tuned
/// Lambert term, which is why surfaces used to fall to flat black as they
/// turned away instead of rolling off.
///
/// The rig works in linear RGB, so the surface colour goes in as RGB and the
/// result comes back as one — collapsing to a single intensity would throw
/// away the highlight's own tint, which on a metal is most of the effect.
pub(super) fn lit(
    base: Hsla,
    roughness: f32,
    metallic: f32,
    normal: (f32, f32, f32),
    yaw: f32,
    pitch: f32,
) -> Hsla {
    let n = crate::v2::scene::camera::to_view(normal, yaw, pitch);
    let rgb: gpui::Rgba = base.into();
    let lit = super::studio::lighting([rgb.r, rgb.g, rgb.b], roughness, metallic, n);
    // Blender renders the rig through the Standard view transform — the sRGB
    // transfer curve — so the same happens here. A 1.7 gain used to stand in
    // for it and left every midtone dark.
    rgba_to_hsla([srgb(lit[0]), srgb(lit[1]), srgb(lit[2]), base.a])
}

/// Blender's "Standard" view transform: scene-linear to sRGB display.
fn srgb(v: f32) -> f32 {
    let x = v.clamp(0.0, 1.0);
    if x <= 0.0031308 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 }
}

/// Convert a material's linear RGBA (`[r, g, b, a]` in 0..1) into the `Hsla`
/// the shader pipeline expects. A straightforward RGB→HSL conversion; the
/// per-triangle `lit` call then applies the directional-light term on top so
/// a material's flat base colour still reads as a shaded surface.
/// A linear RGBA sample as the `Hsla` the shading pipeline works in.
pub(super) fn linear_rgba(rgba: [f32; 4]) -> Hsla {
    rgba_to_hsla(rgba)
}

fn rgba_to_hsla(rgba: [f32; 4]) -> Hsla {
    let (r, g, b, a) = (
        rgba[0].clamp(0.0, 1.0),
        rgba[1].clamp(0.0, 1.0),
        rgba[2].clamp(0.0, 1.0),
        rgba[3].clamp(0.0, 1.0),
    );
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let delta = max - min;
    let s = if delta <= 1e-6 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs()).max(1e-6)
    };
    let mut h = if delta <= 1e-6 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    if h < 0.0 {
        h += 360.0;
    }
    // gpui `hsla` expects hue in 0..1 turns, not degrees.
    hsla(h / 360.0, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0), a)
}

/// Resolve the base fill for triangle `face` of a loaded object: its material
/// base colour when present (converted to `Hsla`), else the neutral `fallback`
/// loaded colour. `lit` applies the directional-light term on top.
pub(super) fn loaded_fill(
    object: &crate::v2::scene::BObject3D,
    face: usize,
    fallback: Hsla,
    shading: crate::v2::scene::BShading,
) -> Hsla {
    object
        .triangle_color(face, shading)
        .map(rgba_to_hsla)
        .unwrap_or(fallback)
}

