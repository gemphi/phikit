//! # Layout Engine
//!
//! A SwiftUI-style declarative layout system for rust-appkit.
//!
//! The layout engine resolves `ViewNode` trees into absolute `NSRect` positions.
//! Application code implements `ViewBuilder` to declare UI structure, and the
//! `FlexboxEngine` computes pixel-perfect frames for rendering.
//!
//! ## Architecture
//!
//! ```text
//! ViewBuilder::body() → ViewNode tree → FlexboxEngine::layout() → LayoutResult (frames)
//! ```
//!
//! ## Coordinate System
//!
//! Top-left origin: y=0 at the top of the available rect, increasing downward.
//!
//! ## Example
//!
//! ```ignore
//! use rust_appkit::layout_engine::*;
//! use rust_appkit::graphics::NSRect;
//!
//! let tree = ViewNode::vstack(12.0, vec![
//!     ViewNode::text("Hello", TextStyle::title()),
//!     ViewNode::spacer(0.0),
//!     ViewNode::button("OK", ButtonStyle::accent_filled(), None),
//! ]).with_padding(Edges::uniform(16.0));
//!
//! let engine = FlexboxEngine::new();
//! let result = engine.layout(&tree, NSRect::sized(400.0, 300.0));
//!
//! for (id, frame) in &result.frames {
//!     println!("Node {id}: ({}, {}, {}×{})", frame.x(), frame.y(), frame.width(), frame.height());
//! }
//! ```

pub mod constraints;
pub mod flexbox;
pub mod renderer;
pub mod state;
pub mod style;
pub mod view_builder;
pub mod view_host;
pub mod view_node;

// Re-export all public types at the module level for ergonomic imports.
pub use constraints::{Edges, LayoutResult, Size, SizeDimension, ViewId};
pub use flexbox::FlexboxEngine;
pub use renderer::render;
pub use state::{State, StateGroup};
pub use style::{BezelStyle, ButtonStyle, ImageStyle, PanelStyle, TextAlignment, TextStyle};
pub use view_builder::ViewBuilder;
pub use view_host::ViewHost;
pub use view_node::{next_view_id, Action, ViewNode};
