//! Projection and shading of scene meshes.

pub mod extreme;
mod hull;
mod triangle;
pub(super) mod kernel;
mod raster;
mod studio;
mod paint;
pub use self::paint::rasterised;

pub(super) use self::triangle::{BMesh, BMeshPalette, BSurface, BTexture, BTriangle};

use crate::phiphx::scene::SceneMesh;
use crate::phiphx::spatial::SpatialObject3D;
use crate::v2::scene::{BObject3D, BShape, project3_full};
use self::kernel::{
    loaded_fill, loaded_normal, primitive_vertex, triangle_from,
};

impl BMesh {
    /// Only the selection outline of `object` — no triangles — for when the
    /// GPU draws the surfaces and the CPU paints just the active object's
    /// hull over them (Blender's `OVERLAY_outline`, in the accent).
    pub fn outline_only(object: &BObject3D, yaw: f32, pitch: f32, pan: (f32, f32), zoom: f32, palette: BMeshPalette) -> Self {
        let points: Vec<(f32, f32)> = match object {
            BObject3D::Primitive { shape, x, y, .. } => {
                let (vertices, ..) = SpatialObject3D::buffers(shape.mesh_type());
                vertices
                    .iter()
                    .map(|&vertex| {
                        let point = primitive_vertex(vertex, *x, *y, yaw, pitch, pan, zoom);
                        (point.0, point.1)
                    })
                    .collect()
            }
            BObject3D::Loaded { mesh, matrix_world, .. } => {
                let m = glam::Mat4::from_cols_array_2d(matrix_world);
                let ms = crate::v2::scene::MESH_SCALE;
                // Its extreme points, not every vertex (`extreme.rs`).
                self::extreme::of(mesh)
                    .iter()
                    .map(|vertex| {
                        let w = m.transform_point3(glam::Vec3::from(*vertex));
                        let point = project3_full(w.x * ms, w.z * ms, w.y * ms, yaw, pitch, pan, zoom);
                        (point.0, point.1)
                    })
                    .collect()
            }
        };
        Self { triangles: Vec::new(), outline: Some(self::hull::convex(points)), outline_color: palette.accent }
    }

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
                mesh, matrix_world, ..
            } => Self::loaded(
                loaded, mesh, matrix_world, yaw, pitch, pan, zoom, palette, active,
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
        // The palette carries three shades, from when there were three
        // primitives; the rest take the one their silhouette is nearest.
        let base = match shape {
            BShape::Sphere | BShape::IcoSphere | BShape::Metaball => palette.sphere,
            BShape::Gem | BShape::Cone | BShape::Torus => palette.gem,
            _ => palette.cube,
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
            let corner_normals =
                indices.map(|i| normals.get(i).copied().unwrap_or((0.0, 1.0, 0.0)));
            // A primitive carries no UVs, so it is never textured, and takes
            // the default dielectric surface.
            triangles.push(triangle_from(
                projected,
                corner_normals,
                base,
                BSurface::DEFAULT,
                None,
            ));
        }
        let outline = active.then(|| {
            self::hull::convex(
                vertices
                    .iter()
                    .map(|&vertex| {
                        let point = primitive_vertex(vertex, x, y, yaw, pitch, pan, zoom);
                        (point.0, point.1)
                    })
                    .collect(),
            )
        });
        Self {
            triangles,
            outline,
            outline_color: palette.accent,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn loaded(
        object: &BObject3D,
        mesh: &SceneMesh,
        matrix_world: &[[f32; 4]; 4],
        yaw: f32,
        pitch: f32,
        pan: (f32, f32),
        zoom: f32,
        palette: BMeshPalette,
        active: bool,
    ) -> Self {
        // Blender parity: vertices are object-local in Blender units and are
        // placed by the object's *world matrix* (loc · rot · scale · parent
        // chain), exactly as `BKE_object_where_is_calc` positions them. No
        // per-mesh normalisation — relative sizes and offsets between objects
        // are preserved. Blender is Z-up; the projector expects (x, up, z),
        // so world (x, y, z) maps to (x, z, y), scaled by MESH_SCALE.
        let m = glam::Mat4::from_cols_array_2d(matrix_world);
        let ms = crate::v2::scene::MESH_SCALE;
        let to_view = |v: glam::Vec3| (v.x * ms, v.z * ms, v.y * ms);
        let projected: Vec<_> = mesh
            .vertices
            .iter()
            .map(|vertex| {
                let w = m.transform_point3(glam::Vec3::from(*vertex));
                let (x, up, z) = to_view(w);
                project3_full(x, up, z, yaw, pitch, pan, zoom)
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
            // Normals are object-local, Blender Z-up: rotate into world
            // space and remap to the projector's (x, up, z) before lighting.
            // One per corner, so a smooth surface interpolates instead of
            // taking whatever the triangle's first corner happened to face.
            let corner_normals = indices.map(|index| {
                let local = loaded_normal(&mesh.normals, index, face);
                let w = m
                    .transform_vector3(glam::Vec3::new(local.0, local.1, local.2))
                    .normalize_or_zero();
                (w.x, w.z, w.y)
            });
            // Per-face material colour; the rasteriser lights it per pixel.
            let base = loaded_fill(object, face, palette.loaded, palette.shading);
            let texture = object
                .triangle_texture(face, palette.shading)
                .zip(object.triangle_uvs(face))
                .map(|(image, uvs)| BTexture { uvs, image: image.clone() });
            let (roughness, metallic) = object.triangle_surface(face);
            let surface = BSurface { roughness, metallic };
            triangles.push(triangle_from(
                points,
                corner_normals,
                base,
                surface,
                texture,
            ));
        }
        let outline = active
            .then(|| self::hull::convex(projected.iter().map(|point| (point.0, point.1)).collect()));
        Self {
            triangles,
            outline,
            outline_color: palette.accent,
        }
    }
}

