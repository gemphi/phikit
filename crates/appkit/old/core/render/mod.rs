//! # Software Render Context Modular Implementation
//!
//! 2D rendering fallback for UI components.

pub mod draw_state;
pub mod effects;
pub mod images;
pub mod primitives;
pub mod render_context;
pub mod text;

pub use draw_state::{
    debug_border_color_for_name, debug_borders_enabled, set_debug_borders, DEBUG_BORDERS,
};
pub use effects::GradientDirection;
pub use render_context::{RenderContext, RoundedClip};

pub(crate) use draw_state::blend_colors;
