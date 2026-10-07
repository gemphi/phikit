//! The primitive shapes the Add menu offers — Blender's `Add ▸ Mesh`
//! entries, plus `Add ▸ Metaball ▸ Ball` and `Add ▸ Text` — and what each
//! one is to the rest of the app: its label, its slug, its outliner icon
//! and the mesh phiphx builds for it.

/// A primitive shape an object can take. Drives both its viewport silhouette
/// and its outliner icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BShape {
    Plane,
    Cube,
    Circle,
    Sphere,
    IcoSphere,
    Cylinder,
    Cone,
    Torus,
    Grid,
    Monkey,
    /// Blender's `Add ▸ Metaball ▸ Ball`.
    Metaball,
    /// Blender's `Add ▸ Text`, which reads "Text" when it is made.
    Text,
    /// Not Blender's: the icosahedron on its own, kept because files and
    /// tests from before the rest of the primitives existed name it.
    Gem,
}

impl BShape {
    /// Every shape the Add menu offers, in Blender's own order.
    pub const ALL: [Self; 13] = [
        Self::Plane,
        Self::Cube,
        Self::Circle,
        Self::Sphere,
        Self::IcoSphere,
        Self::Cylinder,
        Self::Cone,
        Self::Torus,
        Self::Grid,
        Self::Monkey,
        Self::Metaball,
        Self::Text,
        Self::Gem,
    ];

    pub fn icon(self) -> &'static str {
        match self {
            BShape::Cube | BShape::Grid => "cube",
            BShape::Sphere | BShape::IcoSphere | BShape::Metaball => "sphere",
            BShape::Gem => "diamond",
            _ => "cube",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            BShape::Plane => "Plane",
            BShape::Cube => "Cube",
            BShape::Circle => "Circle",
            BShape::Sphere => "Sphere",
            BShape::IcoSphere => "Ico Sphere",
            BShape::Cylinder => "Cylinder",
            BShape::Cone => "Cone",
            BShape::Torus => "Torus",
            BShape::Grid => "Grid",
            BShape::Monkey => "Monkey",
            BShape::Metaball => "Metaball",
            BShape::Text => "Text",
            BShape::Gem => "Gem",
        }
    }
    /// The slug a state key and a menu action use.
    pub fn slug(self) -> &'static str {
        match self {
            BShape::Plane => "plane",
            BShape::Cube => "cube",
            BShape::Circle => "circle",
            BShape::Sphere => "sphere",
            BShape::IcoSphere => "icosphere",
            BShape::Cylinder => "cylinder",
            BShape::Cone => "cone",
            BShape::Torus => "torus",
            BShape::Grid => "grid",
            BShape::Monkey => "monkey",
            BShape::Metaball => "metaball",
            BShape::Text => "text",
            BShape::Gem => "gem",
        }
    }

    /// The shape a slug names (`slug`), if any.
    pub fn of(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.slug() == slug)
    }
    pub fn mesh_type(self) -> crate::phiphx::spatial::PhiQuiMeshType {
        use crate::phiphx::spatial::PhiQuiMeshType;
        match self {
            BShape::Plane => PhiQuiMeshType::Plane,
            BShape::Cube => PhiQuiMeshType::Cube,
            BShape::Circle => PhiQuiMeshType::Circle,
            BShape::Sphere => PhiQuiMeshType::Sphere,
            BShape::IcoSphere => PhiQuiMeshType::IcoSphere,
            BShape::Cylinder => PhiQuiMeshType::Cylinder,
            BShape::Cone => PhiQuiMeshType::Cone,
            BShape::Torus => PhiQuiMeshType::Torus,
            BShape::Grid => PhiQuiMeshType::Grid,
            BShape::Monkey => PhiQuiMeshType::Suzanne,
            BShape::Metaball => PhiQuiMeshType::Metaball,
            BShape::Text => PhiQuiMeshType::Text,
            BShape::Gem => PhiQuiMeshType::Icosahedron,
        }
    }
}
