//! Shared types, constants and trait impls for the native .blend parser.

/// .blend file header magic
pub(crate) const BLEND_MAGIC: &[u8; 7] = b"BLENDER";

/// DNA block type identifier
pub(crate) const DNA1: &[u8; 4] = b"DNA1";

/// ENDB block type identifier  
pub(crate) const ENDB: &[u8; 4] = b"ENDB";

/// Blend file header information
#[derive(Debug, Clone)]
pub struct BlendHeader {
    pub version: String,
    pub pointer_size: u8, // 4 or 8
    pub little_endian: bool,
    pub version_number: u32,
}

/// A data block in the .blend file
#[derive(Debug, Clone)]
pub struct BlendBlock {
    pub code: [u8; 4],
    pub code_str: String,
    pub size: u32,
    pub old_address: u64, // Memory address when saved
    pub sdna_index: u32,
    pub count: u32,
    pub data: Vec<u8>,
}

/// DNA structure field definition
#[derive(Debug, Clone)]
pub struct DnaField {
    pub type_index: u16,
    pub name_index: u16,
}

/// DNA structure type definition
#[derive(Debug, Clone)]
pub struct DnaStruct {
    pub type_index: u16,
    pub fields: Vec<DnaField>,
}

/// DNA type information
#[derive(Debug, Clone)]
pub struct DnaType {
    pub names: Vec<String>,
    pub types: Vec<String>,
    pub type_sizes: Vec<u16>,
    pub structs: Vec<DnaStruct>,
}

/// Parsed .blend file
#[derive(Debug, Clone)]
pub struct BlendFile {
    pub header: BlendHeader,
    pub blocks: Vec<BlendBlock>,
    pub dna: Option<DnaType>,
}

/// Scene object extracted from .blend
#[derive(Debug, Clone)]
pub struct BlendSceneObject {
    pub name: String,
    pub object_type: String,
    pub location: [f32; 3],
    pub rotation: [f32; 3],   // Euler
    pub quaternion: [f32; 4], // Quaternion (w,x,y,z)
    pub scale: [f32; 3],
    pub matrix_world: [[f32; 4]; 4],
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
    /// Per-loop normals (one per loop/corner, indexed same as face loops).
    /// Empty when the file carries no custom or smooth normals.
    pub loop_normals: Vec<[f32; 3]>,
    /// Per-loop UV coordinates (one per loop/corner). Empty when no UV layer
    /// is present in the mesh.
    pub loop_uvs: Vec<[f32; 2]>,
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
    pub base_color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    /// Base Color texture path (Image Texture node), if any.
    pub base_color_texture: Option<String>,
}

impl crate::phx::MeshSource for BlendSceneObject {
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
        crate::phx::SubjectSelector::select_primary_subject(
            &self.objects,
            self.active_object,
            |o| o.vertices.len(),
            |o| o.name.as_str(),
        )
        .map(|i| &self.objects[i])
    }
}

#[cfg(all(test, feature = "proptest"))]
mod primary_object_tests {
    use super::*;
    use proptest::prelude::*;

    /// Build a minimal `BlendSceneObject` with the given name and vertex count.
    /// Only `name` and `vertices` participate in primary-object selection; the
    /// remaining fields are inert defaults.
    fn obj_named(name: &str, verts: usize) -> BlendSceneObject {
        BlendSceneObject {
            name: name.to_string(),
            object_type: "MESH".to_string(),
            location: [0.0; 3],
            rotation: [0.0; 3],
            quaternion: [1.0, 0.0, 0.0, 0.0],
            scale: [1.0; 3],
            matrix_world: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            vertices: vec![[0.0, 0.0, 0.0]; verts],
            faces: Vec::new(),
            loop_normals: Vec::new(),
            loop_uvs: Vec::new(),
            modifiers: Vec::new(),
            instance_targets: Vec::new(),
            is_instance: false,
            base_color: [0.8, 0.8, 0.8, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            materials: Vec::new(),
            texture_paths: Vec::new(),
        }
    }

    fn scene_with(objects: Vec<BlendSceneObject>, active: Option<usize>) -> BlendScene {
        BlendScene {
            name: "test".to_string(),
            collections: Vec::new(),
            objects,
            active_object: active,
            selected: Vec::new(),
        }
    }

    // ---- Task 4.5: unit tests for object selection edge cases ----

    /// Active-present: the recorded active object is chosen even when it is not
    /// the largest mesh. _Requirements: 7.1._
    #[test]
    fn primary_object_active_present_takes_precedence() {
        let scene = scene_with(vec![obj_named("small", 3), obj_named("big", 10)], Some(0));
        assert_eq!(scene.primary_object().unwrap().name, "small");
    }

    /// Fallback-to-largest: with no active object, the largest mesh by vertex
    /// count wins. _Requirements: 7.1._
    #[test]
    fn primary_object_falls_back_to_largest_mesh() {
        let scene = scene_with(vec![obj_named("small", 3), obj_named("big", 10)], None);
        assert_eq!(scene.primary_object().unwrap().name, "big");
    }

    /// Tie-break: equal vertex counts resolve to the smaller name, independent
    /// of object order. _Requirements: 7.1._
    #[test]
    fn primary_object_tie_break_by_name_is_order_independent() {
        let a = scene_with(vec![obj_named("beta", 4), obj_named("alpha", 4)], None);
        let b = scene_with(vec![obj_named("alpha", 4), obj_named("beta", 4)], None);
        assert_eq!(a.primary_object().unwrap().name, "alpha");
        assert_eq!(b.primary_object().unwrap().name, "alpha");
    }

    /// Empty scene: no primary object. _Requirements: 7.2._
    #[test]
    fn primary_object_empty_scene_is_none() {
        let scene = scene_with(Vec::new(), None);
        assert!(scene.primary_object().is_none());
    }

    /// Out-of-range active index guard: an active index `>= objects.len()` is
    /// ignored and selection falls back to the largest mesh. _Requirements: 7.5._
    #[test]
    fn primary_object_out_of_range_active_falls_back() {
        let scene = scene_with(vec![obj_named("small", 3), obj_named("big", 10)], Some(9));
        assert_eq!(scene.primary_object().unwrap().name, "big");
    }

    // ---- Task 4.4: Property 7 — Primary-object determinism ----

    /// Reference selection mirroring the specification: active-if-valid, else
    /// max vertices with a smaller-name tie-break. Returns the observable
    /// identity `(name, vertex_count)` of the expected primary object.
    fn expected_primary(
        specs: &[(String, usize)],
        active: Option<usize>,
    ) -> Option<(String, usize)> {
        if specs.is_empty() {
            return None;
        }
        if let Some(i) = active {
            if i < specs.len() {
                return Some((specs[i].0.clone(), specs[i].1));
            }
        }
        let mut best = 0usize;
        for i in 1..specs.len() {
            let (vi, vb) = (specs[i].1, specs[best].1);
            if vi > vb || (vi == vb && specs[i].0 < specs[best].0) {
                best = i;
            }
        }
        Some((specs[best].0.clone(), specs[best].1))
    }

    proptest! {
        /// **Validates: Requirements 7.1**
        ///
        /// Property 7: Primary-object determinism — `primary_object(scene)` is
        /// deterministic and independent of `objects` ordering for equal-vertex
        /// ties (name tie-break). We verify that across arbitrary permutations
        /// of the object list the resolved primary object has the same
        /// observable identity `(name, vertex_count)`, and that repeated calls
        /// agree.
        #[test]
        fn primary_object_is_order_independent_for_ties(
            // Small vertex range forces frequent equal-vertex ties; names are a
            // short alphabet so the name tie-break is exercised.
            specs in prop::collection::vec(("[a-c]{1,3}", 0usize..4usize), 1..8)
        ) {
            let specs: Vec<(String, usize)> = specs;
            let expected = expected_primary(&specs, None).unwrap();

            // Several representative orderings of the same object set.
            let mut orderings: Vec<Vec<(String, usize)>> = Vec::new();
            orderings.push(specs.clone());
            let mut rev = specs.clone();
            rev.reverse();
            orderings.push(rev);
            let mut asc = specs.clone();
            asc.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
            orderings.push(asc);
            let mut desc = specs.clone();
            desc.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
            orderings.push(desc);

            for ordering in &orderings {
                let objects: Vec<BlendSceneObject> =
                    ordering.iter().map(|(n, v)| obj_named(n, *v)).collect();
                let scene = scene_with(objects, None);

                let p1 = scene.primary_object().expect("non-empty scene has a primary");
                let observed = (p1.name.clone(), p1.vertices.len());

                // Order independence: matches the reference selection.
                prop_assert_eq!(&observed, &expected);

                // Determinism: a second call agrees with the first.
                let p2 = scene.primary_object().expect("non-empty scene has a primary");
                prop_assert_eq!((p2.name.clone(), p2.vertices.len()), observed);
            }
        }

        /// **Validates: Requirements 7.1**
        ///
        /// A recorded, in-range active object is always selected regardless of
        /// object ordering (its observable identity is stable under
        /// permutation of the surrounding objects).
        #[test]
        fn primary_object_active_selection_is_stable(
            specs in prop::collection::vec(("[a-c]{1,3}", 0usize..4usize), 1..8),
            active_seed in 0usize..8,
        ) {
            let specs: Vec<(String, usize)> = specs;
            let active = active_seed % specs.len();
            let expected = (specs[active].0.clone(), specs[active].1);

            let objects: Vec<BlendSceneObject> =
                specs.iter().map(|(n, v)| obj_named(n, *v)).collect();
            let scene = scene_with(objects, Some(active));

            let p = scene.primary_object().expect("non-empty scene has a primary");
            prop_assert_eq!((p.name.clone(), p.vertices.len()), expected);
        }
    }
}
