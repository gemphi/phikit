//! One `SceneMesh` as a `UsdGeomMesh`.
//!
//! The scene's meshes are already evaluated (modifiers run, n-gons
//! fan-triangulated: `scene/import.rs`), so the layer carries triangles and
//! `subdivisionScheme = "none"`; letting USD subdivide again would apply the
//! modifier twice.
use crate::device::usd::write::scene::material::MaterialBook;
use crate::device::usd::write::scene::out::UsdOut;
use crate::device::usd::write::scene::text::UsdText;
use crate::scene::{SceneMesh, SceneNormalList, SceneObject};
use std::collections::BTreeSet;

/// Writes the `Mesh` prim of one object.
pub(super) struct UsdMesh;

impl UsdMesh {
    /// The geometry of `object`, bound to its material slots in `book`.
    pub fn write(out: &mut UsdOut, object: &SceneObject, book: &MaterialBook) {
        let mesh = &*object.mesh;
        let tri = mesh.indices.len() / 3;
        let bound = object.materials.iter().any(|m| book.path(&m.name).is_some());
        let meta = if bound { vec!["prepend apiSchemas = [\"MaterialBindingAPI\"]".to_string()] } else { Vec::new() };
        out.open("Mesh", "Mesh", &meta);
        // Blender draws both sides unless a material asks otherwise.
        out.line("uniform bool doubleSided = true");
        out.line(&format!("point3f[] points = {}", UsdText::vec3_list(&mesh.vertices)));
        if let Some((lo, hi)) = Self::extent(&mesh.vertices) {
            out.line(&format!("float3[] extent = [{}, {}]", UsdText::vec3(lo), UsdText::vec3(hi)));
        }
        out.line(&format!("int[] faceVertexCounts = {}", UsdText::int_list(std::iter::repeat_n(3, tri))));
        out.line(&format!("int[] faceVertexIndices = {}", UsdText::int_list(mesh.indices.iter().copied())));
        if let Some(n) = Self::corner_normal(mesh) {
            out.line(&format!("normal3f[] normals = {} (interpolation = \"faceVarying\")", UsdText::vec3_list(&n)));
        }
        // `uvs` is one per vertex: a seam's two sides are one vertex here,
        // the per-corner map (`scene/uv.rs`) is the UV editor's.
        if !object.uvs.is_empty() && object.uvs.len() == mesh.vertices.len() {
            out.line(&format!("texCoord2f[] primvars:st = {} (interpolation = \"vertex\")", UsdText::vec2_list(&object.uvs)));
        }
        if mesh.corners.colors.len() == mesh.indices.len() && !mesh.indices.is_empty() {
            out.line(&format!("color3f[] primvars:displayColor = {} (interpolation = \"faceVarying\")", UsdText::vec3_list(&mesh.corners.colors)));
        }
        out.line("uniform token subdivisionScheme = \"none\"");
        Self::bind(out, object, book, tri);
        out.close();
    }

    /// Min and max corner of the points, or `None` for no points.
    fn extent(points: &[[f32; 3]]) -> Option<([f32; 3], [f32; 3])> {
        let first = *points.first()?;
        Some(points.iter().fold((first, first), |(lo, hi), p| ([0, 1, 2].map(|k| lo[k].min(p[k])), [0, 1, 2].map(|k| hi[k].max(p[k])))))
    }

    /// One shading normal per triangle corner (`len == indices.len()`):
    /// the split / smooth normal of `corners` where it has one, else the
    /// face's own (a zero corner normal is Blender's flat marker).
    fn corner_normal(mesh: &SceneMesh) -> Option<Vec<[f32; 3]>> {
        let n = mesh.indices.len();
        if n == 0 {
            return None;
        }
        let face = |t: usize| Self::face_normal(mesh, t);
        let out: Vec<[f32; 3]> = if mesh.corners.len() == n {
            (0..n).map(|c| Some(mesh.corners.normal(c)).filter(|v| *v != [0.0; 3]).unwrap_or_else(|| face(c / 3))).collect()
        } else {
            match &mesh.normals {
                SceneNormalList::PerVertex(v) if v.len() == mesh.vertices.len() => mesh.indices.iter().map(|&i| v[i as usize]).collect(),
                SceneNormalList::PerFace(v) if v.len() == n / 3 => (0..n).map(|c| v[c / 3]).collect(),
                _ => (0..n).map(|c| face(c / 3)).collect(),
            }
        };
        Some(out)
    }

    /// The unit normal of triangle `t` from its winding; +Z when degenerate.
    fn face_normal(mesh: &SceneMesh, t: usize) -> [f32; 3] {
        let p = |k: usize| mesh.vertices[mesh.indices[3 * t + k] as usize];
        let (a, b, c) = (p(0), p(1), p(2));
        let (u, v) = ([0, 1, 2].map(|k| b[k] - a[k]), [0, 1, 2].map(|k| c[k] - a[k]));
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        let len = n.iter().map(|x| x * x).sum::<f32>().sqrt();
        if len > 0.0 { n.map(|x| x / len) } else { [0.0, 0.0, 1.0] }
    }

    /// Bind slot 0 on the mesh and, when the faces use several slots, one
    /// `GeomSubset` per slot used (a partition: every face in exactly one).
    fn bind(out: &mut UsdOut, object: &SceneObject, book: &MaterialBook, tri: usize) {
        let Some(first) = object.materials.first().and_then(|m| book.path(&m.name)) else { return };
        out.line(&format!("rel material:binding = <{first}>"));
        let slot = &object.material_indices;
        if object.materials.len() < 2 || slot.len() != tri {
            return;
        }
        let last = object.materials.len() - 1;
        let used: BTreeSet<usize> = slot.iter().map(|&s| (s as usize).min(last)).collect();
        out.line("uniform token subsetFamily:materialBind:familyType = \"partition\"");
        for s in used {
            let Some(path) = book.path(&object.materials[s].name) else { continue };
            let face = (0..tri).filter(|&t| (slot[t] as usize).min(last) == s).map(|t| t as u32);
            out.open("GeomSubset", &format!("Slot{s}"), &["prepend apiSchemas = [\"MaterialBindingAPI\"]".to_string()]);
            out.line("uniform token elementType = \"face\"");
            out.line("uniform token familyName = \"materialBind\"");
            out.line(&format!("int[] indices = {}", UsdText::int_list(face)));
            out.line(&format!("rel material:binding = <{path}>"));
            out.close();
        }
    }
}
