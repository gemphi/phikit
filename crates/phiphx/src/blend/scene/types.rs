//! Semantic result types: `BlendScene`, `BlendCollection`,
//! `BlendSceneObject`, `BlendMaterial`.

/// Scene object extracted from .blend
#[derive(Debug, Clone)]
pub struct BlendSceneObject {
    pub name: String,
    pub object_type: String,
    pub location: [f32; 3],
    pub rotation: [f32; 3],   // Euler
    pub quaternion: [f32; 4], // Quaternion (w,x,y,z)
    pub scale: [f32; 3],
    /// World matrix, Blender `float[4][4]` layout (`m[col][row]`, translation
    /// in `m[3]`). Resolved from the parent chain by
    /// `BlendFile::resolve_world_matrices`; equals `matrix_local` for
    /// unparented objects.
    pub matrix_world: [[f32; 4]; 4],
    /// `BKE_object_to_mat4` result (loc/rot/scale + deltas), same layout.
    pub matrix_local: [[f32; 4]; 4],
    /// Saved address of this `OB` block (parent-chain resolution key).
    pub block_address: u64,
    /// `Object.parent` saved address, 0 when unparented.
    pub parent_address: u64,
    /// `Object.parentinv` (inverse of the parent's matrix at parenting time).
    pub parentinv: [[f32; 4]; 4],
    /// `Object.partype & PARTYPE` — 0 = PAROBJECT (only mode fully honoured).
    pub partype: u8,
    /// `Object.visibility_flag`: disabled in viewports / not selectable /
    /// disabled in renders.
    pub visibility: crate::blend::BlendVisibility,
    /// `Object.col`: the object's own colour (RGBA, scene-linear), Solid's
    /// Color ▸ Object. White when the file has none.
    pub color: [f32; 4],
    /// Whether it casts a shadow, in the viewport and to rays.
    pub shadow: crate::blend::BlendShadowCast,
    pub vertices: Vec<[f32; 3]>,
    pub faces: crate::blend::BFaces,
    /// Per-loop normals (one per loop/corner, indexed same as face loops).
    /// Empty when the file carries no custom or smooth normals.
    pub loop_normals: Vec<[f32; 3]>,
    /// Per-loop UV coordinates (one per loop/corner). Empty when no UV layer
    /// is present in the mesh.
    pub loop_uvs: Vec<[f32; 2]>,
    /// The active colour attribute, scene-linear, one per loop (corner);
    /// empty when the mesh has none (`attrs.rs`, `attr_corner_colors`).
    pub loop_colors: Vec<[f32; 3]>,
    /// Camera / light / empty display data (overlay engine inputs).
    pub data: crate::blend::object::data::BlendObjectData,
    /// Per-face smooth flag (`!sharp_face`). Empty = unknown (draw flat).
    pub face_smooth: Vec<bool>,
    /// Per-face material slot (`material_index` attribute). Empty when the
    /// mesh has one slot / no attribute.
    pub face_materials: Vec<u16>,
    pub modifiers: Vec<String>,
    pub instance_targets: Vec<String>,
    pub is_instance: bool,
    /// Base/diffuse colour (RGBA) from the object's first material.
    pub base_color: [f32; 4],
    /// PBR roughness from the object's first material.
    pub roughness: f32,
    /// PBR metallic from the object's first material.
    pub metallic: f32,
    /// All material slots on this object (colours + PBR scalars). Index 0 is
    /// the same as `base_color`/`roughness`/`metallic`; additional slots
    /// represent multi-material assignments. Empty only when no material is
    /// linked.
    pub materials: Vec<BlendMaterial>,
    /// Texture image file paths referenced by the object's shader node trees
    /// (Image Texture → filepath). Relative to the .blend file or absolute,
    /// as stored by Blender. May include `//` prefix (Blender-relative).
    pub texture_paths: Vec<String>,
}

/// A single PBR material slot extracted from a Blender object.
#[derive(Debug, Clone)]
pub struct BlendMaterial {
    pub name: String,
    /// Principled BSDF "Base Color" (Material Preview / Rendered).
    pub base_color: [f32; 4],
    /// `Material.r/g/b/a` — the viewport display colour Blender's Solid mode
    /// draws with `color_type = MATERIAL` (the default).
    pub viewport_color: [f32; 4],
    /// `Material.roughness` / `metallic` — the viewport display values
    /// Solid mode shades with (0.4 and 0 by default), not the Principled
    /// BSDF's.
    pub viewport_pbr: [f32; 2],
    pub roughness: f32,
    pub metallic: f32,
    /// Base Color texture path (Image Texture node), if any.
    pub base_color_texture: Option<String>,
    /// Principled BSDF inputs beyond colour/roughness/metallic.
    pub bsdf: BlendBsdf,
    /// The surface as a Mix Shader's two lobes, when it is one
    /// (`shade/surface.rs`); `bsdf` is then the two folded together.
    pub mix: Option<crate::blend::shade::surface::BlendMix>,
    /// The material's node tree as the Shader editor draws it — every node with
    /// its type, name, position and size, and the links between them
    /// (`shade/graph.rs`). Empty for a material with no tree.
    pub graph: crate::blend::shade::graph::BlendNodeGraph,
    /// Tangent-space normal map feeding the Principled `Normal` input.
    pub normal_map: Option<crate::blend::shade::links::BlendNormalMap>,
    /// Images linked to Roughness / Metallic / Emission Color.
    pub maps: crate::blend::shade::maps::BlendMaps,
}

pub use super::bsdf::BlendBsdf;

impl crate::MeshSource for BlendSceneObject {
    fn name(&self) -> &str {
        &self.name
    }
    fn vertices(&self) -> &[[f32; 3]] {
        &self.vertices
    }
    fn indices(&self) -> &[[u32; 3]] {
        // This is inefficient but satisfies the trait for now.
        // In a real implementation, faces would be converted to indices once.
        &[] // Pending proper index conversion
    }
    fn matrix_world(&self) -> [[f32; 4]; 4] {
        self.matrix_world
    }
}

/// Collection info from .blend
#[derive(Debug, Clone)]
pub struct BlendCollection {
    pub name: String,
    pub object_names: Vec<String>,
    /// Indices into [`BlendScene::objects`] for the objects directly held by
    /// this collection (not its children). Lets a consumer render a precise
    /// subset without re-matching by name (which collides across duplicate
    /// object names in different collections).
    pub object_indices: Vec<usize>,
    pub children: Vec<BlendCollection>,
    pub is_instance: bool,
    pub instance_targets: Vec<String>,
    /// Saved address of the `GR` block (view-layer flag lookup key).
    pub block_address: u64,
    /// `LayerCollection.flag & LAYER_COLLECTION_EXCLUDE` — unchecked in the outliner.
    pub exclude: bool,
    /// `LayerCollection.flag & LAYER_COLLECTION_HIDE` — eye closed in the outliner.
    pub hide_viewport: bool,
    /// `Collection.flag`: disabled in viewports / not selectable / disabled
    /// in renders — the collection's own toggles, every layer alike.
    pub visibility: crate::blend::BlendVisibility,
}

impl BlendCollection {
    /// Collect this collection's object indices plus every descendant's,
    /// flattened. Order follows a depth-first walk; duplicates are possible
    /// only if the source file links one object into multiple collections.
    pub fn all_object_indices(&self) -> Vec<usize> {
        let mut out = self.object_indices.clone();
        for child in &self.children {
            out.extend(child.all_object_indices());
        }
        out
    }

    /// Depth-first search for the first descendant collection (including
    /// `self`) whose name equals `name`.
    pub fn find(&self, name: &str) -> Option<&BlendCollection> {
        if self.name == name {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(name))
    }
}

/// Complete scene from .blend file
#[derive(Debug, Clone)]
pub struct BlendScene {
    pub name: String,
    pub collections: Vec<BlendCollection>,
    pub objects: Vec<BlendSceneObject>,
    /// Index into [`Self::objects`] of Blender's active object, if the file
    /// records one. Invariant: when `Some(i)`, `i < objects.len()`.
    pub active_object: Option<usize>,
    /// Indices into [`Self::objects`] of selected objects (Blender `SELECT`
    /// flag). Each entry is `< objects.len()`.
    pub selected: Vec<usize>,
    /// Indices of objects hidden in the view layer (`BASE_HIDDEN`).
    pub hidden: Vec<usize>,
    /// Frame range / fps / resolution / engine / world colour.
    pub props: crate::blend::scene::props::BlendSceneProps,
}

impl BlendScene {
    /// The object to frame/inspect by default.
    ///
    /// Resolution order:
    /// 1. Blender's active object, when recorded and in range (`i < len`).
    /// 2. Otherwise the largest mesh by vertex count, breaking ties
    ///    deterministically by the smaller object name (independent of
    ///    iteration order).
    /// 3. `None` only when the scene has no objects.
    pub fn primary_object(&self) -> Option<&BlendSceneObject> {
        // Delegate to the shared selection so blend/USD/glTF stay identical.
        crate::SubjectSelector::select_primary_subject(
            &self.objects,
            self.active_object,
            |o| o.vertices.len(),
            |o| o.name.as_str(),
        )
        .map(|i| &self.objects[i])
    }
}
