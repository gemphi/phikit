//! What the projector produces and the rasteriser consumes.
//!
//! A triangle carries everything shading needs — its corners with depth, a
//! normal and a UV at each of them, the surface colour and how that surface
//! responds to light — rather than a finished colour. The shading happens per
//! pixel (`mesh::raster`), so the decisions cannot be made per triangle here.

use gpui::{Bounds, Hsla, Pixels, Window};

#[derive(Clone, Copy)]
pub(in super::super) struct BMeshPalette {
    /// The active object's outline: the accent a step lighter.
    pub accent_active: gpui::Hsla,
    /// Active `View3DShading.type` — picks viewport vs. base colour.
    pub shading: crate::v2::scene::BShading,
    pub cube: Hsla,
    pub sphere: Hsla,
    pub gem: Hsla,
    pub loaded: Hsla,
    pub accent: Hsla,
    /// MatCap lighting's image, for the Solid build to upload
    /// (`BSolidShade::matcap_for_build`); none under Studio or Flat.
    pub matcap: Option<&'static str>,
    /// Color ▸ Texture: Solid uploads the materials' base-colour images
    /// (`BSolidShade::texture_for_build`); Solid otherwise uploads none.
    pub texture: bool,
}

/// How a surface responds to light: `Material.roughness` and
/// `Material.metallic`.
#[derive(Clone, Copy)]
pub(in super::super) struct BSurface {
    pub roughness: f32,
    pub metallic: f32,
}

impl BSurface {
    /// Blender's Principled defaults — a plain dielectric.
    pub const DEFAULT: Self = Self { roughness: 0.5, metallic: 0.0 };
}

/// A texture and where this triangle sits in it.
#[derive(Clone)]
pub(in super::super) struct BTexture {
    pub uvs: [[f32; 2]; 3],
    pub image: crate::phiphx::scene::SceneTexture,
}

#[derive(Clone)]
pub(in super::super) struct BTriangle {
    /// Each corner as `(x, y, depth)` in canvas space. The depth is kept per
    /// corner, not averaged: a z-buffer interpolates it across the triangle,
    /// which is how two surfaces that cross each other resolve correctly.
    pub points: [(f32, f32, f32); 3],
    /// One normal per corner, in projector space. The rasteriser interpolates
    /// them across the triangle and lights each pixel from its own normal, so
    /// a smooth surface reads smooth. Taking one corner's normal for the
    /// whole triangle is what made every curved surface look faceted.
    pub normals: [(f32, f32, f32); 3],
    /// The surface colour before lighting.
    pub base: Hsla,
    /// `Material.roughness` and `Material.metallic`: they split the light
    /// into a diffuse and a specular half (`workbench_world_light_lib.glsl`),
    /// so a metal takes its highlight's colour from the surface and a rough
    /// surface spreads that highlight out.
    pub roughness: f32,
    pub metallic: f32,
    /// The UV at each corner, and the texture they address. The rasteriser
    /// interpolates them and samples per pixel: sampling once at the UV
    /// centroid made every textured face a flat patch of one colour, which is
    /// what made textures look wrong rather than missing.
    pub texture: Option<BTexture>,
}

#[derive(Clone)]
pub(in super::super) struct BMesh {
    pub triangles: Vec<BTriangle>,
    pub outline: Option<Vec<(f32, f32)>>,
    pub outline_color: Hsla,
}

impl BMesh {
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        bounds: Bounds<Pixels>,
        meshes: &[Self],
        outline_width: f32,
        wire: bool,
        yaw: f32,
        pitch: f32,
        corners: gpui::Corners<gpui::Pixels>,
        fill: bool,
        supersample: usize,
        window: &mut Window,
    ) {
        super::paint::meshes(bounds, meshes, outline_width, wire, yaw, pitch, corners, fill, supersample, window);
    }
}
