//! Scene Data Types — Mirrors RealityKit scene structure
//!
//! Corresponds to: .scene.json files in Reality Composer Pro

use serde::{Deserialize, Serialize};
use serde_json;

/// A scene file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneData {
    pub name: String,
    pub entities: Vec<EntityData>,
    #[serde(default)]
    pub environment: EnvironmentData,
}

/// Serialized entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<u64>,
    pub transform: TransformData,
    #[serde(default)]
    pub components: Vec<ComponentData>,
    #[serde(default)]
    pub children: Vec<u64>,
}

/// Transform (position, rotation, scale)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformData {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for TransformData {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

/// A component attached to an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentData {
    pub component_type: String,
    pub data: serde_json::Value,
}

/// Scene environment (skybox, IBL, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentData {
    #[serde(default = "default_skybox")]
    pub skybox: String,
    #[serde(default = "default_exposure")]
    pub exposure: f32,
}

fn default_skybox() -> String {
    "default".to_string()
}
fn default_exposure() -> f32 {
    1.0
}

impl Default for EnvironmentData {
    fn default() -> Self {
        Self {
            skybox: default_skybox(),
            exposure: default_exposure(),
        }
    }
}
