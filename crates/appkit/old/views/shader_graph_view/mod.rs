//! # NSShaderGraphView — Node-Based Material Editor
//!
//! A node-based visual editor for shader and material authoring.
//!
//! **Custom component (no direct Apple equivalent)** — This component provides
//! node-based shader graph editing functionality similar to Blender's Shader Editor,
//! Unreal Engine's Material Editor, or Unity's Shader Graph.
//!
//! ## Apple Documentation
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//! - [NSScrollView](https://developer.apple.com/documentation/appkit/nsscrollview)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut graph = NSShaderGraphView::new(frame);
//! let node1 = graph.add_node("Texture", position);
//! let node2 = graph.add_node("Color", position);
//! graph.connect(node1, "output", node2, "input");
//! ```
//!
//! ## See Also
//! - Custom component for shader graph editing
//! - Similar to Blender Shader Editor / Unreal Material Editor
pub mod compiler;
pub mod delegate;
pub mod model;
pub use model::{
    NSShaderConnection, NSShaderGraphState, NSShaderGraphView, NSShaderNode, NSShaderPort,
    PortDirection,
};
