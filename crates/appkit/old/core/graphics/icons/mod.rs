//! SVG Icons — Geometric icon rendering system
//!
//! Each icon category lives in its own file. The public API is:
//! - `IconId` enum — identifies which icon to render
//! - `get(id)` → Option<Arc<NSImage>> at default size (20px)
//! - `get_sized(id, w, h)` → Option<Arc<NSImage>> at custom size
//! - `get_icon_pixels(id, size)` → (Vec<u8>, u32, u32) raw RGBA pixels
//!
//! Icons are rendered as geometric primitives (filled shapes, arcs, lines)
//! matching Apple SF Symbols aesthetic.

mod layout;
mod media;
mod navigation;
mod primitives;
mod render;
mod shapes;
mod simple;
mod toolbar;
mod types;

pub use render::{get, get_icon_pixels, get_sized, render_icon_to_buffer};
pub use types::{IconId, Iconable};
