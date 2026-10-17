//! `Scenes/<name>.scene.json` — the entity graph.
//!
//! One root entity called "Scene", and one child per object the asset
//! held. The ids are positional: the root is 1 and object `i` is `i + 2`,
//! which is what the app's loader expects.
//!
//! The root's `children` and each child's `parent_id` are two statements of
//! the same fact, so they are built from one walk rather than from two
//! loops that can disagree — which the version this replaces did, with the
//! root's child list built from a range and the children from an
//! enumeration.

use glam::Vec3;
use serde::Serialize;

/// Where an entity sits. A freshly imported object is at the origin: the
/// asset's own transform is baked into its mesh by the importer, so a
/// second copy here would apply it twice.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct BTransform {
    pub position: [f32; 3],
    /// `xyzw`, as the loader reads a quaternion.
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for BTransform {
    fn default() -> Self {
        Self { position: [0.0; 3], rotation: [0.0, 0.0, 0.0, 1.0], scale: [1.0; 3] }
    }
}

/// What an entity is made of. Only a model so far; the enum is what keeps
/// the next kind from being a bare field nothing reads.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BComponent {
    /// A mesh the GPU loader fetches, and whether it is drawn.
    Model { mesh_ref: String, visible: bool },
}

/// One entity of the scene.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BEntity {
    pub id: u64,
    pub name: String,
    pub parent_id: Option<u64>,
    pub transform: BTransform,
    pub component: Vec<BComponent>,
    pub children: Vec<u64>,
}

/// The scene file.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BScene {
    pub name: String,
    /// The bounding sphere the importer measured, for the camera to frame.
    /// `None` when the asset held nothing to measure.
    pub bounds: Option<BBounds>,
    pub entity: Vec<BEntity>,
}

/// A bounding sphere: where it is and how big.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct BBounds {
    pub center: [f32; 3],
    pub radius: f32,
}

impl BScene {
    /// The id of the root entity every scene has.
    pub const ROOT: u64 = 1;

    /// A scene called `name`, holding one entity per name in `object`, each
    /// referring back into `mesh` by its index.
    pub fn new(name: &str, object: &[String], mesh: &str, bounds: Option<(Vec3, f32)>) -> Self {
        let child: Vec<u64> = (0..object.len() as u64).map(|i| i + Self::ROOT + 1).collect();
        let root = BEntity {
            id: Self::ROOT,
            name: "Scene".to_string(),
            parent_id: None,
            transform: BTransform::default(),
            component: Vec::new(),
            children: child.clone(),
        };
        let entity = std::iter::once(root)
            .chain(object.iter().zip(&child).enumerate().map(|(index, (object, id))| BEntity {
                id: *id,
                name: object.clone(),
                parent_id: Some(Self::ROOT),
                transform: BTransform::default(),
                component: vec![BComponent::Model { mesh_ref: format!("{mesh}?objects={index}"), visible: true }],
                children: Vec::new(),
            }))
            .collect();
        Self {
            name: name.to_string(),
            bounds: bounds.map(|(center, radius)| BBounds { center: center.to_array(), radius }),
            entity,
        }
    }

    /// The file's bytes, pretty printed and newline terminated.
    pub fn text(&self) -> Result<String, serde_json::Error> {
        Ok(format!("{}\n", serde_json::to_string_pretty(self)?))
    }
}

#[cfg(test)]
#[path = "scene_tests.rs"]
mod scene_tests;
