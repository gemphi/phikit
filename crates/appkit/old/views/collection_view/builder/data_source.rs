//! Generic Collection View Data Source and Delegate
//!
//! this file implements standard data source and delegate logic for grid-based
//! asset displays. it handles item creation, icon mapping based on file extensions,
//! and selection events. mirrors Apple's modular approach to collection management.

use crate::collection_view::{NSCollectionViewDataSource, NSCollectionViewDelegate};
use crate::graphics::{NSImage, NSSize};
use crate::svg_icons::{self, IconId};
use crate::view::NSViewDelegate;
use crate::{NSColor, NSImagePosition, NSRect, NSTextField, NSView};
use std::sync::Arc;

/// Standard data source for asset collections.
///
/// tracks a list of asset names and handles their visual representation.
/// coordinates with the application to dispatch actions on item selection.
#[derive(Clone)]
pub struct GenericAssetDataSource {
    pub assets: Vec<String>,
    pub action: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl NSCollectionViewDataSource for GenericAssetDataSource {
    /// Returns the total number of assets in the collection.
    fn number_of_items(&self) -> usize {
        self.assets.len()
    }

    /// Creates and configures a view for a specific item index.
    /// maps file extensions to appropriate icons and centers labels.
    fn view_for_item(&self, index: usize) -> NSView {
        let name = self.assets.get(index).cloned().unwrap_or_default();
        let mut item = NSView::new(NSRect::sized(100.0, 100.0));
        item.set_name(&name);

        let ext = name.split('.').last().unwrap_or("").to_lowercase();
        let icon_id = match ext.as_str() {
            "usdz" | "obj" | "fbx" | "gltf" | "phx" => IconId::Mesh,
            "material" | "mat" => IconId::Material,
            "shader" | "sh" | "riograph" => IconId::Shader,
            "wav" | "mp3" | "ogg" | "audio" | "m4a" => IconId::Audio,
            "png" | "jpg" | "jpeg" | "tga" | "hdr" => IconId::Image,
            "" => IconId::Folder,
            _ => IconId::File,
        };

        if let Some(icon) = svg_icons::get_sized(icon_id, 32.0, 32.0) {
            let mut icon_view = NSView::new(NSRect::new(34.0, 20.0, 32.0, 32.0));
            icon_view.set_delegate(GenericGridItemDelegate { image: icon });
            item.add_subview(icon_view);
        }

        let mut label = NSTextField::label(&name);
        label
            .control_mut()
            .view_mut()
            .set_frame(NSRect::new(5.0, 65.0, 90.0, 20.0));
        label.set_font_size(10.0);
        label.set_text_alignment(crate::NSTextAlignment::Center);
        item.add_subview(label.view_clone());

        item
    }

    /// Dispatches the selection action when an item is clicked.
    fn collection_view_did_select_item(&mut self, index: usize) {
        if let Some(name) = self.assets.get(index) {
            if let Some(action) = &self.action {
                action(name.clone());
            }
        }
    }
}

impl NSCollectionViewDelegate for GenericAssetDataSource {
    fn collection_view_selection_did_change(&mut self, _selected_index: Option<usize>) {}

    /// Dispatches the same selection action on double click.
    fn collection_view_item_did_double_click(&mut self, index: usize) {
        if let Some(name) = self.assets.get(index) {
            if let Some(action) = &self.action {
                action(name.clone());
            }
        }
    }
}

/// Simple delegate for drawing an image within a grid item.
pub struct GenericGridItemDelegate {
    pub image: Arc<NSImage>,
}

impl NSViewDelegate for GenericGridItemDelegate {
    /// Renders the associated image stretched to the view's bounds.
    fn draw(&self, view: &NSView, ctx: &mut crate::render::RenderContext) {
        ctx.draw_image(
            &self.image,
            NSRect::sized(view.bounds().width(), view.bounds().height()),
        );
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
