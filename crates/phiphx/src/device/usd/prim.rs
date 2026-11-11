//! USD prims — the nodes of the USD scene graph, addressed by path.

use crate::device::usd::attribute::{UsdAttribute, UsdValue};

/// A prim (primitive) in a USD stage: a typed, named node with attributes and
/// children, addressed by an absolute path like `/Root/FlatTerrain`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UsdPrim {
    /// Absolute prim path (e.g. `/Root/Diorama_Base`).
    pub path: String,
    /// Prim type name (e.g. `Mesh`, `Xform`, `Material`).
    pub type_name: String,
    pub attributes: Vec<UsdAttribute>,
    pub children: Vec<UsdPrim>,
}

impl UsdPrim {
    pub fn new(path: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            type_name: type_name.into(),
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    /// The leaf name of this prim (last path segment).
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    /// Read an attribute value by name.
    pub fn attribute(&self, name: &str) -> Option<&UsdValue> {
        self.attributes.iter().find(|a| a.name == name).map(|a| &a.value)
    }

    /// Author (insert or replace) an attribute.
    pub fn set_attribute(&mut self, name: impl Into<String>, value: UsdValue) {
        let name = name.into();
        if let Some(a) = self.attributes.iter_mut().find(|a| a.name == name) {
            a.value = value;
        } else {
            self.attributes.push(UsdAttribute::new(name, value));
        }
    }

    /// Add a child prim.
    pub fn add_child(&mut self, child: UsdPrim) {
        self.children.push(child);
    }

    /// Whether this prim has geometry points (`points` attribute present).
    pub fn is_mesh(&self) -> bool {
        self.type_name == "Mesh" || self.attribute("points").is_some()
    }
}
