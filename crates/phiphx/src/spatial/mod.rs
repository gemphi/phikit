//! PhiQui — Native Rust GPUI 3D Spatial UI & Mesh Engine

pub mod glyph;
pub mod implicit;
pub mod text;
pub mod mesh_converter;
pub mod spatial_animator;
#[cfg(feature = "bui")]
pub mod spatial_bui;
#[cfg(feature = "bui")]
pub mod spatial_card;
#[cfg(feature = "bui")]
pub mod spatial_element;
pub mod spatial_object;
#[cfg(feature = "bui")]
pub mod spatial_scene;

pub use mesh_converter::{MeshSpaceConversion, PhiQuiMeshType};
pub use spatial_animator::{SpatialSpring, SpatialTransform3D};
#[cfg(feature = "bui")]
pub use spatial_bui::{BSpatialButton, BSpatialCard};
#[cfg(feature = "bui")]
pub use spatial_card::SpatialCard;
#[cfg(feature = "bui")]
pub use spatial_element::SpatialMeshElement;
pub use spatial_object::SpatialObject3D;
#[cfg(feature = "bui")]
pub use spatial_scene::SpatialScene;
