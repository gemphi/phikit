//! Bringing other formats into the scene — `File ▸ Import`. Blender's
//! importers add the file's objects to the scene collection, selected, the
//! last one active; the scene otherwise stays as it was. Here the meshes
//! phiphx reads (Wavefront OBJ, USD, and glTF when built with
//! `gltf-import`) become `SceneObject`s appended to the loaded model — or
//! to a model made for them when nothing is loaded — under the Scene
//! Collection, with a still motion each so playback leaves them be.

use crate::phiphx::blend::{BlendMotion, ObjectTransformRaw};
use crate::phiphx::scene::{SceneCollection, SceneMesh, SceneModel, SceneNormals, SceneObject};
use std::path::Path;

/// The extensions `read` takes.
pub fn readable(path: &str) -> bool {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
    matches!(ext.as_deref(), Some("obj" | "usd" | "usda" | "usdc") | Some("gltf" | "glb"))
}

/// The file's meshes as scene objects, in Blender's Z-up space.
pub fn read(path: &str) -> Result<Vec<SceneObject>, String> {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).unwrap_or_default();
    let (meshes, y_up) = match ext.as_str() {
        "obj" => (crate::phiphx::import::ObjImporter::import_file_as_mesh(path).map_err(|e| e.to_string())?, true),
        "usd" | "usda" | "usdc" => (crate::phiphx::import::usd::UsdImporter::import_file_as_mesh(path).map_err(|e| e.to_string())?, false),
        #[cfg(feature = "gltf-import")]
        "gltf" | "glb" => (crate::phiphx::import::gltf::GltfImporter::import_meshes(path).map_err(|e| e.to_string())?, true),
        other => return Err(format!(".{other}: not a format phiber reads yet")),
    };
    let stem = Path::new(path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Imported".into());
    Ok(meshes
        .into_iter()
        .enumerate()
        .filter(|(_, mesh)| !mesh.positions.is_empty())
        .map(|(i, mesh)| {
            // Y up (OBJ, glTF) → Blender's Z up: (x, y, z) → (x, −z, y).
            let up = |p: [f32; 3]| if y_up { [p[0], -p[2], p[1]] } else { p };
            let name = if mesh.name.is_empty() || mesh.name == "Default" { if i == 0 { stem.clone() } else { format!("{stem}.{i:03}") } } else { mesh.name.clone() };
            let normals = match &mesh.normals {
                Some(n) if n.len() == mesh.positions.len() => SceneNormals::PerVertex(n.iter().map(|n| up(*n)).collect()),
                _ => SceneNormals::PerFace(mesh.indices.iter().map(|_| [0.0, 0.0, 1.0]).collect()),
            };
            SceneObject::identity(
                name,
                SceneMesh {
                    vertices: mesh.positions.iter().map(|p| up(*p)).collect(),
                    indices: mesh.indices.iter().flat_map(|t| t.iter().copied()).collect(),
                    normals,
                    corners: Default::default(),
                },
            )
        })
        .collect())
}

/// `objects` appended to `model` under the Scene Collection, each with a
/// still motion, as Blender's importers add to the active collection.
pub fn append(model: &SceneModel, objects: Vec<SceneObject>) -> SceneModel {
    let mut next = model.clone();
    for object in objects {
        let index = next.objects.len();
        next.motion.motion.list.push(BlendMotion::still(object.name.clone(), ObjectTransformRaw::default()));
        next.motion.object.push(next.motion.motion.list.len() - 1);
        next.object_types.push("MESH".to_string());
        match next.collections.first_mut() {
            Some(root) => root.object_indices.push(index),
            None => next.collections.push(SceneCollection {
                name: "Scene Collection".into(),
                children: Vec::new(),
                object_indices: vec![index],
                extra_names: Vec::new(),
                exclude: false,
                hide_viewport: false,
                visibility: Default::default(),
            }),
        }
        next.objects.push(object);
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_obj_file_becomes_objects_in_blender_space_appended_to_the_scene() {
        let dir = std::env::temp_dir().join(format!("phiber_import_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("tri.obj");
        // A triangle standing up in Y-up OBJ space.
        std::fs::write(&file, "o Tri\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        let path = file.to_string_lossy().to_string();
        assert!(readable(&path));
        let objects = read(&path).unwrap();
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].name, "Tri");
        assert_eq!(objects[0].mesh.vertices[2], [0.0, 0.0, 1.0], "Y up became Z up");
        let model = append(&SceneModel::default(), objects);
        assert_eq!(model.objects.len(), 1);
        assert_eq!(model.motion.object, [0]);
        assert_eq!(model.collections[0].object_indices, [0]);
        assert!(!readable("thing.fbx"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
