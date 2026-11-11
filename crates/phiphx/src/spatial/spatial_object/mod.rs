//! Native Rust 3D Spatial Object & Geometry Mesh Buffer Representation

mod round;
mod solid;
use crate::spatial::mesh_converter::PhiQuiMeshType;
use crate::spatial::spatial_animator::SpatialTransform3D;

/// Compute a smooth per-vertex normal buffer from a triangle-indexed mesh.
///
/// Each triangle's geometric (face) normal is accumulated onto its three
/// vertices, then every vertex normal is normalized. Returns one normal per
/// vertex (same length as `verts`), matching the convention used by the other
/// primitive builders in this module.
fn face_normals(verts: &[(f32, f32, f32)], idx: &[u32]) -> Vec<(f32, f32, f32)> {
    let mut acc = vec![(0.0f32, 0.0f32, 0.0f32); verts.len()];
    for tri in idx.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let (pa, pb, pc) = (verts[a], verts[b], verts[c]);
        let e1 = (pb.0 - pa.0, pb.1 - pa.1, pb.2 - pa.2);
        let e2 = (pc.0 - pa.0, pc.1 - pa.1, pc.2 - pa.2);
        // Cross product e1 × e2 — the (unnormalized) face normal.
        let n = (
            e1.1 * e2.2 - e1.2 * e2.1,
            e1.2 * e2.0 - e1.0 * e2.2,
            e1.0 * e2.1 - e1.1 * e2.0,
        );
        for &v in &[a, b, c] {
            acc[v].0 += n.0;
            acc[v].1 += n.1;
            acc[v].2 += n.2;
        }
    }
    for n in acc.iter_mut() {
        let len = (n.0 * n.0 + n.1 * n.1 + n.2 * n.2).sqrt();
        if len > f32::EPSILON {
            n.0 /= len;
            n.1 /= len;
            n.2 /= len;
        }
    }
    acc
}

#[derive(Debug, Clone)]
pub struct SpatialObject3D {
    pub name: String,
    pub mesh_type: PhiQuiMeshType,
    pub transform: SpatialTransform3D,
    pub vertices: Vec<(f32, f32, f32)>,
    pub indices: Vec<u32>,
    pub normals: Vec<(f32, f32, f32)>,
    pub color: (f32, f32, f32, f32),
}

impl SpatialObject3D {
    pub fn new(name: impl Into<String>, mesh_type: PhiQuiMeshType) -> Self {
        let (vertices, indices, normals) = Self::buffers(mesh_type);
        Self {
            name: name.into(),
            mesh_type,
            transform: SpatialTransform3D::default(),
            vertices,
            indices,
            normals,
            color: (0.65, 0.33, 0.96, 1.0),
        }
    }

    pub fn with_transform(mut self, transform: SpatialTransform3D) -> Self {
        self.transform = transform;
        self
    }

    /// Vertex / triangle / normal buffers for a primitive. Cube is unit,
    /// sphere is a UV sphere, icosahedron is a normalized gem.
    pub fn buffers(
        mesh_type: PhiQuiMeshType,
    ) -> (Vec<(f32, f32, f32)>, Vec<u32>, Vec<(f32, f32, f32)>) {
        match mesh_type {
            PhiQuiMeshType::Cube => Self::build_cube_mesh(),
            PhiQuiMeshType::Sphere => Self::build_sphere_mesh(),
            PhiQuiMeshType::Torus => Self::build_torus_mesh(),
            PhiQuiMeshType::Suzanne => Self::build_suzanne_mesh(),
            PhiQuiMeshType::Cylinder => Self::build_cylinder_mesh(),
            PhiQuiMeshType::Icosahedron => Self::build_icosahedron_mesh(),
            PhiQuiMeshType::Plane => Self::build_plane_mesh(),
            PhiQuiMeshType::Circle => Self::build_circle_mesh(),
            PhiQuiMeshType::Cone => Self::build_cone_mesh(),
            PhiQuiMeshType::IcoSphere => Self::build_ico_sphere_mesh(),
            PhiQuiMeshType::Grid => Self::build_grid_mesh(),
            PhiQuiMeshType::Metaball => Self::build_metaball_mesh(),
            // Blender's new Text object reads "Text", extruded by 0 — a
            // flat face. A visible depth is more use as a primitive.
            PhiQuiMeshType::Text => crate::spatial::text::mesh("Text", 0.22),
        }
    }

}

#[cfg(test)]
#[path = "../test/spatial_object.rs"]
mod spatial_object_test;
