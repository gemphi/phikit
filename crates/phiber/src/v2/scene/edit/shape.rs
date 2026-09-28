//! The primitives `Add ▸ Mesh` offers — Blender's `mesh.primitive_*_add`
//! — each a mesh built once from the spatial buffers and named as
//! `BLI_uniquename` names a new object.

use std::collections::HashMap;
use std::sync::Arc;

use crate::phiphx::scene::{SceneMesh, SceneNormals};
use crate::phiphx::spatial::{PhiQuiMeshType, SpatialObject3D};

/// The primitives `Add ▸ Mesh` offers, as Blender lists them.
pub const SHAPES: &[(&str, &str, PhiQuiMeshType)] = &[
    ("plane", "Plane", PhiQuiMeshType::Plane),
    ("cube", "Cube", PhiQuiMeshType::Cube),
    ("circle", "Circle", PhiQuiMeshType::Circle),
    ("uv_sphere", "Sphere", PhiQuiMeshType::Sphere),
    ("ico_sphere", "Icosphere", PhiQuiMeshType::Icosahedron),
    ("cylinder", "Cylinder", PhiQuiMeshType::Cylinder),
    ("cone", "Cone", PhiQuiMeshType::Cone),
    ("torus", "Torus", PhiQuiMeshType::Torus),
    ("monkey", "Suzanne", PhiQuiMeshType::Suzanne),
];

thread_local! {
    /// One mesh per primitive, built once: the object list is memoised on
    /// mesh identity, so a shape must hand out the same `Arc` every time.
    static PRIMITIVES: std::cell::RefCell<HashMap<&'static str, Arc<SceneMesh>>> = Default::default();
}

/// The mesh of a primitive by slug, in Blender's Z-up object space.
pub fn primitive_mesh(shape: &str) -> Option<Arc<SceneMesh>> {
    let &(slug, _, kind) = SHAPES.iter().find(|(slug, ..)| *slug == shape)?;
    Some(PRIMITIVES.with(|cell| {
        Arc::clone(cell.borrow_mut().entry(slug).or_insert_with(|| {
            // The spatial buffers are (x, up, z); Blender is (x, y, z up).
            let (vertices, indices, normals) = SpatialObject3D::buffers(kind);
            Arc::new(SceneMesh {
                vertices: vertices.iter().map(|v| [v.0, v.2, v.1]).collect(),
                indices,
                normals: SceneNormals::PerVertex(normals.iter().map(|n| [n.0, n.2, n.1]).collect()),
                corners: Default::default(),
            })
        }))
    }))
}

/// `Cube`, then `Cube.001`, `Cube.002`, … — Blender's `BLI_uniquename`.
pub fn unique_name(base: &str, taken: &[String]) -> String {
    let stem = base.rsplit_once('.').filter(|(_, n)| n.len() == 3 && n.chars().all(|c| c.is_ascii_digit())).map_or(base, |(s, _)| s);
    if !taken.iter().any(|t| t == stem) {
        return stem.to_string();
    }
    (1..1000).map(|n| format!("{stem}.{n:03}")).find(|candidate| !taken.contains(candidate)).unwrap_or_else(|| stem.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_the_way_blender_makes_them() {
        assert_eq!(unique_name("Cube", &[]), "Cube");
        assert_eq!(unique_name("Cube", &["Cube".into()]), "Cube.001");
        assert_eq!(unique_name("Cube.001", &["Cube".into(), "Cube.001".into()]), "Cube.002");
    }

    #[test]
    fn a_primitive_mesh_is_built_once_and_stands_in_blender_space() {
        let a = primitive_mesh("cube").unwrap();
        let b = primitive_mesh("cube").unwrap();
        assert!(Arc::ptr_eq(&a, &b), "one Arc per shape, so the memo holds");
        assert!(a.vertices.iter().all(|v| v.iter().all(|c| c.abs() <= 1.0 + 1e-6)), "Blender's 2 m cube");
        assert_eq!(primitive_mesh("teapot"), None);
    }
}
