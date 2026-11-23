//! # Glass
//!
//! A surface made from what is already drawn beneath it. The names follow
//! `NSGlassEffectView` and `NSGlassEffectContainerView`: a style, a corner
//! radius, a tint, and a container that merges glass within `spacing`.
//!
//! This crate is the one home of glass. It depends on nothing, so every
//! renderer can take it and it builds for the web as it is.
//!
//! ```text
//!  a host: a style, a tint, the shapes        (gpui::BackdropStyle, an NSView, a page)
//!         │
//!         ├─ on the CPU ──▶ GlassPipeline::render over a GlassCanvas
//!         │                   cpu/backdrop  capture the region below, downsampled
//!         │                   cpu/blur      three box passes, close to a Gaussian
//!         │                   cpu/shade     shadow, lens, saturation, tint, edge light
//!         │
//!         └─ on the GPU ──▶ GlassPlan::of: one copy, four draws, the same on every backend
//!                             glass.hlsl   DirectX  (compiled into gpui_windows)
//!                             glass.wgsl   wgpu and the web
//! ```
//!
//! The CPU stages are the reference: what a GPU backend draws is checked
//! against them. Both know only a canvas or a texture, a [`GlassShape`] and a
//! [`GlassMaterial`]; neither knows what a view or a theme is.
//!
//! The DirectX pass is `gpui_windows`'s own (`directx_backdrop.rs`): it
//! makes a [`GlassRequest`] and draws the four stages of its [`GlassPlan`].

mod cpu;
mod effect;
mod material;
pub mod plan;
mod shape;

pub use cpu::{BufferCanvas, GlassCanvas, GlassLayer, GlassPipeline, GlassStat};
pub use effect::{GlassContainer, GlassEffect};
pub use material::{GlassMaterial, GlassStyle};
pub use plan::{GlassParam, GlassPlan, GlassRequest};
pub use shape::{GlassBounds, GlassShape};

/// The stages as shader source, for a renderer to compile.
pub mod shader {
    /// For DirectX: vertex `backdrop_vertex`, fragment `backdrop_fragment`.
    pub const HLSL: &str = include_str!("glass.hlsl");
    /// For wgpu and the web: vertex `glass_vertex`, fragment `glass_fragment`.
    pub const WGSL: &str = include_str!("glass.wgsl");
}

#[cfg(test)]
mod tests;
