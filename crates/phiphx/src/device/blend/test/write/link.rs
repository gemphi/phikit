use crate::device::blend::write::link::*;
use crate::device::blend::write::mesh::BlendNewMesh;

fn triangle(name: &str) -> BlendNewMesh {
    BlendNewMesh { name: name.into(), vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], faces: vec![vec![0, 1, 2]] }
}

#[test]
fn a_linked_object_is_in_the_scene_and_an_unlinked_one_is_gone_with_its_mesh() {
    for name in ["nla_cube.blend", "THYMUS-BOLDROID-WAVE-003.blend"] {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        let mut file = BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse");
        let before = file.extract_scene().expect("scene");
        let scene = file.current_scene_index().unwrap();
        let view_layer = file.view_layer_index(scene).unwrap();
        let master = file.collections_at(scene, view_layer)[0].collection;
        let (_, mesh_address) = file.add_mesh(file.mesh_template().unwrap(), &triangle("Tri")).unwrap();
        let (_, object_address) = file.add_object(file.mesh_object_template().unwrap(), "Tri", mesh_address, [0.0; 3], [1.0; 3]).unwrap();
        // Indices moved: find them again.
        let scene = file.current_scene_index().unwrap();
        let view_layer = file.view_layer_index(scene).unwrap();
        let master = file.collections_at(scene, view_layer).iter().find(|c| c.collection_address == file.blocks[master.min(file.blocks.len() - 1)].old_address || c.path == "Scene Collection").map(|c| c.collection).unwrap();
        file.link_object(scene, view_layer, master, object_address, 1).expect("linked");
        let after = BlendFile::parse_bytes(&file.to_bytes()).unwrap().extract_scene().unwrap();
        assert_eq!(after.objects.len(), before.objects.len() + 1, "{name}");
        let tri = after.objects.iter().position(|o| o.name == "Tri").unwrap_or_else(|| panic!("{name}: Tri is in the scene"));
        assert_eq!(after.objects[tri].vertices.len(), 3);
        assert!(after.collections[0].object_indices.contains(&tri) || after.collections[0].object_names.contains(&"Tri".to_string()), "{name}: in the scene collection");
        assert!(after.selected.contains(&tri), "{name}: its base is selected");

        // Now remove the first original object: gone, with its mesh.
        let victim = before.objects[0].name.clone();
        let mut file = BlendFile::parse_bytes(&file.to_bytes()).unwrap();
        let scene = file.current_scene_index().unwrap();
        let view_layer = file.view_layer_index(scene).unwrap();
        let ob = file.find_id("OB", &victim).unwrap();
        let blocks = file.blocks.len();
        let removed = file.remove_object(scene, view_layer, ob);
        assert!(removed >= 3, "{name}: the object, its link, its base: {removed}");
        assert_eq!(file.blocks.len(), blocks - removed);
        let after = BlendFile::parse_bytes(&file.to_bytes()).unwrap().extract_scene().unwrap();
        assert!(!after.objects.iter().any(|o| o.name == victim), "{name}: {victim} is gone");
        assert_eq!(after.objects.len(), before.objects.len(), "{name}: one in, one out");
        assert!(after.objects.iter().any(|o| o.name == "Tri"));
    }
}
