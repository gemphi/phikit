//! The entity graph: its ids, and that the two statements of parentage
//! agree.

use super::*;

fn scene(object: &[&str]) -> BScene {
    let object: Vec<String> = object.iter().map(|name| (*name).to_string()).collect();
    BScene::new("rig", &object, "/assets/rig.blend", Some((Vec3::new(1.0, 2.0, 3.0), 4.5)))
}

#[test]
fn a_scene_has_a_root_and_one_entity_per_object() {
    let made = scene(&["Cube", "Lamp"]);
    assert_eq!(made.entity.len(), 3, "the root and two objects");
    assert_eq!(made.entity[0].id, BScene::ROOT);
    assert_eq!(made.entity[0].name, "Scene");
    assert_eq!(made.entity[0].parent_id, None);
    assert_eq!(made.entity[1].id, 2);
    assert_eq!(made.entity[2].id, 3);
}

#[test]
fn the_roots_child_list_and_the_childrens_parents_say_the_same_thing() {
    // Built from two loops, these drifted: the root listed `2..=n+1` from a
    // range while the children were numbered from an enumeration, so one
    // object too few or too many made the scene refer to an entity that was
    // not there.
    let made = scene(&["A", "B", "C"]);
    let root = &made.entity[0];
    let child: Vec<u64> = made.entity.iter().filter(|e| e.parent_id == Some(BScene::ROOT)).map(|e| e.id).collect();
    assert_eq!(root.children, child);
    for id in &root.children {
        assert!(made.entity.iter().any(|e| e.id == *id), "the root's child {id} is an entity");
    }
}

#[test]
fn an_empty_asset_gives_a_root_and_nothing_else() {
    let made = scene(&[]);
    assert_eq!(made.entity.len(), 1);
    assert!(made.entity[0].children.is_empty(), "and no child ids pointing at nothing");
}

#[test]
fn each_object_refers_back_into_the_asset_by_its_index() {
    let made = scene(&["Cube", "Lamp"]);
    let mesh = |entity: &BEntity| match entity.component.first() {
        Some(BComponent::Model { mesh_ref, .. }) => mesh_ref.clone(),
        None => String::new(),
    };
    assert_eq!(mesh(&made.entity[1]), "/assets/rig.blend?objects=0");
    assert_eq!(mesh(&made.entity[2]), "/assets/rig.blend?objects=1");
    assert!(made.entity[0].component.is_empty(), "the root is not a model");
}

#[test]
fn an_objects_name_is_kept_exactly_and_the_json_still_parses() {
    // The predecessor did `replace('"', "'")`, which renames the user's
    // object to keep its own string building working.
    let made = scene(&[r#"He said "hi""#, r#"back\slash"#]);
    let text = made.text().expect("it serialises");
    let back: serde_json::Value = serde_json::from_str(&text).expect("and parses");
    assert_eq!(back["entity"][1]["name"], r#"He said "hi""#);
    assert_eq!(back["entity"][2]["name"], r#"back\slash"#);
}

#[test]
fn a_fresh_entity_sits_at_the_origin_unrotated() {
    // The importer bakes the asset's transform into the mesh, so a second
    // copy here would apply it twice.
    let made = scene(&["Cube"]);
    let at = made.entity[1].transform;
    assert_eq!(at.position, [0.0, 0.0, 0.0]);
    assert_eq!(at.rotation, [0.0, 0.0, 0.0, 1.0], "the identity quaternion is xyzw, so w is last");
    assert_eq!(at.scale, [1.0, 1.0, 1.0]);
}

#[test]
fn the_bounds_are_carried_so_the_camera_can_frame_the_import() {
    let made = scene(&["Cube"]);
    assert_eq!(made.bounds.map(|b| (b.center, b.radius)), Some(([1.0, 2.0, 3.0], 4.5)));
    let none = BScene::new("rig", &[], "rig.blend", None);
    let text = none.text().expect("serialises");
    assert!(text.contains("\"bounds\": null"), "nothing measured is null, not a zero sphere:\n{text}");
}
