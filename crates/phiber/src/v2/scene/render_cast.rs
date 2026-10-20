//! What a render is handed, and why it is never handed nothing.
//!
//! F12 on pose 26 reached the tracer with an EMPTY scene — 995 objects in
//! the file, 967 drawn in the viewport, none in the render — and the only
//! evidence was one journal line. The list a render draws goes through two
//! filters the viewport does not: the Outliner's render restrictions
//! (`pose::visible_in_render`) and the exact pose at the frame
//! (`BPosed::exact`), and when either leaves nothing there was no second
//! look. This is the second look.
//!
//! It counts what each step kept — objects, then objects with geometry —
//! and when the render would be of nothing while the viewport shows a scene,
//! it says so in the Info editor and renders what the viewport shows: the
//! objects the eye leaves visible, with their meshes as loaded. That is a
//! divergence from Blender, which renders an empty frame when every object is
//! disabled in renders, and it is deliberate: a demo that renders black
//! because one collection checkbox is misread is worse than a render that
//! says which rule it set aside. The note names the rule, so the misread
//! can be found.

use super::BObject3D;
use crate::phiphx::scene::SceneModel;
use crate::v2::state::BState;

/// The objects a render draws, and the note to report when they are not
/// the ones the rules would have given.
pub struct BRenderCast {
    pub objects: Vec<BObject3D>,
    /// `None` when the render rules produced a scene with geometry.
    pub note: Option<String>,
}

impl BRenderCast {
    /// The cast for this frame: the render rules' objects, or the fallback
    /// with its note.
    pub fn of(state: &BState, loaded: Option<&SceneModel>) -> Self {
        let all = super::current_scene(state, loaded);
        let total = all.len();
        let by_rules = super::pose::visible_in_render(state, all.clone());
        let kept = by_rules.len();
        let with_geometry = Self::count_geometry(&by_rules);
        if with_geometry > 0 || total == 0 {
            return Self { objects: by_rules, note: None };
        }
        // Nothing would be drawn. Say which step emptied it, then fall back.
        let shown = super::pose::visible_in_viewport(state, all);
        let shown_geometry = Self::count_geometry(&shown);
        let step = if kept == 0 {
            let culprits = Self::culprits(state, loaded);
            format!(
                "the Outliner's render restrictions disable all {total} objects — collections with the camera off or excluded: {}",
                if culprits.is_empty() { "none; every object carries its own camera-off flag".to_string() } else { culprits.join(", ") }
            )
        } else {
            format!("{kept} of {total} objects pass the render restrictions but none carries geometry at this frame")
        };
        let (objects, drew) = if shown_geometry > 0 {
            (shown, format!("rendering the {shown_geometry} objects the viewport shows instead"))
        } else {
            // Even the viewport's set is empty: the meshes as loaded, unposed.
            let raw = super::pose::BPosed::default().scene(state, loaded);
            let n = Self::count_geometry(&raw);
            (raw, format!("rendering the file's {n} meshes as loaded instead"))
        };
        Self { objects, note: Some(format!("Render would be empty: {step} — {drew}")) }
    }

    /// The collections whose own row would hide a render — the camera icon
    /// off, or the checkbox unticked — by their outliner ids, so the note
    /// can name the rule that emptied the render. At most eight.
    pub fn culprits(state: &BState, loaded: Option<&SceneModel>) -> Vec<String> {
        use crate::v2::part::outliner::key::{is_excluded, is_render_hidden};
        use crate::v2::part::outliner::layer::collection_id;
        use crate::v2::scene::links::{BCollectionNode, BSceneTree};
        fn walk(c: &BCollectionNode, state: &BState, out: &mut Vec<String>) {
            let id = collection_id(&c.path);
            match (is_render_hidden(state, id), is_excluded(state, id)) {
                (true, _) => out.push(format!("{id} (camera off)")),
                (false, true) => out.push(format!("{id} (excluded)")),
                (false, false) => {}
            }
            for k in &c.children {
                walk(k, state, out);
            }
        }
        let Some(model) = loaded else { return Vec::new() };
        let tree = BSceneTree::now(state, model);
        let mut out = Vec::new();
        for c in &tree.collections {
            walk(c, state, &mut out);
        }
        out.truncate(8);
        out
    }

    /// How many objects carry triangles — a primitive always does.
    pub fn count_geometry(objects: &[BObject3D]) -> usize {
        objects
            .iter()
            .filter(|o| match o {
                BObject3D::Primitive { .. } => true,
                BObject3D::Loaded { mesh, .. } => !mesh.indices.is_empty(),
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::scene::{SceneCorners, SceneMesh, SceneNormals, SceneObject};

    fn mesh(tris: usize) -> SceneMesh {
        SceneMesh {
            vertices: vec![[0.0; 3]; 3],
            indices: (0..tris * 3).map(|i| (i % 3) as u32).collect(),
            normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]; tris]),
            corners: SceneCorners::default(),
        }
    }

    fn model(names: &[(&str, usize)]) -> SceneModel {
        SceneModel {
            objects: names.iter().map(|(n, t)| SceneObject::identity(*n, mesh(*t))).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn a_scene_the_rules_keep_is_handed_over_unchanged_and_unremarked() {
        let model = model(&[("Car", 4), ("Floor", 2)]);
        let cast = BRenderCast::of(&BState::new(), Some(&model));
        assert_eq!(cast.objects.len(), 2);
        assert!(cast.note.is_none(), "{:?}", cast.note);
    }

    #[test]
    fn every_object_disabled_in_renders_falls_back_to_the_viewport_with_a_note() {
        use crate::v2::part::outliner::key::k_restrict;
        use crate::v2::state::val::V_TRUE;
        let model = model(&[("Car", 4), ("Floor", 2)]);
        let mut s = BState::new();
        for name in ["Car", "Floor"] {
            s.set(&k_restrict(name, "hide_render"), V_TRUE);
        }
        let cast = BRenderCast::of(&s, Some(&model));
        assert_eq!(cast.objects.len(), 2, "the viewport's two are rendered");
        let note = cast.note.expect("the fallback is reported");
        assert!(note.contains("render restrictions") && note.contains("all 2 objects"), "{note}");
    }

    #[test]
    fn one_object_left_in_is_enough_and_nothing_is_said() {
        use crate::v2::part::outliner::key::k_restrict;
        use crate::v2::state::val::V_TRUE;
        let model = model(&[("Car", 4), ("Floor", 2)]);
        let mut s = BState::new();
        s.set(&k_restrict("Car", "hide_render"), V_TRUE);
        let cast = BRenderCast::of(&s, Some(&model));
        assert_eq!(cast.objects.len(), 1);
        assert!(cast.note.is_none());
    }

    #[test]
    fn geometry_is_counted_not_assumed() {
        let model = model(&[("Empty", 0), ("Floor", 2)]);
        let scene = super::super::current_scene(&BState::new(), Some(&model));
        assert_eq!(BRenderCast::count_geometry(&scene), 1);
    }
}
