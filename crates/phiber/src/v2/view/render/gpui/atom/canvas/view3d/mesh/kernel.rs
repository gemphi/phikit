//! Pure projection + shading helpers for the view3d mesh renderer.
//!
//! These small, stateless kernels are exercised by `BMesh::build` for every
//! primitive and every loaded object. Splitting them out keeps `mesh.rs`
//! itself focused on the two `BMesh` builders and under the 200-line focused
//! renderer cap.

use super::BTriangle;
use crate::phiphx::services::scene::SceneNormals;
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

/// Assemble a `BTriangle` from three projected `(x, y, depth)` points and a
/// pre-computed fill colour. Depth is the mean of the three vertex depths
/// (good enough for painter-sort within a mesh).
pub(super) fn triangle_from(points: [(f32, f32, f32); 3], fill: Hsla) -> BTriangle {
    BTriangle {
        points: points.map(|point| (point.0, point.1)),
        depth: (points[0].2 + points[1].2 + points[2].2) / 3.0,
        fill,
    }
}

/// Look up the appropriate normal for a loaded mesh's face. Per-vertex normals
/// use the first vertex of the triangle; per-face use the face index. Falls
/// back to +Y when the buffer is short.
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

/// Shade `base` by the diffuse contribution of a single directional light.
/// Lighting constants are 3D scene mechanics, not UI style values.
pub(super) fn lit(base: Hsla, normal: (f32, f32, f32)) -> Hsla {
    let length = (normal.0 * normal.0 + normal.1 * normal.1 + normal.2 * normal.2)
        .sqrt()
        .max(1e-8);
    let dot =
        ((normal.0 / length) * 0.35 + (normal.1 / length) * 0.80 + (normal.2 / length) * 0.48)
            .clamp(0.0, 1.0);
    let light = 0.28 + 0.72 * dot;
    hsla(base.h, base.s, (base.l * light).clamp(0.08, 0.92), base.a)
}

/// Convert a material's linear RGBA (`[r, g, b, a]` in 0..1) into the `Hsla`
/// the shader pipeline expects. A straightforward RGB→HSL conversion; the
/// per-triangle `lit` call then applies the directional-light term on top so
/// a material's flat base colour still reads as a shaded surface.
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
) -> Hsla {
    object
        .triangle_base_color(face)
        .map(rgba_to_hsla)
        .unwrap_or(fallback)
}

/// Painter-sort a triangle slice back-to-front by mean depth.
pub(super) fn sort_triangles(triangles: &mut [BTriangle]) {
    triangles.sort_by(|left, right| {
        right
            .depth
            .partial_cmp(&left.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
