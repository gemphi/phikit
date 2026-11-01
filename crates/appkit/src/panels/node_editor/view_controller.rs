//! # NSNodeEditorViewController — Node-Based Shader Editor Panel
//!
//! Node editor view controller for visual shader graph editing.
//!
//! ## Apple Documentation
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//! - [NSScrollView](https://developer.apple.com/documentation/appkit/nsscrollview)
//!
//! ## See Also
//! - Custom component for node-based editing
//! - [NSShaderGraphView](crate::shader_graph_view::NSShaderGraphView)
use crate::prelude::*;
use crate::shader_graph_view::NSShaderGraphView;
use crate::theme::{keys};

pub struct NSNodeEditorViewController {
    pub view: NSView,
    pub graph_view: NSShaderGraphView,
}

impl NSNodeEditorViewController {
    pub fn new(rect: NSRect) -> Self {
        let mut view = NSView::new(rect).with_name("NodeEditorPanel");
        view.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        view.set_background_color(Some(crate::theme::color(keys::CLR_BG_CANVAS)));
        let graph_view = NSShaderGraphView::new(rect);
        view.add_subview(graph_view.view().clone());
        Self { view, graph_view }
    }
}

pub fn get_last_compiled_material() -> Option<String> {
    None
}
