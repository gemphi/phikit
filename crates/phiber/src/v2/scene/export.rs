//! Writing the scene out — `File ▸ Export`. Every visible object's
//! triangles in world space (Blender's Z up, metres), one object per
//! named block where the format has one: Wavefront OBJ (`o`), Stanford PLY
//! (one element list, comments naming the objects), STL (one solid per
//! object), USD ASCII (an `Xform` per object with its `Mesh`), and glTF 2.0
//! as a single `.gltf` with its buffer embedded (one node and mesh per
//! object). What phiber exports is what it draws: no materials yet, no
//! animation — a still frame of geometry.

use crate::phiphx::spatial::SpatialObject3D;
use crate::v2::scene::{BObject3D, MESH_SCALE};
use glam::{Mat4, Vec3};
use std::path::Path;

/// The formats `scene/export` writes, by the slug the menu names.
pub const FORMATS: &[(&str, &str)] = &[("obj", "obj"), ("ply", "ply"), ("stl", "stl"), ("usda", "usda"), ("gltf", "gltf")];

/// One object's geometry, in world space.
pub struct BExportMesh {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

impl BExportMesh {
    pub fn of(object: &BObject3D) -> Self {
        let (vertices, triangles): (Vec<[f32; 3]>, Vec<u32>) = match object {
            BObject3D::Primitive { shape, x, y, .. } => {
                let (v, idx, _) = SpatialObject3D::buffers(shape.mesh_type());
                let (ox, oy) = (x / MESH_SCALE, y / MESH_SCALE);
                (v.iter().map(|v| [ox + v.0, oy + v.2, v.1]).collect(), idx)
            }
            BObject3D::Loaded { mesh, matrix_world, .. } => {
                let m = Mat4::from_cols_array_2d(matrix_world);
                (mesh.vertices.iter().map(|v| m.transform_point3(Vec3::from(*v)).to_array()).collect(), mesh.indices.clone())
            }
        };
        let count = vertices.len() as u32;
        let triangles = triangles
            .chunks_exact(3)
            .filter(|t| t.iter().all(|&i| i < count))
            .map(|t| [t[0], t[1], t[2]])
            .collect();
        Self { name: object.name().to_string(), vertices, triangles }
    }
}

/// The extension a format's file takes.
pub fn extension(format: &str) -> Option<&'static str> {
    FORMATS.iter().find(|(slug, _)| *slug == format).map(|(_, ext)| *ext)
}

/// Write `objects` to `path` in `format`. The file takes the format's
/// extension when it has none.
pub fn write(path: &Path, format: &str, objects: &[BObject3D]) -> Result<std::path::PathBuf, String> {
    let ext = extension(format).ok_or_else(|| format!("{format}: not a format phiber writes"))?;
    let meshes: Vec<BExportMesh> = objects.iter().map(BExportMesh::of).collect();
    let file = if path.extension().is_some() { path.to_path_buf() } else { path.with_extension(ext) };
    let bytes = match format {
        "obj" => obj(&meshes).into_bytes(),
        "ply" => ply(&meshes).into_bytes(),
        "stl" => stl(&meshes).into_bytes(),
        "usda" => usda(&meshes).into_bytes(),
        "gltf" => gltf(&meshes).into_bytes(),
        _ => unreachable!("extension() vetted the format"),
    };
    if let Some(folder) = file.parent().filter(|f| !f.as_os_str().is_empty()) {
        std::fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    }
    std::fs::write(&file, bytes).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(file)
}

fn safe(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' }).collect()
}

/// Wavefront OBJ: Blender writes Y up, so `(x, y, z)` → `(x, z, -y)`.
pub fn obj(meshes: &[BExportMesh]) -> String {
    let mut out = String::from("# phiber\n");
    let mut base = 1u32;
    for mesh in meshes {
        out.push_str(&format!("o {}\n", safe(&mesh.name)));
        for v in &mesh.vertices {
            out.push_str(&format!("v {} {} {}\n", v[0], v[2], -v[1]));
        }
        for t in &mesh.triangles {
            out.push_str(&format!("f {} {} {}\n", base + t[0], base + t[1], base + t[2]));
        }
        base += mesh.vertices.len() as u32;
    }
    out
}

/// Stanford PLY, ASCII, Blender's axes as they are.
pub fn ply(meshes: &[BExportMesh]) -> String {
    let (verts, faces): (usize, usize) = meshes.iter().fold((0, 0), |(v, f), m| (v + m.vertices.len(), f + m.triangles.len()));
    let mut out = String::from("ply\nformat ascii 1.0\ncomment phiber\n");
    for mesh in meshes {
        out.push_str(&format!("comment object {} {} vertices\n", safe(&mesh.name), mesh.vertices.len()));
    }
    out.push_str(&format!("element vertex {verts}\nproperty float x\nproperty float y\nproperty float z\nelement face {faces}\nproperty list uchar uint vertex_indices\nend_header\n"));
    for mesh in meshes {
        for v in &mesh.vertices {
            out.push_str(&format!("{} {} {}\n", v[0], v[1], v[2]));
        }
    }
    let mut base = 0u32;
    for mesh in meshes {
        for t in &mesh.triangles {
            out.push_str(&format!("3 {} {} {}\n", base + t[0], base + t[1], base + t[2]));
        }
        base += mesh.vertices.len() as u32;
    }
    out
}

/// STL, ASCII, one solid per object.
pub fn stl(meshes: &[BExportMesh]) -> String {
    let mut out = String::new();
    for mesh in meshes {
        out.push_str(&format!("solid {}\n", safe(&mesh.name)));
        for t in &mesh.triangles {
            let [a, b, c] = t.map(|i| Vec3::from(mesh.vertices[i as usize]));
            let n = (b - a).cross(c - a).normalize_or_zero();
            out.push_str(&format!("  facet normal {} {} {}\n    outer loop\n", n.x, n.y, n.z));
            for p in [a, b, c] {
                out.push_str(&format!("      vertex {} {} {}\n", p.x, p.y, p.z));
            }
            out.push_str("    endloop\n  endfacet\n");
        }
        out.push_str(&format!("endsolid {}\n", safe(&mesh.name)));
    }
    out
}

/// USD ASCII: an `Xform` per object with its `Mesh`, Z up, metres.
pub fn usda(meshes: &[BExportMesh]) -> String {
    let mut out = String::from("#usda 1.0\n(\n    metersPerUnit = 1\n    upAxis = \"Z\"\n)\n\n");
    for mesh in meshes {
        let name = safe(&mesh.name);
        out.push_str(&format!("def Xform \"{name}\"\n{{\n    def Mesh \"{name}\"\n    {{\n"));
        out.push_str(&format!("        int[] faceVertexCounts = [{}]\n", vec!["3"; mesh.triangles.len()].join(", ")));
        let indices: Vec<String> = mesh.triangles.iter().flat_map(|t| t.iter().map(|i| i.to_string())).collect();
        out.push_str(&format!("        int[] faceVertexIndices = [{}]\n", indices.join(", ")));
        let points: Vec<String> = mesh.vertices.iter().map(|v| format!("({}, {}, {})", v[0], v[1], v[2])).collect();
        out.push_str(&format!("        point3f[] points = [{}]\n    }}\n}}\n\n", points.join(", ")));
    }
    out
}

/// glTF 2.0, one file: the buffer embedded as a data URI, a node and a
/// mesh per object. glTF is Y up, so `(x, y, z)` → `(x, z, -y)`.
pub fn gltf(meshes: &[BExportMesh]) -> String {
    use base64::Engine as _;
    let mut bin: Vec<u8> = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();
    let mut gltf_meshes = Vec::new();
    let mut nodes = Vec::new();
    for (i, mesh) in meshes.iter().enumerate() {
        let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        let start = bin.len();
        for v in &mesh.vertices {
            let p = [v[0], v[2], -v[1]];
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
                bin.extend_from_slice(&p[k].to_le_bytes());
            }
        }
        let pos_view = views.len();
        views.push(format!("{{\"buffer\":0,\"byteOffset\":{start},\"byteLength\":{}}}", bin.len() - start));
        let start = bin.len();
        for t in &mesh.triangles {
            for i in t {
                bin.extend_from_slice(&i.to_le_bytes());
            }
        }
        let idx_view = views.len();
        views.push(format!("{{\"buffer\":0,\"byteOffset\":{start},\"byteLength\":{}}}", bin.len() - start));
        let pos = accessors.len();
        accessors.push(format!(
            "{{\"bufferView\":{pos_view},\"componentType\":5126,\"count\":{},\"type\":\"VEC3\",\"min\":[{},{},{}],\"max\":[{},{},{}]}}",
            mesh.vertices.len(), lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]
        ));
        let idx = accessors.len();
        accessors.push(format!("{{\"bufferView\":{idx_view},\"componentType\":5125,\"count\":{},\"type\":\"SCALAR\"}}", mesh.triangles.len() * 3));
        gltf_meshes.push(format!(
            "{{\"name\":\"{}\",\"primitives\":[{{\"attributes\":{{\"POSITION\":{pos}}},\"indices\":{idx}}}]}}",
            safe(&mesh.name)
        ));
        nodes.push(format!("{{\"name\":\"{}\",\"mesh\":{i}}}", safe(&mesh.name)));
    }
    let scene_nodes: Vec<String> = (0..meshes.len()).map(|i| i.to_string()).collect();
    let uri = format!("data:application/octet-stream;base64,{}", base64::engine::general_purpose::STANDARD.encode(&bin));
    format!(
        "{{\"asset\":{{\"version\":\"2.0\",\"generator\":\"phiber\"}},\"scene\":0,\"scenes\":[{{\"nodes\":[{}]}}],\"nodes\":[{}],\"meshes\":[{}],\"accessors\":[{}],\"bufferViews\":[{}],\"buffers\":[{{\"byteLength\":{},\"uri\":\"{uri}\"}}]}}",
        scene_nodes.join(","), nodes.join(","), gltf_meshes.join(","), accessors.join(","), views.join(","), bin.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube() -> Vec<BObject3D> {
        vec![BObject3D::Primitive { name: "Cube".into(), shape: crate::v2::scene::BShape::Cube, x: 0.0, y: 0.0 }]
    }

    #[test]
    fn every_format_writes_the_cubes_geometry_and_names_it() {
        let dir = std::env::temp_dir().join(format!("phiber_export_{}", std::process::id()));
        for (format, ext) in FORMATS {
            let file = write(&dir.join("cube"), format, &cube()).unwrap();
            assert_eq!(file.extension().and_then(|e| e.to_str()), Some(*ext));
            let text = std::fs::read_to_string(&file).unwrap();
            assert!(text.contains("Cube"), "{format} names the object");
            assert!(text.len() > 200, "{format} carries the geometry");
        }
        let obj = std::fs::read_to_string(dir.join("cube.obj")).unwrap();
        assert_eq!(obj.lines().filter(|l| l.starts_with("f ")).count(), 12, "a cube is twelve triangles");
        let ply = std::fs::read_to_string(dir.join("cube.ply")).unwrap();
        assert!(ply.contains("element face 12"));
        let gltf = std::fs::read_to_string(dir.join("cube.gltf")).unwrap();
        assert!(gltf.contains("\"count\":36") && gltf.contains("data:application/octet-stream;base64,"));
        assert!(write(&dir.join("cube"), "fbx", &cube()).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
