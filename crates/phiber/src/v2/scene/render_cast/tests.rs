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
