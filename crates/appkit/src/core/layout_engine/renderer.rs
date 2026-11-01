//! # Layout Renderer
//!
//! Bridges the declarative layout engine to the NSView rendering system.
//!
//! KEY INSIGHT: The flexbox engine produces ABSOLUTE frames (relative to window).
//! But NSView uses RELATIVE frames (each view's origin is relative to its parent).
//! The renderer converts absolute → relative by subtracting the parent's origin
//! from each child's frame.

use crate::graphics::{NSColor, NSRect};
use crate::theme;
use crate::view::NSView;
use crate::{NSBezelStyle as AppKitBezelStyle, NSButton, NSTextField};

use super::constraints::LayoutResult;
use super::style::{BezelStyle, TextAlignment};
use super::view_node::ViewNode;
use super::FlexboxEngine;

/// Render a ViewNode tree into a concrete NSView tree.
///
/// Runs the flexbox layout pass, then materializes each node into
/// an NSView with parent-relative frames.
///
/// The returned view's frame = `available` (absolute position in window).
/// All children inside have frames relative to their parent.
pub fn render(root: &ViewNode, available: NSRect) -> NSView {
    let engine = FlexboxEngine::new();
    let result = engine.layout(root, available);
    // The root's parent is the window at (0,0), so we pass (0,0,w,h) as parent_frame.
    // This means the root view keeps its absolute position from the layout engine.
    let window_frame = NSRect::new(
        0.0,
        0.0,
        available.x() + available.width(),
        available.y() + available.height(),
    );
    materialize(root, &result, window_frame)
}

/// Materialize a ViewNode into an NSView.
/// `parent_frame` is the absolute frame of the parent — used to convert
/// child absolute frames into parent-relative coordinates.
fn materialize(node: &ViewNode, result: &LayoutResult, parent_frame: NSRect) -> NSView {
    let abs_frame = result.frame_for(node.id()).unwrap_or(parent_frame);

    // Convert absolute frame to parent-relative frame.
    // The root node keeps its absolute position (it's placed in the window).
    // Children subtract their parent's origin.
    let rel_frame = NSRect::new(
        abs_frame.x() - parent_frame.x(),
        abs_frame.y() - parent_frame.y(),
        abs_frame.width(),
        abs_frame.height(),
    );

    match node {
        ViewNode::VStack { children, .. } => {
            let mut view = NSView::new(rel_frame).with_name("VStack");
            for child in children {
                let child_view = materialize(child, result, abs_frame);
                view.add_subview(child_view);
            }
            view
        }

        ViewNode::HStack { children, .. } => {
            let mut view = NSView::new(rel_frame).with_name("HStack");
            for child in children {
                let child_view = materialize(child, result, abs_frame);
                view.add_subview(child_view);
            }
            view
        }

        ViewNode::ZStack { children, .. } => {
            let mut view = NSView::new(rel_frame).with_name("ZStack");
            for child in children {
                let child_view = materialize(child, result, abs_frame);
                view.add_subview(child_view);
            }
            view
        }

        ViewNode::Text { content, style, .. } => {
            let mut label = NSTextField::label(content);
            label.set_font_size(style.font_size);
            label.set_text_color(resolve_color(&style.color));
            label.set_bordered(false);
            match style.alignment {
                TextAlignment::Center => label.set_text_alignment(crate::NSTextAlignment::Center),
                TextAlignment::Right => label.set_text_alignment(crate::NSTextAlignment::Right),
                TextAlignment::Left => {}
            }
            let mut view = label.view_clone();
            view.set_frame(rel_frame);
            view.set_background_color(None);
            view
        }

        ViewNode::Button { title, style, .. } => {
            let mut btn = NSButton::new(rel_frame);
            btn.set_title(title);
            btn.set_bezel_style(map_bezel_style(style.bezel_style));
            if let Some(fill) = &style.fill_color {
                btn.set_accent_color(Some(resolve_color(fill)));
            } else {
                // Default: give buttons a visible background so they're not invisible
                btn.set_accent_color(Some(NSColor::bg_control()));
            }
            if let Some(tint) = &style.tint_color {
                btn.set_accent_color(Some(resolve_color(tint)));
            }
            btn.view_mut().set_corner_radius(style.corner_radius);
            btn.view_clone()
        }

        ViewNode::Image { name, style, .. } => {
            let mut view = NSView::new(rel_frame).with_name(name);
            if let Some(tint) = &style.tint_color {
                view.set_background_color(Some(resolve_color(tint)));
            } else {
                view.set_background_color(Some(NSColor::accent()));
            }
            view.set_corner_radius(style.corner_radius);
            view
        }

        ViewNode::Spacer { .. } => NSView::new(rel_frame).with_name("Spacer"),

        ViewNode::Panel { child, style, .. } => {
            let mut view = NSView::new(rel_frame).with_name("Panel");
            view.set_background_color(Some(resolve_color(&style.background_color)));
            view.set_corner_radius(style.corner_radius);
            if style.border_width > 0.0 {
                view.set_border_width(style.border_width);
                view.set_border_color(Some(resolve_color(&style.border_color)));
            }
            let child_view = materialize(child, result, abs_frame);
            view.add_subview(child_view);
            view
        }

        ViewNode::Custom { view, .. } => {
            let mut v = view.clone();
            v.set_frame(rel_frame);
            v
        }

        ViewNode::ForEach { children, .. } => {
            let mut view = NSView::new(rel_frame).with_name("ForEach");
            for child in children {
                let child_view = materialize(child, result, abs_frame);
                view.add_subview(child_view);
            }
            view
        }
    }
}

/// Resolve a theme token string to an NSColor.
fn resolve_color(token: &str) -> NSColor {
    match token {
        "clr.accent" => NSColor::accent(),
        "clr.text.label" => NSColor::text_label(),
        "clr.text.secondary" => NSColor::text_secondary(),
        "clr.text.tertiary" => NSColor::text_tertiary(),
        "clr.bg.panel" => NSColor::bg_panel(),
        "clr.bg.sidebar" => NSColor::bg_sidebar(),
        "clr.bg.content" => NSColor::bg_content(),
        "clr.bg.control" => NSColor::bg_control(),
        "clr.bg.canvas" => NSColor::bg_canvas(),
        "clr.bg.selected" => NSColor::bg_selected(),
        "clr.border" => NSColor::border(),
        "clr.white" => NSColor::white(),
        "clr.red" => NSColor::new(1.0, 0.27, 0.23, 1.0),
        "clr.green" => NSColor::new(0.2, 0.78, 0.35, 1.0),
        "clr.blue" => NSColor::new(0.2, 0.5, 1.0, 1.0),
        _ => theme::color(token),
    }
}

/// Map layout engine BezelStyle to AppKit NSBezelStyle.
fn map_bezel_style(style: BezelStyle) -> AppKitBezelStyle {
    match style {
        BezelStyle::Rounded => AppKitBezelStyle::Rounded,
        BezelStyle::RegularSquare => AppKitBezelStyle::RegularSquare,
        BezelStyle::TexturedRounded => AppKitBezelStyle::TexturedSquare,
        BezelStyle::SmallSquare => AppKitBezelStyle::RegularSquare,
        BezelStyle::Toolbar => AppKitBezelStyle::Rounded,
        BezelStyle::AccessoryBarAction => AppKitBezelStyle::Rounded,
        BezelStyle::Inline => AppKitBezelStyle::Inline,
        BezelStyle::Recessed => AppKitBezelStyle::Recessed,
        BezelStyle::Push => AppKitBezelStyle::Rounded,
    }
}
