//! A part: a mesh, the material it is drawn in, and where it stands — the
//! unit the generators hand Phiber. A crowd is many parts on one mesh:
//! the mesh is shared (`Arc`), and so may be the material.

use super::image::BImage;
use super::mesh::BMesh;
use crate::colour::Rgb;
use glam::Mat4;
use std::sync::Arc;

/// A material: the Principled values the sets decide, and an optional
/// image for the base colour (and the emission, when it glows).
#[derive(Clone, Debug, PartialEq)]
pub struct BMat {
    pub name: String,
    pub base: Rgb,
    pub roughness: f32,
    pub metallic: f32,
    /// Light it gives off, and how strongly (Blender's Emission Strength).
    pub emission: Rgb,
    pub strength: f32,
    pub image: Option<Arc<BImage>>,
}

impl BMat {
    /// A plain surface of `base`.
    pub fn plain(name: impl Into<String>, base: Rgb, roughness: f32) -> Self {
        BMat { name: name.into(), base, roughness, metallic: 0.0, emission: Rgb(0.0, 0.0, 0.0), strength: 0.0, image: None }
    }

    /// Metal of `base`.
    pub fn metal(name: impl Into<String>, base: Rgb, roughness: f32) -> Self {
        BMat { metallic: 1.0, ..BMat::plain(name, base, roughness) }
    }

    /// Lit from within: `colour` at `strength`.
    pub fn glowing(name: impl Into<String>, colour: Rgb, strength: f32) -> Self {
        BMat { emission: colour, strength, ..BMat::plain(name, colour, 0.5) }
    }

    /// The same, painted with `image`.
    pub fn with_image(mut self, image: Arc<BImage>) -> Self {
        self.image = Some(image);
        self
    }
}

#[derive(Clone, Debug)]
pub struct BPart {
    pub name: String,
    pub mesh: Arc<BMesh>,
    pub material: Arc<BMat>,
    pub at: Mat4,
}

impl BPart {
    pub fn new(name: impl Into<String>, mesh: BMesh, material: BMat, at: Mat4) -> Self {
        BPart { name: name.into(), mesh: Arc::new(mesh), material: Arc::new(material), at }
    }
}
