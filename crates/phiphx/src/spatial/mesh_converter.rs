//! Rust Mesh Space Converter — 2D BUIKit Panel ↔ 3D Spatial Mesh Converter

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhiQuiMeshType {
    Cube,
    Sphere,
    Torus,
    Suzanne,
    Cylinder,
    Icosahedron,
    /// Flat quad on the ground plane — Blender's `Add ▸ Mesh ▸ Plane`.
    Plane,
    /// Regular polygon on the ground plane — Blender's `Add ▸ Mesh ▸ Circle`.
    Circle,
    /// Closed right-cone: apex + base rim + centre — `Add ▸ Mesh ▸ Cone`.
    Cone,
}

impl PhiQuiMeshType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cube => "Cube Mesh",
            Self::Sphere => "UV Sphere Mesh",
            Self::Torus => "Torus Donut Mesh",
            Self::Suzanne => "Suzanne Monkey Mesh",
            Self::Cylinder => "Cylinder Mesh",
            Self::Icosahedron => "Icosahedron Mesh",
            Self::Plane => "Plane Mesh",
            Self::Circle => "Circle Mesh",
            Self::Cone => "Cone Mesh",
        }
    }

    /// Documented approximate vertex-count for UI display. Real geometry
    /// comes from `SpatialObject3D::buffers(mesh_type)`.
    pub fn vertex_count(&self) -> usize {
        match self {
            Self::Cube => 24,
            Self::Sphere => 512,
            Self::Torus => 1024,
            Self::Suzanne => 960,
            Self::Cylinder => 128,
            Self::Icosahedron => 80,
            Self::Plane => 4,
            Self::Circle => 33,
            Self::Cone => 50,
        }
    }

    /// Generate 3D spatial vertex coordinates (x, y, z) for the selected mesh type
    pub fn generate_spatial_vertices(&self) -> Vec<(f32, f32, f32)> {
        match self {
            Self::Cube => vec![
                (-1.0, -1.0, 1.0),
                (1.0, -1.0, 1.0),
                (1.0, 1.0, 1.0),
                (-1.0, 1.0, 1.0),
                (-1.0, -1.0, -1.0),
                (1.0, -1.0, -1.0),
                (1.0, 1.0, -1.0),
                (-1.0, 1.0, -1.0),
            ],
            Self::Plane => vec![
                (-1.0, 0.0, -1.0),
                (1.0, 0.0, -1.0),
                (1.0, 0.0, 1.0),
                (-1.0, 0.0, 1.0),
            ],
            _ => vec![
                (0.0, 1.0, 0.0),
                (-1.0, 0.0, 0.0),
                (1.0, 0.0, 0.0),
                (0.0, -1.0, 0.0),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshSpaceConversion {
    pub mesh_type: PhiQuiMeshType,
    pub is_3d_mode: bool,
    pub spatial_depth: f32,
    pub bounds_radius: f32,
}

impl MeshSpaceConversion {
    pub fn new(mesh_type: PhiQuiMeshType) -> Self {
        Self {
            mesh_type,
            is_3d_mode: true,
            spatial_depth: 12.0,
            bounds_radius: 1.5,
        }
    }

    pub fn convert_to_3d_space(&mut self, depth: f32) {
        self.is_3d_mode = true;
        self.spatial_depth = depth;
    }

    pub fn convert_to_2d_layout(&mut self) {
        self.is_3d_mode = false;
        self.spatial_depth = 0.0;
    }
}

impl Default for MeshSpaceConversion {
    fn default() -> Self {
        Self::new(PhiQuiMeshType::Cube)
    }
}
