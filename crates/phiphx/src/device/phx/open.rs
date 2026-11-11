//! A PHX file as a `SceneModel`, and back. A PHX scene is JSON plus a magic;
//! a PHX point cloud (`PHX\0` chunks) is not a scene and is refused.
use crate::device::phx::cloud::chunk::PHX_MAGIC;
use crate::device::phx::scene::PhxScene;
use crate::device::Source;
use crate::device::{Peek, PeekScene};
use crate::scene::{SceneLoadError, SceneMesh, SceneModel, SceneNormalList, SceneObject};
use crate::{PointCloudError, Result};
use crate::device::phx::scene::PhxMesh;
/// Version, name and frame of a PHX scene, without building a model.
pub fn peek(bytes: &[u8]) -> Result<Peek> {
    if bytes.starts_with(PHX_MAGIC) {
        return Ok(Peek { version: "1".into(), ..Default::default() });
    }
    let scene = PhxScene::from_bytes(bytes)?;
    let frame = scene.frame;
    Ok(Peek {
        version: scene.format,
        scenes: vec![PeekScene { scene: scene.name, frame_start: frame, frame_end: frame }],
        thumbnail: None,
    })
}
/// The scene a PHX file holds: one object per mesh.
pub fn scene(source: &Source) -> std::result::Result<SceneModel, SceneLoadError> {
    if source.bytes.starts_with(PHX_MAGIC) {
        return Err(SceneLoadError::Unsupported("a PHX point cloud holds no scene".into()));
    }
    let phx = PhxScene::from_bytes(source.bytes).map_err(malformed)?;
    Ok(model(&phx))
}
/// A scene as PHX scene bytes.
pub fn write(scene: &SceneModel) -> std::result::Result<Vec<u8>, SceneLoadError> {
    to_phx(scene).to_bytes().map_err(malformed)
}
fn malformed(e: PointCloudError) -> SceneLoadError {
    SceneLoadError::Malformed(e.to_string())
}
/// Every mesh of the PHX scene as an object, placed by its world matrix.
pub fn model(phx: &PhxScene) -> SceneModel {
    let mut model = SceneModel::default();
    model.report.scene = phx.name.clone();
    for m in &phx.meshes {
        let normals = if m.normals.len() == m.vertices.len() {
            SceneNormalList::PerVertex(m.normals.clone())
        } else {
            SceneNormalList::PerFace(Vec::new())
        };
        let flat = m.indices.iter().flatten().copied().collect();
        let shape = SceneMesh { vertices: m.vertices.clone(), indices: flat, normals, corners: Default::default() };
        let mut object = SceneObject::identity(m.name.clone(), shape);
        object.matrix_world = m.matrix_world;
        model.locals.push(object.matrix_world);
        model.object_types.push("MESH".into());
        model.report.verts += object.mesh.vertices.len();
        model.report.tris += object.mesh.indices.len() / 3;
        model.objects.push(object);
    }
    model.report.objects = model.objects.len();
    model
}
/// A scene as a PHX scene: its objects as meshes (geometry and placement).
pub fn to_phx(scene: &SceneModel) -> PhxScene {
    let name = if scene.report.scene.is_empty() { "Scene" } else { &scene.report.scene };
    let mut phx = PhxScene::new(name);
    for o in &scene.objects {
        let normals = match &o.mesh.normals {
            SceneNormalList::PerVertex(n) => n.clone(),
            SceneNormalList::PerFace(_) => Vec::new(),
        };
        phx.add_mesh(PhxMesh {
            name: o.name.clone(),
            vertices: o.mesh.vertices.clone(),
            normals,
            uvs: o.uvs.as_ref().clone(),
            colors: Vec::new(),
            indices: o.mesh.indices.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect(),
            materials: Vec::new(),
            matrix_world: o.matrix_world,
            visible: true,
        });
    }
    phx
}