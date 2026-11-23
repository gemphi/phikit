//! Rendering primitives — shape fills, borders, gradients, line drawing, and pixel blitting.

mod clipping;
mod gradient_rendering;
mod shape_primitives;
mod text_primitives;

pub use gradient_rendering::{gradient_interpolate, gradient_interpolate_channel};

#[cfg(test)]
mod tests;
