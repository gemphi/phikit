//! PHX scene format writer (binary: magic + JSON).

use super::types::{BlenderLightType, BlenderScene, CameraType};
use crate::phx::{PointCloudError, Result};

/// Export a BlenderScene to PHX scene format (binary: magic + JSON)
///
/// This is the Rust equivalent of the deleted Python `export_phx_scene()`.
/// Output format: `PHX_SCENE\x01` + u32 JSON length + JSON bytes
pub fn export_scene_to_phx(scene: &BlenderScene) -> Result<Vec<u8>> {
    use serde_json::json;

    // Build camera JSON
    let camera_json = scene.camera.as_ref().map(|cam| {
        json!({
            "name": cam.name,
            "type": match cam.camera_type {
                CameraType::Perspective => "perspective",
                CameraType::Orthographic => "orthographic",
                CameraType::Panoramic => "panoramic",
            },
            "location": cam.location,
            "rotation": cam.rotation,
            "matrix_world": cam.matrix_world,
            "fov": cam.fov,
            "clip_start": cam.clip_start,
            "clip_end": cam.clip_end,
        })
    });

    // Build lights JSON
    let lights_json: Vec<serde_json::Value> = scene
        .lights
        .iter()
        .map(|light| {
            let mut data = json!({
                "name": light.name,
                "type": match light.light_type {
                    BlenderLightType::Point => "POINT",
                    BlenderLightType::Sun => "SUN",
                    BlenderLightType::Spot => "SPOT",
                    BlenderLightType::Area => "AREA",
                },
                "color": light.color,
                "energy": light.energy,
                "location": light.location,
                "direction": light.direction,
                "matrix_world": light.matrix_world,
            });
            if let Some(angle) = light.spot_angle {
                data["spot_angle"] = json!(angle);
            }
            if let Some(blend) = light.spot_blend {
                data["spot_blend"] = json!(blend);
            }
            if let Some(size) = light.area_size {
                data["area_size"] = json!(size);
            }
            data
        })
        .collect();

    // Build meshes JSON
    let meshes_json: Vec<serde_json::Value> = scene
        .meshes
        .iter()
        .map(|mesh| {
            json!({
                "name": mesh.name,
                "vertices": mesh.vertices,
                "normals": mesh.normals,
                "uvs": mesh.uvs,
                "indices": mesh.indices,
                "materials": mesh.material_names,
                "matrix_world": mesh.matrix_world,
                "visible": mesh.visible,
            })
        })
        .collect();

    // Build materials JSON
    let materials_json: Vec<serde_json::Value> = scene
        .materials
        .iter()
        .map(|mat| {
            json!({
                "name": mat.name,
                "base_color": mat.base_color,
                "metallic": mat.metallic,
                "roughness": mat.roughness,
                "emission": mat.emission,
                "emission_strength": mat.emission_strength,
                "alpha": mat.alpha,
            })
        })
        .collect();

    // Build instances JSON
    let instances_json: Vec<serde_json::Value> = scene
        .instances
        .iter()
        .map(|inst| {
            json!({
                "prototype": inst.prototype_name,
                "transforms": inst.transforms,
            })
        })
        .collect();

    // Assemble full scene JSON
    let scene_json = json!({
        "name": scene.name,
        "frame": scene.frame,
        "resolution": scene.resolution,
        "format": "phx-scene-v1",
        "camera": camera_json,
        "lights": lights_json,
        "meshes": meshes_json,
        "materials": materials_json,
        "instances": instances_json,
    });

    let json_bytes = serde_json::to_vec(&scene_json)
        .map_err(|e| PointCloudError::Serialization(format!("JSON serialization failed: {}", e)))?;

    // Write PHX scene file: magic + JSON length (u32 LE) + JSON bytes
    let mut output = Vec::with_capacity(14 + json_bytes.len());
    output.extend_from_slice(b"PHX_SCENE\x01");
    output.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    output.extend_from_slice(&json_bytes);

    Ok(output)
}
