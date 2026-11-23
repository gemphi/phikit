//! View components — reusable UI building blocks.
//!
//! Views are self-contained UI components that handle their own layout,
//! rendering, and interaction. Similar to AppKit's NSView subclasses.

pub mod activity_bar;
pub mod collection_view;
pub mod image_view;
pub mod outline_view;
pub mod render_view;
pub mod scroll_view;
pub mod sidebar;
pub mod split_view;
pub mod stack_view;
pub mod tab_view;
pub mod table_view;
pub mod shader_graph_view;
pub mod timeline;
pub mod toolbar;
pub mod menu;

// Re-export commonly used types
pub use activity_bar::{NSActivityBar, NSActivityBarItem, ActivityBarGroup};
