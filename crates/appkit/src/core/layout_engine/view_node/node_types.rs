//! # ViewNode Types
//!
//! The declarative view tree representation. Each variant describes
//! a UI element or container that the layout engine resolves into
//! absolute frames.

use super::super::constraints::{Edges, Size, ViewId};
use super::super::style::{ButtonStyle, ImageStyle, PanelStyle, TextStyle};
use crate::view::NSView;

use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for generating unique ViewNode IDs.
static NEXT_VIEW_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a unique ViewId for a new node.
pub fn next_view_id() -> ViewId {
    NEXT_VIEW_ID.fetch_add(1, Ordering::Relaxed)
}

/// Action callback type for interactive nodes (buttons, taps).
pub type Action = Box<dyn Fn() + Send + Sync>;

/// A declarative description of a UI element or container.
/// The layout engine walks this tree to produce absolute NSRect frames.
pub enum ViewNode {
    /// Vertical stack — distributes children top-to-bottom.
    VStack {
        id: ViewId,
        children: Vec<ViewNode>,
        spacing: f32,
        padding: Edges,
        size: Size,
    },

    /// Horizontal stack — distributes children left-to-right.
    HStack {
        id: ViewId,
        children: Vec<ViewNode>,
        spacing: f32,
        padding: Edges,
        size: Size,
    },

    /// Overlay stack — layers children on top of each other (back to front).
    ZStack {
        id: ViewId,
        children: Vec<ViewNode>,
        padding: Edges,
        size: Size,
    },

    /// Static text label.
    Text {
        id: ViewId,
        content: String,
        style: TextStyle,
        size: Size,
    },

    /// Clickable button.
    Button {
        id: ViewId,
        title: String,
        icon: Option<String>,
        style: ButtonStyle,
        size: Size,
        #[allow(dead_code)]
        action: Option<Action>,
    },

    /// Image (icon, asset thumbnail, etc.).
    Image {
        id: ViewId,
        name: String,
        style: ImageStyle,
        size: Size,
    },

    /// Flexible spacer — fills remaining space in a stack.
    Spacer { id: ViewId, min_length: f32 },

    /// Bordered/styled container panel wrapping a single child.
    Panel {
        id: ViewId,
        child: Box<ViewNode>,
        style: PanelStyle,
        padding: Edges,
        size: Size,
    },

    /// Escape hatch: embed a pre-built NSView directly.
    Custom {
        id: ViewId,
        view: NSView,
        size: Size,
    },

    /// Repeating content — maps a collection to view nodes.
    ForEach {
        id: ViewId,
        children: Vec<ViewNode>,
        spacing: f32,
    },
}

impl ViewNode {
    /// Returns the unique ID of this node.
    pub fn id(&self) -> ViewId {
        match self {
            ViewNode::VStack { id, .. } => *id,
            ViewNode::HStack { id, .. } => *id,
            ViewNode::ZStack { id, .. } => *id,
            ViewNode::Text { id, .. } => *id,
            ViewNode::Button { id, .. } => *id,
            ViewNode::Image { id, .. } => *id,
            ViewNode::Spacer { id, .. } => *id,
            ViewNode::Panel { id, .. } => *id,
            ViewNode::Custom { id, .. } => *id,
            ViewNode::ForEach { id, .. } => *id,
        }
    }

    /// Returns the size specification of this node.
    pub fn size(&self) -> Size {
        match self {
            ViewNode::VStack { size, .. } => *size,
            ViewNode::HStack { size, .. } => *size,
            ViewNode::ZStack { size, .. } => *size,
            ViewNode::Text { size, .. } => *size,
            ViewNode::Button { size, .. } => *size,
            ViewNode::Image { size, .. } => *size,
            ViewNode::Spacer { .. } => Size::default(),
            ViewNode::Panel { size, .. } => *size,
            ViewNode::Custom { size, .. } => *size,
            ViewNode::ForEach { .. } => Size::default(),
        }
    }
}
