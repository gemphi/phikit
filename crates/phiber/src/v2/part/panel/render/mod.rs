//! Render context — Cycles + EEVEE common sections.
//! Blender ref: `bl_ui/properties_render.py`, `rna_scene.cc` RenderSettings,
//! `intern/cycles/blender/*.cpp` for Cycles-specific properties.

pub mod set;
pub use set::SECTIONS;
