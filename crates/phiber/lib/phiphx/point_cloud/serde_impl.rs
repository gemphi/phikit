//! Serialization for `PointCloud`.

use crate::phx::vertex::Vertex;

use super::types::{PointCloud, PointCloudMetadata};

impl serde::Serialize for PointCloud {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PointCloud", 3)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("vertices", &self.vertices)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for PointCloud {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct PointCloudData {
            name: String,
            vertices: Vec<Vertex>,
            metadata: PointCloudMetadata,
        }

        let data = PointCloudData::deserialize(deserializer)?;

        Ok(Self {
            name: data.name,
            vertices: data.vertices,
            metadata: data.metadata,
        })
    }
}
