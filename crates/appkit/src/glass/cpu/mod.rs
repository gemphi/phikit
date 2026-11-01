//! The stages on the CPU, over the pixels of a [`GlassCanvas`].

mod backdrop;
mod blur;
mod canvas;
mod pipeline;
mod shade;

pub use canvas::{BufferCanvas, GlassCanvas};
pub use pipeline::{GlassLayer, GlassPipeline, GlassStats};
