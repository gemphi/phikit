//! Animation system - SceneKit-compatible (< 200 lines)
pub mod manager;
pub mod player;
pub mod transitions;
pub mod types;

pub use manager::*;
pub use player::*;
pub use transitions::{FadeTransition, TransitionContext, ViewTransition};
pub use types::*;
