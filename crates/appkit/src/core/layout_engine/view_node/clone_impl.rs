//! # ViewNode Clone Implementation
//!
//! Manual Clone implementation for ViewNode because `Action` (Box<dyn Fn()>)
//! is not Clone. We drop the action on clone — callers should re-attach if needed.

use super::node_types::ViewNode;

impl Clone for ViewNode {
    fn clone(&self) -> Self {
        match self {
            ViewNode::VStack {
                id,
                children,
                spacing,
                padding,
                size,
            } => ViewNode::VStack {
                id: *id,
                children: children.clone(),
                spacing: *spacing,
                padding: *padding,
                size: *size,
            },
            ViewNode::HStack {
                id,
                children,
                spacing,
                padding,
                size,
            } => ViewNode::HStack {
                id: *id,
                children: children.clone(),
                spacing: *spacing,
                padding: *padding,
                size: *size,
            },
            ViewNode::ZStack {
                id,
                children,
                padding,
                size,
            } => ViewNode::ZStack {
                id: *id,
                children: children.clone(),
                padding: *padding,
                size: *size,
            },
            ViewNode::Text {
                id,
                content,
                style,
                size,
            } => ViewNode::Text {
                id: *id,
                content: content.clone(),
                style: style.clone(),
                size: *size,
            },
            ViewNode::Button {
                id,
                title,
                icon,
                style,
                size,
                ..
            } => ViewNode::Button {
                id: *id,
                title: title.clone(),
                icon: icon.clone(),
                style: style.clone(),
                size: *size,
                action: None,
            },
            ViewNode::Image {
                id,
                name,
                style,
                size,
            } => ViewNode::Image {
                id: *id,
                name: name.clone(),
                style: style.clone(),
                size: *size,
            },
            ViewNode::Spacer { id, min_length } => ViewNode::Spacer {
                id: *id,
                min_length: *min_length,
            },
            ViewNode::Panel {
                id,
                child,
                style,
                padding,
                size,
            } => ViewNode::Panel {
                id: *id,
                child: child.clone(),
                style: style.clone(),
                padding: *padding,
                size: *size,
            },
            ViewNode::Custom { id, view, size } => ViewNode::Custom {
                id: *id,
                view: view.clone(),
                size: *size,
            },
            ViewNode::ForEach {
                id,
                children,
                spacing,
            } => ViewNode::ForEach {
                id: *id,
                children: children.clone(),
                spacing: *spacing,
            },
        }
    }
}
