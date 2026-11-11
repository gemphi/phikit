//! A USD file as a `SceneModel`.
//!
//! The usdkit parser reads the layer into a [`UsdStage`]; its mesh prims,
//! each placed by the composed `xformOp`s of its parents, become the scene's
//! objects. What the parser does not read (time samples, value types it has
//! no variant for) and the binary crate format go to the heuristic readers
//! in [`super::mesh`], which are named as fallbacks and kept for those files.
use super::UsdaParser;
use crate::device::Source;
use crate::device::usd::prim::UsdPrim;
use crate::device::usd::stage::UsdStage;
use crate::device::usd::xform::XformResolver;
use crate::device::usd::attribute::UsdValue;
use super::mesh::UsdImporter;
use crate::scene::{SceneLoadError, SceneMesh, SceneModel, SceneNormals, SceneObject};
use crate::Mesh;
use glam::Mat4;

/// The scene a USD file holds.
pub fn open(source: &Source) -> Result<SceneModel, SceneLoadError> {
    let bytes = source.bytes;
    let mesh = if bytes.starts_with(b"#usda") {
        let text = std::str::from_utf8(bytes).map_err(|e| SceneLoadError::Malformed(format!("Invalid UTF-8: {e}")))?;
        match UsdaParser::parse(text).map(|s| mesh_list(&s)) {
            Ok(list) if !list.is_empty() => list,
            _ => UsdImporter::import_usda_as_mesh(text).map_err(malformed)?,
        }
    } else if bytes.starts_with(b"PXR-USDC") {
        UsdImporter::import_usdc_as_mesh(bytes).map_err(malformed)?
    } else if bytes.starts_with(b"PK") {
        package(bytes)?
    } else {
        return Err(SceneLoadError::Unsupported("not a USD layer".into()));
    };
    Ok(model(source.name, mesh))
}

#[cfg(feature = "usdz-import")]
fn package(bytes: &[u8]) -> Result<Vec<Mesh>, SceneLoadError> {
    match crate::device::usd::package::UsdzReader::read(bytes).map(|s| mesh_list(&s)) {
        Ok(list) if !list.is_empty() => Ok(list),
        _ => UsdImporter::import_usdz_as_mesh(bytes).map_err(malformed),
    }
}

#[cfg(not(feature = "usdz-import"))]
fn package(_bytes: &[u8]) -> Result<Vec<Mesh>, SceneLoadError> {
    Err(SceneLoadError::Unsupported("USDZ needs the `usdz-import` feature".into()))
}

fn malformed(e: crate::PointCloudError) -> SceneLoadError {
    SceneLoadError::Malformed(e.to_string())
}

/// Every mesh prim of the stage as a `Mesh` (triangles, local points), its
/// `transform` the composed matrix of the prim and its parents.
pub fn mesh_list(stage: &UsdStage) -> Vec<Mesh> {
    let mut out = Vec::new();
    for top in &stage.root.children {
        walk(top, Mat4::IDENTITY, &mut out);
    }
    out
}

fn walk(prim: &UsdPrim, parent: Mat4, out: &mut Vec<Mesh>) {
    let world = parent * XformResolver::resolve(prim);
    if prim.type_name == "Mesh" {
        if let Some(mesh) = mesh_of(prim, world) {
            out.push(mesh);
        }
    }
    for child in &prim.children {
        walk(child, world, out);
    }
}

fn mesh_of(prim: &UsdPrim, world: Mat4) -> Option<Mesh> {
    let Some(UsdValue::Float3Array(point)) = prim.attribute("points") else { return None };
    let count = int_list(prim, "faceVertexCounts");
    let index = int_list(prim, "faceVertexIndices");
    let mut triangle = Vec::new();
    let mut at = 0usize;
    for n in count {
        let n = n.max(0) as usize;
        let face = index.get(at..at + n)?;
        for k in 1..n.saturating_sub(1) {
            triangle.push([face[0] as u32, face[k] as u32, face[k + 1] as u32]);
        }
        at += n;
    }
    let per_vertex = |name: &str| match prim.attribute(name) {
        Some(UsdValue::Float3Array(v)) if v.len() == point.len() => Some(v.clone()),
        _ => None,
    };
    let uv = match prim.attribute("primvars:st") {
        Some(UsdValue::Float2Array(v)) if v.len() == point.len() => Some(v.clone()),
        _ => None,
    };
    Some(Mesh {
        name: prim.name().to_string(),
        positions: point.clone(),
        normals: per_vertex("normals").or_else(|| per_vertex("primvars:normals")),
        uvs: uv,
        indices: triangle,
        transform: Some(world),
        ..Default::default()
    })
}

fn int_list(prim: &UsdPrim, name: &str) -> Vec<i32> {
    match prim.attribute(name) {
        Some(UsdValue::IntArray(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Meshes as a scene: one object each, placed by its `transform`.
pub fn model(name: &str, mesh: Vec<Mesh>) -> SceneModel {
    let mut model = SceneModel::default();
    model.report.scene = name.to_string();
    for m in mesh {
        let normals = match m.normals {
            Some(n) => SceneNormals::PerVertex(n),
            None => SceneNormals::PerFace(Vec::new()),
        };
        let flat = m.indices.iter().flatten().copied().collect();
        let shape = SceneMesh { vertices: m.positions, indices: flat, normals, corners: Default::default() };
        let mut object = SceneObject::identity(m.name, shape);
        object.matrix_world = m.transform.unwrap_or(Mat4::IDENTITY).to_cols_array_2d();
        model.locals.push(object.matrix_world);
        model.object_types.push("MESH".into());
        model.report.verts += object.mesh.vertices.len();
        model.report.tris += object.mesh.indices.len() / 3;
        model.objects.push(object);
    }
    model.report.objects = model.objects.len();
    model
}
