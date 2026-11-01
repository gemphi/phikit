//! # ViewNode Builder Helpers
//!
//! Convenience constructors and modifier methods for building ViewNode trees.

use super::node_types::{next_view_id, Action, ViewNode};
use super::super::constraints::{Edges, Size};
use super::super::style::{ButtonStyle, ImageStyle, PanelStyle, TextStyle};
use crate::view::NSView;

impl ViewNode {
    /// Create a VStack with default padding and flexible size.
    pub fn vstack(spacing: f32, children: Vec<ViewNode>) -> Self {
        ViewNode::VStack {
            id: next_view_id(),
            children,
            spacing,
            padding: Edges::zero(),
            size: Size::default(),
        }
    }

    /// Create an HStack with default padding and flexible size.
    pub fn hstack(spacing: f32, children: Vec<ViewNode>) -> Self {
        ViewNode::HStack {
            id: next_view_id(),
            children,
            spacing,
            padding: Edges::zero(),
            size: Size::default(),
        }
    }

    /// Create a ZStack with default padding and flexible size.
    pub fn zstack(children: Vec<ViewNode>) -> Self {
        ViewNode::ZStack {
            id: next_view_id(),
            children,
            padding: Edges::zero(),
            size: Size::default(),
        }
    }

    /// Create a Text node.
    pub fn text(content: &str, style: TextStyle) -> Self {
        ViewNode::Text {
            id: next_view_id(),
            content: content.to_string(),
            style,
            size: Size::default(),
        }
    }

    /// Create a Button node.
    pub fn button(title: &str, style: ButtonStyle, action: Option<Action>) -> Self {
        ViewNode::Button {
            id: next_view_id(),
            title: title.to_string(),
            icon: None,
            style,
            size: Size::default(),
            action,
        }
    }

    /// Create an icon button node.
    pub fn icon_button(icon: &str, style: ButtonStyle, action: Option<Action>) -> Self {
        ViewNode::Button {
            id: next_view_id(),
            title: String::new(),
            icon: Some(icon.to_string()),
            style,
            size: Size::default(),
            action,
        }
    }

    /// Create an Image node with a fixed size.
    pub fn image(name: &str, width: f32, height: f32) -> Self {
        ViewNode::Image {
            id: next_view_id(),
            name: name.to_string(),
            style: ImageStyle::default(),
            size: Size::fixed(width, height),
        }
    }

    /// Create an Image node with a custom style.
    pub fn image_styled(name: &str, size: Size, style: ImageStyle) -> Self {
        ViewNode::Image {
            id: next_view_id(),
            name: name.to_string(),
            style,
            size,
        }
    }

    /// Create a Spacer with a minimum length.
    pub fn spacer(min_length: f32) -> Self {
        ViewNode::Spacer {
            id: next_view_id(),
            min_length,
        }
    }

    /// Create a Panel wrapping a child node.
    pub fn panel(child: ViewNode, style: PanelStyle) -> Self {
        ViewNode::Panel {
            id: next_view_id(),
            child: Box::new(child),
            style,
            padding: Edges::uniform(8.0),
            size: Size::default(),
        }
    }

    /// Create a Custom node wrapping an existing NSView.
    pub fn custom(view: NSView, size: Size) -> Self {
        ViewNode::Custom {
            id: next_view_id(),
            view,
            size,
        }
    }

    /// Create a ForEach node from a list of children.
    pub fn for_each(spacing: f32, children: Vec<ViewNode>) -> Self {
        ViewNode::ForEach {
            id: next_view_id(),
            children,
            spacing,
        }
    }

    /// Set padding on this node (only applies to VStack, HStack, ZStack, Panel).
    pub fn with_padding(mut self, padding: Edges) -> Self {
        match &mut self {
            ViewNode::VStack { padding: p, .. } => *p = padding,
            ViewNode::HStack { padding: p, .. } => *p = padding,
            ViewNode::ZStack { padding: p, .. } => *p = padding,
            ViewNode::Panel { padding: p, .. } => *p = padding,
            _ => {}
        }
        self
    }

    /// Set size on this node.
    pub fn with_size(mut self, new_size: Size) -> Self {
        match &mut self {
            ViewNode::VStack { size, .. } => *size = new_size,
            ViewNode::HStack { size, .. } => *size = new_size,
            ViewNode::ZStack { size, .. } => *size = new_size,
            ViewNode::Text { size, .. } => *size = new_size,
            ViewNode::Button { size, .. } => *size = new_size,
            ViewNode::Image { size, .. } => *size = new_size,
            ViewNode::Panel { size, .. } => *size = new_size,
            ViewNode::Custom { size, .. } => *size = new_size,
            _ => {}
        }
        self
    }
}
