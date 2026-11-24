//! A whole [`SceneModel`] as USDA, whatever it was read from: every
//! visible object with faces an Xform at its world transform holding its
//! mesh, every material a `UsdPreviewSurface`, per-face material slots as
//! `GeomSubset`s. The scene model is Blender's space (Z up); the layer is
//! Y up, so each transform is turned once on the way out.

use super::UsdaWriter;
use super::name::{UsdNameSet, usd_word};
use super::stage::UsdMesh;
use crate::format::PreviewSurfaceMaterial;
use crate::scene::{SceneMaterial, SceneModel, SceneNormals, SceneObject};
use std::collections::HashMap;

/// A scene written as USDA, and what went into it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UsdaScene {
    pub text: String,
    /// Mesh prims written (one per visible object with faces).
    pub mesh_count: usize,
    pub triangle_count: usize,
    /// The prim name each written object got, in scene order.
    pub prim: Vec<String>,
}

/// Z up (Blender) to Y up (USD): x stays, z becomes y, y becomes -z.
pub const Z_TO_Y: glam::Mat4 = glam::Mat4::from_cols(glam::Vec4::X, glam::Vec4::NEG_Z, glam::Vec4::Y, glam::Vec4::W);

impl SceneModel {
    /// Whether object `i` is one a viewer would see (not hidden, not
    /// disabled in viewports) and has faces to draw.
    pub fn shows(&self, i: usize) -> bool {
        let Some(o) = self.objects.get(i) else { return false };
        let s = &self.selection;
        !o.mesh.indices.is_empty() && !s.hidden.contains(&o.name) && !s.viewport_disabled.contains(&o.name)
    }

    /// The scene as a USDA layer, Y up, in metres.
    pub fn to_usda(&self) -> UsdaScene {
        let stem = self.source_path.as_deref().and_then(|p| std::path::Path::new(p).file_stem()).and_then(|s| s.to_str());
        let named = Some(self.report.scene.as_str()).filter(|s| !s.is_empty());
        let root = UsdNameSet::clean(stem.or(named).unwrap_or(usd_word("root")));
        let look_scope = usd_word("look");
        let scope = format!("/{root}/{look_scope}");
        let mut w = UsdaWriter::stage(&root, usd_word("up"));
        w.push_scope(&root, "Xform");
        let mut sibling = UsdNameSet::default();
        sibling.take(look_scope);
        let (look_name, look) = self.look_list();
        let mut out = UsdaScene::default();
        for (_, o) in self.objects.iter().enumerate().filter(|(i, _)| self.shows(*i)) {
            let prim = sibling.take(&o.name);
            w.push_scope(&prim, "Xform");
            let world = Z_TO_Y * glam::Mat4::from_cols_array_2d(&o.matrix_world);
            w.transform(&world.to_cols_array_2d());
            let slot: Vec<String> = o.materials.iter().map(|m| format!("{scope}/{}", look_name[&m.name])).collect();
            object_mesh(&mut w, o, &prim, &slot);
            w.pop_scope();
            out.mesh_count += 1;
            out.triangle_count += o.mesh.indices.len() / 3;
            out.prim.push(prim);
        }
        if !look.is_empty() {
            w.push_scope(look_scope, "Scope");
            for m in &look {
                w.look(&scope, &look_name[&m.name], &preview(m));
            }
            w.pop_scope();
        }
        w.pop_scope();
        out.text = w.get_text();
        out
    }

    /// Every material the shown objects use, once each, with its prim name.
    fn look_list(&self) -> (HashMap<String, String>, Vec<&SceneMaterial>) {
        let mut name = UsdNameSet::default();
        let mut by_name = HashMap::new();
        let mut look = Vec::new();
        for (_, o) in self.objects.iter().enumerate().filter(|(i, _)| self.shows(*i)) {
            for m in &o.materials {
                if !by_name.contains_key(&m.name) {
                    by_name.insert(m.name.clone(), name.take(&m.name));
                    look.push(m);
                }
            }
        }
        (by_name, look)
    }
}

/// One object's mesh prim under its Xform: its triangles, a normal per
/// corner, its UVs, bound to its first slot and, when its faces use more
/// than one slot, a subset per slot.
fn object_mesh(w: &mut UsdaWriter, o: &SceneObject, prim: &str, slot: &[String]) {
    let mesh = &o.mesh;
    let triangle = mesh.indices.len() / 3;
    let normal: Vec<[f32; 3]> = (0..mesh.indices.len()).map(|c| corner_normal(o, c)).collect();
    let count = vec![3u32; triangle];
    let per_face = o.material_indices.len() == triangle && o.material_indices.iter().any(|&s| s != o.material_indices[0]);
    let first = o.material_indices.first().map(|&s| s as usize).unwrap_or(0);
    let mut subset = Vec::new();
    if per_face {
        let mut sibling = UsdNameSet::default();
        for (s, look) in slot.iter().enumerate() {
            let face: Vec<u32> = (0..triangle as u32).filter(|&t| o.material_indices[t as usize] as usize == s).collect();
            if !face.is_empty() {
                let name = sibling.take(look.rsplit('/').next().unwrap_or(look));
                subset.push((name, look.clone(), face));
            }
        }
    }
    let colour = o.materials.get(first).map(|m| m.base_color).unwrap_or(o.color);
    w.mesh(&UsdMesh {
        name: prim,
        point: &mesh.vertices,
        count: &count,
        index: &mesh.indices,
        normal: &normal,
        uv: &o.uvs,
        colour: [colour[0], colour[1], colour[2]],
        look: slot.get(first).or(slot.first()).cloned(),
        subset,
    });
}

/// The shading normal at triangle corner `c`: the split / smooth normal
/// when the mesh has one there, else its vertex or face normal, else the
/// triangle's own.
fn corner_normal(o: &SceneObject, c: usize) -> [f32; 3] {
    let mesh = &o.mesh;
    let corner = mesh.corners.normal(c);
    let given = match (&mesh.normals, corner != [0.0; 3]) {
        (_, true) => Some(corner),
        (SceneNormals::PerVertex(n), false) => n.get(mesh.indices[c] as usize).copied(),
        (SceneNormals::PerFace(n), false) => n.get(c / 3).copied(),
    };
    given.unwrap_or_else(|| {
        let t = c / 3 * 3;
        let p = |k: usize| glam::Vec3::from(mesh.vertices[mesh.indices[t + k] as usize]);
        (p(1) - p(0)).cross(p(2) - p(0)).normalize_or_zero().into()
    })
}

/// A scene material as the preview surface USD shows.
fn preview(m: &SceneMaterial) -> PreviewSurfaceMaterial {
    PreviewSurfaceMaterial {
        name: m.name.clone(),
        diffuse_color: [m.base_color[0], m.base_color[1], m.base_color[2]],
        roughness: m.roughness,
        metallic: m.metallic,
        opacity: m.base_color[3],
        ..Default::default()
    }
}
