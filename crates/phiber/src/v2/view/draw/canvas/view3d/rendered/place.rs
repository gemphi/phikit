//! How a scene of the shape the tracer holds goes up (`worker.rs`): objects
//! moved whole — the same meshes under other matrices, a grab, a turn, a
//! scale — are placed (`Tracer::reinstance`: the instance table and the
//! small top tree, nothing of the meshes); vertices moved (a frame of an
//! animation, an edit) are posed (`Tracer::repose`: the vertices in place,
//! each tree refitted); anything else builds the tracer again.

use super::super::mesh::BMeshPalette;
use super::scene;
use crate::v2::app::journal::BJournal;
use crate::v2::scene::BObject3D;
use phiray::{Gpu, Tracer};
use std::time::Instant;

/// What the tracer kept of itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BKept {
    /// Only matrices changed: the meshes, their trees and the atlas stay.
    Placed,
    /// Vertices changed: the trees are refitted, to be built fresh once still.
    Posed,
}

/// Whether `new` is `old` with its objects placed elsewhere and nothing
/// else: the same meshes (the very same, not copies), materials and
/// textures, so the scene the tracer holds is the scene.
pub(super) fn only_placed(old: &[BObject3D], new: &[BObject3D]) -> bool {
    use std::sync::Arc;
    old.len() == new.len()
        && old.iter().zip(new).all(|(a, b)| match (a, b) {
            (BObject3D::Primitive { shape: s, .. }, BObject3D::Primitive { shape: t, .. }) => s == t,
            (
                BObject3D::Loaded { mesh: m, materials: a, material_indices: i, uvs: u, .. },
                BObject3D::Loaded { mesh: n, materials: b, material_indices: j, uvs: v, .. },
            ) => Arc::ptr_eq(m, n) && Arc::ptr_eq(i, j) && Arc::ptr_eq(u, v) && a == b,
            _ => false,
        })
}

/// Take `objects` into `tracer`, which holds a scene of the same shape:
/// placed when only matrices changed from `held`, posed otherwise; `None`
/// when the tracer could not (build it again). Each way notes its time in
/// the session journal.
pub(super) fn update(gpu: &Gpu, tracer: &mut Tracer, held: Option<&[BObject3D]>, objects: &[BObject3D], palette: &BMeshPalette) -> Option<BKept> {
    let at = Instant::now();
    let ms = || at.elapsed().as_secs_f64() * 1000.0;
    if held.is_some_and(|h| only_placed(h, objects)) && tracer.reinstance(gpu, &scene::worlds(objects)) {
        BJournal::note(format!("Tracer: {} objects placed in {:.1}ms", objects.len(), ms()));
        return Some(BKept::Placed);
    }
    if tracer.repose(gpu, &scene::build_pose(objects, palette)) {
        BJournal::note(format!("Tracer: scene posed in {:.0}ms", ms()));
        return Some(BKept::Posed);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::BShape;

    fn cube(x: f32) -> BObject3D {
        BObject3D::Primitive { name: "Cube".into(), shape: BShape::Cube, x, y: 0.0 }
    }

    #[test]
    fn a_moved_primitive_is_placed_and_another_shape_is_not() {
        assert!(only_placed(&[cube(0.0)], &[cube(5.0)]));
        let sphere = BObject3D::Primitive { name: "Cube".into(), shape: BShape::Sphere, x: 0.0, y: 0.0 };
        assert!(!only_placed(&[cube(0.0)], &[sphere]));
        assert!(!only_placed(&[cube(0.0)], &[cube(0.0), cube(1.0)]));
    }

    fn loaded() -> BObject3D {
        use crate::phiphx::scene::{SceneMesh, SceneNormals};
        use std::sync::Arc;
        let mesh = Arc::new(SceneMesh { vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], indices: vec![0, 1, 2], normals: SceneNormals::PerFace(Vec::new()), corners: Default::default() });
        BObject3D::Loaded {
            name: "Mesh".into(),
            mesh,
            location: [0.0; 3],
            scale: [1.0; 3],
            matrix_world: phiray::Instance::translation([0.0; 3]),
            materials: Vec::new(),
            material_indices: Arc::default(),
            uvs: Arc::default(),
            color: [1.0; 4],
        }
    }

    /// The object with its mesh built again (a rig's frame): the same
    /// triangles, another `Arc`.
    fn rebuilt(o: &BObject3D) -> BObject3D {
        let mut copied = o.clone();
        if let BObject3D::Loaded { mesh, .. } = &mut copied {
            *mesh = std::sync::Arc::new((**mesh).clone());
        }
        copied
    }

    #[test]
    fn a_loaded_object_is_placed_only_under_the_very_same_mesh() {
        let scene = loaded();
        let mut moved = scene.clone();
        if let BObject3D::Loaded { matrix_world, .. } = &mut moved {
            matrix_world[3][0] = 4.0;
        }
        assert!(only_placed(&[scene.clone()], &[moved]));
        assert!(!only_placed(&[scene.clone()], &[rebuilt(&scene)]), "a mesh built again is a pose, not a place");
    }

    #[test]
    fn moved_objects_are_placed_and_a_rebuilt_mesh_is_posed() {
        let Some(gpu) = phiray::Gpu::new() else { return };
        let grey = gpui::white();
        let palette = BMeshPalette { shading: crate::v2::scene::BShading::Solid, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey };
        let held = vec![cube(0.0), loaded()];
        let mut tracer = Tracer::from_scene(&gpu, scene::build(&held, &palette)).expect("a scene");
        let mut moved = held.clone();
        moved[0] = cube(3.0);
        if let BObject3D::Loaded { matrix_world, .. } = &mut moved[1] {
            matrix_world[3][2] = 2.0;
        }
        assert_eq!(update(&gpu, &mut tracer, Some(&held), &moved, &palette), Some(BKept::Placed));
        let posed = vec![cube(3.0), rebuilt(&moved[1])];
        assert_eq!(update(&gpu, &mut tracer, Some(&moved), &posed, &palette), Some(BKept::Posed));
        assert_eq!(update(&gpu, &mut tracer, None, &posed, &palette), Some(BKept::Posed), "nothing held to compare: posed");
        assert_eq!(update(&gpu, &mut tracer, Some(&posed), &posed[..1], &palette), None, "an object fewer: built again");
    }
}
