//! Inspector model — data types for the property inspector panel.

/// A single property row in the inspector.
#[derive(Debug, Clone)]
pub struct NSInspectorProperty {
    pub label: String,
    pub value: NSInspectorValue,
}

/// Value types the inspector can display/edit.
#[derive(Debug, Clone)]
pub enum NSInspectorValue {
    /// A 3-component vector (position, rotation, scale).
    Vec3([f32; 3]),
    /// A single float value.
    Float(f32),
    /// A string value (name, path, etc.).
    Text(String),
    /// A color value (RGBA).
    Color([f32; 4]),
    /// A boolean toggle.
    Bool(bool),
    /// An integer value.
    Int(i64),
}

/// A section of properties grouped under a header.
#[derive(Debug, Clone)]
pub struct NSInspectorSection {
    pub title: String,
    pub properties: Vec<NSInspectorProperty>,
    pub collapsed: bool,
}

/// Full inspector state for a selected entity.
#[derive(Debug, Clone, Default)]
pub struct NSInspectorState {
    /// Display name of the selected entity (or "No Selection").
    pub entity_name: Option<String>,
    /// Entity ID if something is selected.
    pub entity_id: Option<u64>,
    /// Sections of properties to display.
    pub sections: Vec<NSInspectorSection>,
}

impl NSInspectorState {
    /// Returns true if nothing is selected.
    pub fn is_empty(&self) -> bool {
        self.entity_id.is_none()
    }

    /// Convenience: create a state with transform section populated.
    pub fn with_transform(
        entity_name: &str,
        entity_id: u64,
        position: [f32; 3],
        rotation: [f32; 3],
        scale: [f32; 3],
    ) -> Self {
        Self {
            entity_name: Some(entity_name.to_string()),
            entity_id: Some(entity_id),
            sections: vec![NSInspectorSection {
                title: "Transform".to_string(),
                properties: vec![
                    NSInspectorProperty {
                        label: "Position".to_string(),
                        value: NSInspectorValue::Vec3(position),
                    },
                    NSInspectorProperty {
                        label: "Rotation".to_string(),
                        value: NSInspectorValue::Vec3(rotation),
                    },
                    NSInspectorProperty {
                        label: "Scale".to_string(),
                        value: NSInspectorValue::Vec3(scale),
                    },
                ],
                collapsed: false,
            }],
        }
    }
}
