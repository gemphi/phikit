//! Panel implementations - core logic for editor panels.
//!
//! Each panel has a model (data) and view_controller (UI).
//! rust-kit-pro provides thin wrappers that delegate here.
//!
//! Pattern: every panel in the editor has a matching controller here.
//! The controller owns the view tree and is driven by a model struct.
//! rust-kit-pro panels are thin composition wrappers that feed app data
//! into these controllers.

pub mod activity_bar;
pub mod asset_library;
pub mod defaults;
pub mod floating_panels;
pub mod hierarchy;
pub mod inspector;
pub mod material_inspector;
pub mod node_editor;
pub mod project_browser;
pub mod statistics;
pub mod timeline;
pub mod tools_sidebar;
pub mod toolbar;
/// Backward-compatible alias for code using `panels::top_bar::*`.
pub use toolbar as top_bar;
pub mod traits;
pub mod viewport;
/// Backward-compatible alias — existing code uses `panels::workspace_layout::*`.
pub use crate::workspace as workspace_layout;

pub use activity_bar::*;
pub use asset_library::*;
pub use defaults::*;
pub use hierarchy::*;
pub use inspector::*;
pub use material_inspector::*;
pub use node_editor::*;
pub use project_browser::*;
pub use statistics::*;
pub use timeline::*;
pub use tools_sidebar::*;
pub use toolbar::*;
pub use traits::*;
#[allow(ambiguous_glob_reexports)]
pub use viewport::*;
