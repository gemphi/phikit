//! Primary-object selection tests (split from `types.rs`).

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
            matrix_world: crate::blend::object::transform::IDENTITY4,
            matrix_local: crate::blend::object::transform::IDENTITY4,
            block_address: 0,
            parent_address: 0,
            parentinv: crate::blend::object::transform::IDENTITY4,
            partype: 0,
            parent_verts: [0; 3],
            visibility: Default::default(),
            vertices: vec![[0.0, 0.0, 0.0]; verts],
            faces: Vec::new(),
            loop_normals: Vec::new(),
            loop_uvs: Vec::new(),
            loop_colors: Vec::new(),
            shadow: Default::default(),
            face_materials: Vec::new(),
            face_smooth: Vec::new(),
            data: Default::default(),
            modifiers: Vec::new(),
            skipped_modifiers: Vec::new(),
            instance_targets: Vec::new(),
            is_instance: false,
            instance_source: None,
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
            hidden: Vec::new(),
            props: Default::default(),
            deferred: None,
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
