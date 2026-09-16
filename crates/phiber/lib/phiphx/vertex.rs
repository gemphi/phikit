//! Vertex types and their serialization.

/// A single vertex/point in 3D space
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex {
    /// Position [x, y, z]
    pub position: [f32; 3],
    /// Normal [nx, ny, nz] (optional)
    pub normal: Option<[f32; 3]>,
    /// Color [r, g, b, a] (optional, 0-255)
    pub color: Option<[u8; 4]>,
    /// Additional attributes
    pub attributes: VertexAttributes,
}

/// Additional vertex attributes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VertexAttributes {
    /// Texture coordinates [u, v]
    pub uv: Option<[f32; 2]>,
    /// Intensity value (for LiDAR data)
    pub intensity: Option<f32>,
    /// Classification (for LAS files)
    pub classification: Option<u8>,
}

impl Vertex {
    /// Create a new vertex at the given position
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x, y, z],
            normal: None,
            color: None,
            attributes: VertexAttributes::default(),
        }
    }

    /// Set the normal vector
    pub fn with_normal(mut self, nx: f32, ny: f32, nz: f32) -> Self {
        self.normal = Some([nx, ny, nz]);
        self
    }

    /// Set the color
    pub fn with_color(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.color = Some([r, g, b, a]);
        self
    }

    /// Set UV coordinates
    pub fn with_uv(mut self, u: f32, v: f32) -> Self {
        self.attributes.uv = Some([u, v]);
        self
    }

    /// Set intensity
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.attributes.intensity = Some(intensity);
        self
    }

    /// Apply a 4x4 transformation matrix
    pub fn transform(&self, matrix: &[f32; 16]) -> Self {
        let x = self.position[0];
        let y = self.position[1];
        let z = self.position[2];

        let tx = matrix[0] * x + matrix[1] * y + matrix[2] * z + matrix[3];
        let ty = matrix[4] * x + matrix[5] * y + matrix[6] * z + matrix[7];
        let tz = matrix[8] * x + matrix[9] * y + matrix[10] * z + matrix[11];
        let w = matrix[12] * x + matrix[13] * y + matrix[14] * z + matrix[15];

        let w_inv = if w != 0.0 { 1.0 / w } else { 1.0 };

        let mut result = self.clone();
        result.position = [tx * w_inv, ty * w_inv, tz * w_inv];

        // Transform normal if present
        if let Some(n) = self.normal {
            let nx = matrix[0] * n[0] + matrix[1] * n[1] + matrix[2] * n[2];
            let ny = matrix[4] * n[0] + matrix[5] * n[1] + matrix[6] * n[2];
            let nz = matrix[8] * n[0] + matrix[9] * n[1] + matrix[10] * n[2];
            result.normal = Some([nx, ny, nz]);
        }

        result
    }

    /// Calculate distance to another vertex
    pub fn distance_to(&self, other: &Vertex) -> f32 {
        let dx = self.position[0] - other.position[0];
        let dy = self.position[1] - other.position[1];
        let dz = self.position[2] - other.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl serde::Serialize for Vertex {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Vertex", 4)?;
        state.serialize_field("position", &self.position)?;
        state.serialize_field("normal", &self.normal)?;
        state.serialize_field("color", &self.color)?;
        state.serialize_field("attributes", &self.attributes)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Vertex {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct VertexData {
            position: [f32; 3],
            normal: Option<[f32; 3]>,
            color: Option<[u8; 4]>,
            #[serde(default)]
            attributes: VertexAttributes,
        }

        let data = VertexData::deserialize(deserializer)?;

        Ok(Self {
            position: data.position,
            normal: data.normal,
            color: data.color,
            attributes: data.attributes,
        })
    }
}

impl serde::Serialize for VertexAttributes {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("VertexAttributes", 3)?;
        state.serialize_field("uv", &self.uv)?;
        state.serialize_field("intensity", &self.intensity)?;
        state.serialize_field("classification", &self.classification)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for VertexAttributes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize, Default)]
        struct AttrData {
            uv: Option<[f32; 2]>,
            intensity: Option<f32>,
            classification: Option<u8>,
        }

        let data = AttrData::deserialize(deserializer)?;

        Ok(Self {
            uv: data.uv,
            intensity: data.intensity,
            classification: data.classification,
        })
    }
}
