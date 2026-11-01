//! # Glass
//!
//! A view's surface made from what is already drawn beneath it. The names
//! follow `NSGlassEffectView` and `NSGlassEffectContainerView`: a style, a
//! corner radius, a tint, and a container that merges glass within `spacing`.
//!
//! ```text
//!  NSView (is_glass, style, tint)        container: one layer for many views
//!         │
//!         ▼ view.rs: shapes and a material, once per glass layer
//!  GlassPipeline::render
//!         │
//!         ├─ cpu/backdrop.rs capture the region below, downsampled
//!         ├─ cpu/blur.rs  three box passes, close to a Gaussian
//!         └─ cpu/shade.rs shadow, lens, saturation, tint, edge light
//!         │
//!         ▼
//!  GlassCanvas (cpu/canvas.rs): the pixels drawn so far
//! ```
//!
//! Views draw back to front into the UI buffer, the 3D viewport included, so
//! the buffer holds everything below a view at the moment it draws. Nothing
//! has to be split or read back, and it behaves the same whichever way the
//! buffer reaches the screen. Subviews draw afterwards and stay sharp.
//!
//! The pipeline knows only [`GlassCanvas`], [`GlassShape`] and
//! [`GlassMaterial`]; it has no idea what a view or a theme is.
//!
//! `gpu/directx.rs` and `gpu/glass.hlsl` are the same stages on the GPU, which gpui's
//! Windows renderer compiles in and every gpui app reaches through
//! `gpui::BackdropStyle`. They are not modules of this crate.

mod cpu;
mod material;
mod shape;
mod view;

pub use cpu::{BufferCanvas, GlassCanvas, GlassLayer, GlassPipeline, GlassStats};
pub use material::{GlassMaterial, GlassStyle};
pub use shape::{GlassBounds, GlassShape};
