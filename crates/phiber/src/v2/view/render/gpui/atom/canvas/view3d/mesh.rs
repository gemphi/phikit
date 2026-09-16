//! Projection and shading of scene meshes.

mod hull;
mod kernel;
mod paint;

use crate::phiphx::services::scene::SceneMesh;
use crate::phiphx::spatial::SpatialObject3D;
use crate::v2::scene::{BObject3D, BShape, fit_vertices, project3_full};
use gpui::{Bounds, Hsla, Pixels, Window};
use kernel::{
    lit, loaded_fill, loaded_normal, primitive_vertex, sort_triangles, triangle_from,
};

#[derive(Clone, Copy)]
pub(super) struct BMeshPalette {
    pub cube: Hsla,
    pub sphere: Hsla,
    pub gem: Hsla,
    pub loaded: Hsla,
    pub accent: Hsla,
}

#[derive(Clone)]
pub(super) struct BTriangle {
    pub points: [(f32, f32); 3],
    pub depth: f32,
    pub fill: Hsla,
}

#[derive(Clone)]
pub(super) struct BMesh {
    pub triangles: Vec<BTriangle>,
    pub outline: Option<Vec<(f32, f32)>>,
    pub outline_color: Hsla,
    pub depth: f32,
}

impl BMesh {
    pub fn paint(bounds: Bounds<Pixels>, meshes: &[Self], outline_width: f32, window: &mut Window) {
        paint::meshes(bounds, meshes, outline_width, window);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build(
        object: &BObject3D,
        yaw: f32,
        pitch: f32,
        pan: (f32, f32),
        zoom: f32,
        palette: BMeshPalette,
        active: bool,
    ) -> Self {
        match object {
            BObject3D::Primitive { shape, x, y, .. } => {
                Self::primitive(*shape, *x, *y, yaw, pitch, pan, zoom, palette, active)
            }
            loaded @ BObject3D::Loaded {
                mesh,
                location,
                scale,
                ..
            } => Self::loaded(
                loaded, mesh, *location, *scale, yaw, pitch, pan, zoom, palette, active,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn primitive(
        shape: BShape,
        x: f32,
        y: f32,
        yaw: f32,
        pitch: f32,
        pan: (f32, f32),
        zoom: f32,
        palette: BMeshPalette,
        active: bool,
    ) -> Self {
        let (vertices, indices, normals) = SpatialObject3D::buffers(shape.mesh_type());
        let base = match shape {
            BShape::Cube => palette.cube,
            BShape::Sphere => palette.sphere,
            BShape::Gem => palette.gem,
        };
        let mut triangles = Vec::with_capacity(indices.len() / 3);
        for triangle in indices.chunks_exact(3) {
            let indices = [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ];
            let projected =
                indices.map(|index| primitive_vertex(vertices[index], x, y, yaw, pitch, pan, zoom));
            let edge_a = (
                projected[1].0 - projected[0].0,
                projected[1].1 - projected[0].1,
            );
            let edge_b = (
                projected[2].0 - projected[0].0,
                projected[2].1 - projected[0].1,
            );
            if edge_a.0 * edge_b.1 - edge_a.1 * edge_b.0 >= 0.0 {
                continue;
            }
            let normal = normals.get(indices[0]).copied().unwrap_or((0.0, 1.0, 0.0));
            triangles.push(triangle_from(projected, lit(base, normal)));
        }
        sort_triangles(&mut triangles);
        let outline = active.then(|| {
            hull::convex(
                vertices
                    .iter()
                    .map(|&vertex| {
                        let point = primitive_vertex(vertex, x, y, yaw, pitch, pan, zoom);
                        (point.0, point.1)
                    })
                    .collect(),
            )
        });
        let (_, _, depth) = project3_full(x, 0.0, y, yaw, pitch, pan, zoom);
        Self {
            triangles,
            outline,
            outline_color: palette.accent,
            depth,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn loaded(
        object: &BObject3D,
        mesh: &SceneMesh,
        location: [f32; 3],
        scale: [f32; 3],
        yaw: f32,
        pitch: f32,
        pan: (f32, f32),
        zoom: f32,
        palette: BMeshPalette,
        active: bool,
    ) -> Self {
        // Max-axis scale so a non-uniform blend scale preserves the largest
        // dimension on screen without shearing the mesh.
        let s = scale[0]
            .abs()
            .max(scale[1].abs())
            .max(scale[2].abs())
            .max(1e-6);
        // Blender is Z-up: `location` is (x, y, z_up). Our projector expects
        // (x, up, z). Remap and downscale by MESH_SCALE so 1 blend unit maps
        // to one MESH_SCALE step — the same footprint one primitive occupies.
        let (lx, lup, lz) = (
            location[0] * crate::v2::scene::MESH_SCALE,
            location[2] * crate::v2::scene::MESH_SCALE,
            location[1] * crate::v2::scene::MESH_SCALE,
        );
        let projected: Vec<_> = fit_vertices(&mesh.vertices)
            .iter()
            .map(|vertex| {
                project3_full(
                    vertex[0] * s + lx,
                    vertex[1] * s + lup,
                    vertex[2] * s + lz,
                    yaw,
                    pitch,
                    pan,
                    zoom,
                )
            })
            .collect();
        let mut triangles = Vec::with_capacity(mesh.indices.len() / 3);
        for (face, triangle) in mesh.indices.chunks_exact(3).enumerate() {
            let indices = [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ];
            if indices.iter().any(|&index| index >= projected.len()) {
                continue;
            }
            let points = indices.map(|index| projected[index]);
            let normal = loaded_normal(&mesh.normals, indices[0], face);
            // Per-triangle material colour (else neutral); `lit` shades it.
            let base = loaded_fill(object, face, palette.loaded);
            triangles.push(triangle_from(points, lit(base, normal)));
        }
        sort_triangles(&mut triangles);
        let outline = active
            .then(|| hull::convex(projected.iter().map(|point| (point.0, point.1)).collect()));
        // Depth uses the object's world-position ground point so multiple
        // loaded objects paint back-to-front consistently with each other.
        let (_, _, depth) = project3_full(lx, lup, lz, yaw, pitch, pan, zoom);
        Self {
            triangles,
            outline,
            outline_color: palette.accent,
            depth,
        }
    }
}

pub(super) fn name(object: &BObject3D) -> &str {
    match object {
        BObject3D::Primitive { name, .. } | BObject3D::Loaded { name, .. } => name,
    }
}

pub(super) fn ground(object: &BObject3D) -> (f32, f32) {
    match object {
        BObject3D::Primitive { x, y, .. } => (*x, *y),
        // Loaded objects use their Blender location remapped from Z-up to our
        // (x, up, z) coord system — same mapping mesh::loaded applies. Uses
        // MESH_SCALE so hit labels sit next to the projected mesh, not off
        // in world-space.
        BObject3D::Loaded { location, .. } => (
            location[0] * crate::v2::scene::MESH_SCALE,
            location[1] * crate::v2::scene::MESH_SCALE,
        ),
    }
}


