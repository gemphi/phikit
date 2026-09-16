//! Unit tests for Blender native scene export.

use super::*;

#[test]
fn test_default_scene_export_config() {
    let config = SceneExportConfig::default();
    assert!(config.include_meshes);
    assert!(config.include_lights);
    assert!(config.include_camera);
    assert!(config.include_instances);
}

#[test]
fn test_parse_object_type() {
    assert!(matches!(parse_object_type("MESH"), BlenderObjectType::Mesh));
    assert!(matches!(
        parse_object_type("LIGHT"),
        BlenderObjectType::Light
    ));
    assert!(matches!(
        parse_object_type("CAMERA"),
        BlenderObjectType::Camera
    ));
}

#[test]
fn test_parse_light_type() {
    assert!(matches!(parse_light_type("POINT"), BlenderLightType::Point));
    assert!(matches!(parse_light_type("SUN"), BlenderLightType::Sun));
    assert!(matches!(parse_light_type("SPOT"), BlenderLightType::Spot));
    assert!(matches!(parse_light_type("AREA"), BlenderLightType::Area));
}

#[test]
fn test_export_scene_to_phx_roundtrip() {
    // Build a test scene — same as what the Python would have produced
    let scene = BlenderScene {
        name: "TestScene".to_string(),
        frame: 1,
        resolution: [1920, 1080],
        camera: Some(CameraData {
            name: "Camera".to_string(),
            location: [0.0, -5.0, 2.0],
            rotation: [1.1, 0.0, 0.0],
            fov: 0.6911,
            clip_start: 0.1,
            clip_end: 1000.0,
            camera_type: CameraType::Perspective,
            matrix_world: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, -5.0, 2.0, 1.0],
            ],
        }),
        lights: vec![LightData {
            name: "Sun".to_string(),
            light_type: BlenderLightType::Sun,
            color: [1.0, 0.95, 0.9],
            energy: 5.0,
            location: [0.0, 0.0, 10.0],
            direction: [0.0, 0.0, -1.0],
            matrix_world: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 10.0, 1.0],
            ],
            spot_angle: None,
            spot_blend: None,
            area_size: None,
        }],
        meshes: vec![MeshObjectData {
            name: "Cube".to_string(),
            vertices: vec![
                [1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0],
                [-1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0],
            ],
            normals: vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            colors: vec![],
            indices: vec![[0, 1, 2], [0, 2, 3]],
            material_names: vec!["Default".to_string()],
            matrix_world: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            visible: true,
        }],
        instances: vec![],
        materials: vec![MaterialData {
            name: "Default".to_string(),
            base_color: [0.8, 0.8, 0.8, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emission: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            alpha: 1.0,
            use_nodes: true,
            base_color_texture: None,
            normal_texture: None,
            roughness_texture: None,
        }],
        collections: vec![],
    };

    // Export to PHX
    let bytes = export_scene_to_phx(&scene).unwrap();

    // Verify magic
    assert_eq!(&bytes[0..10], b"PHX_SCENE\x01", "Magic header mismatch");

    // Verify JSON length
    let json_len = u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]) as usize;
    assert_eq!(
        bytes.len(),
        14 + json_len,
        "File size should be header + JSON"
    );

    // Parse JSON back
    let json_str = std::str::from_utf8(&bytes[14..]).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();

    assert_eq!(parsed["name"], "TestScene");
    assert_eq!(parsed["format"], "phx-scene-v1");
    assert_eq!(parsed["frame"], 1);
    assert_eq!(parsed["resolution"][0], 1920);
    assert_eq!(parsed["resolution"][1], 1080);
    assert_eq!(parsed["camera"]["name"], "Camera");
    assert_eq!(parsed["lights"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["lights"][0]["type"], "SUN");
    assert_eq!(parsed["meshes"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["meshes"][0]["name"], "Cube");
    assert_eq!(parsed["meshes"][0]["vertices"].as_array().unwrap().len(), 4);
    assert_eq!(parsed["meshes"][0]["indices"].as_array().unwrap().len(), 2);
    assert_eq!(parsed["materials"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["materials"][0]["name"], "Default");

    println!(
        "PHX scene export: {} bytes, {} JSON bytes",
        bytes.len(),
        json_len
    );
}
