//! # NSImage — Image Handling
//!
//! Support for PNG and SVG icons with HiDPI-aware rasterization.
//! SVGs are rendered at `logical_size × scale_factor` to produce
//! pixel-perfect icons on high-DPI displays.

mod format_types;
mod loading;
mod processing;

pub use format_types::{NSImage, NSImageSymbol};
pub use loading::{display_scale_factor, set_display_scale_factor};
