//! Blender's View Layer ▸ Override ▸ Material (`ViewLayer.mat_override`):
//! one material standing in for every other in the layer. EEVEE reads it in
//! `eevee_material.cc:446` — `blender_mat = material_override ?
//! material_override : <the slot's>` — before a single node is evaluated, so
//! every surface in the render shades alike.
//!
//! A render with it on measures the **light**. A level difference between two
//! renderers is either the light arriving at a surface or what the surface
//! does with it, and the two cannot be told apart while every face carries
//! its own albedo, roughness, texture and mix weight (`rendering.md` §60).

use super::surface::BSurface;
use phiray::Tri;
use std::cell::Cell;

/// The material standing in for the file's own.
#[derive(Clone, Copy)]
pub struct BMatStand {
    /// Base colour, linear and grey — the albedo the measurement divides by.
    pub albedo: f32,
    pub rough: f32,
    pub metal: f32,
}

impl Default for BMatStand {
    /// A Lambertian surface: fully rough, dielectric, albedo 1. Specular is
    /// not switched off — Blender's Principled keeps its 0.5 IOR level under
    /// an override too, and a measurement that removed it would not be
    /// measuring what Blender renders.
    fn default() -> Self {
        Self { albedo: 1.0, rough: 1.0, metal: 0.0 }
    }
}

thread_local! {
    static STAND: Cell<Option<BMatStand>> = const { Cell::new(None) };
}

impl BMatStand {
    /// This albedo's stand-in, the rest of it [`Default`].
    pub fn grey(albedo: f32) -> Self {
        Self { albedo, ..Self::default() }
    }

    /// Put it in scope for every face built on this thread; `None` restores
    /// the file's own materials.
    pub fn set(stand: Option<Self>) {
        STAND.with(|c| c.set(stand));
    }

    /// What stands now.
    pub fn standing() -> Option<Self> {
        STAND.with(Cell::get)
    }

    /// The face it builds: the colour and the two parameters, and nothing
    /// else — no image, no procedural texture, no second lobe, no emission,
    /// opaque. An override that kept the slot's textures would not be one.
    fn tri(self) -> Tri {
        let a = self.albedo;
        Tri::new([[0.0; 3]; 3], [[0.0; 3]; 3], [a, a, a, 1.0]).with_pbr(self.rough, self.metal)
    }
}

/// A surface the layer's override may stand in for.
pub(super) trait BMatOverride {
    /// The override's face, when one stands — the slot is then not read at
    /// all, which is both what Blender does and the point of asking.
    fn overridden(&self) -> Option<Tri>;
}

impl BMatOverride for BSurface<'_> {
    fn overridden(&self) -> Option<Tri> {
        BMatStand::standing().map(BMatStand::tri)
    }
}
