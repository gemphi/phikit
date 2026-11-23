//! # ViewNode Debug Implementation
//!
//! Manual Debug implementation for ViewNode because `Action` (Box<dyn Fn()>)
//! does not implement Debug.

use super::node_types::ViewNode;

impl std::fmt::Debug for ViewNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewNode::VStack {
                id,
                children,
                spacing,
                padding,
                size,
            } => f
                .debug_struct("VStack")
                .field("id", id)
                .field("children", children)
                .field("spacing", spacing)
                .field("padding", padding)
                .field("size", size)
                .finish(),
            ViewNode::HStack {
                id,
                children,
                spacing,
                padding,
                size,
            } => f
                .debug_struct("HStack")
                .field("id", id)
                .field("children", children)
                .field("spacing", spacing)
                .field("padding", padding)
                .field("size", size)
                .finish(),
            ViewNode::ZStack {
                id,
                children,
                padding,
                size,
            } => f
                .debug_struct("ZStack")
                .field("id", id)
                .field("children", children)
                .field("padding", padding)
                .field("size", size)
                .finish(),
            ViewNode::Text {
                id,
                content,
                style,
                size,
            } => f
                .debug_struct("Text")
                .field("id", id)
                .field("content", content)
                .field("style", style)
                .field("size", size)
                .finish(),
            ViewNode::Button {
                id,
                title,
                icon,
                style,
                size,
                ..
            } => f
                .debug_struct("Button")
                .field("id", id)
                .field("title", title)
                .field("icon", icon)
                .field("style", style)
                .field("size", size)
                .finish(),
            ViewNode::Image {
                id,
                name,
                style,
                size,
            } => f
                .debug_struct("Image")
                .field("id", id)
                .field("name", name)
                .field("style", style)
                .field("size", size)
                .finish(),
            ViewNode::Spacer { id, min_length } => f
                .debug_struct("Spacer")
                .field("id", id)
                .field("min_length", min_length)
                .finish(),
            ViewNode::Panel {
                id,
                child,
                style,
                padding,
                size,
            } => f
                .debug_struct("Panel")
                .field("id", id)
                .field("child", child)
                .field("style", style)
                .field("padding", padding)
                .field("size", size)
                .finish(),
            ViewNode::Custom { id, view, size } => f
                .debug_struct("Custom")
                .field("id", id)
                .field("view", view)
                .field("size", size)
                .finish(),
            ViewNode::ForEach {
                id,
                children,
                spacing,
            } => f
                .debug_struct("ForEach")
                .field("id", id)
                .field("children", children)
                .field("spacing", spacing)
                .finish(),
        }
    }
}
