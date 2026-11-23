//! # NSAssetLibraryViewController — Asset Library Panel
//!
//! Implements the assembly and management logic for the content library grid.
//! Mirrors Apple's view controller pattern using NSCollectionView.
//!
//! ## Apple Documentation
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [NSCollectionView](https://developer.apple.com/documentation/appkit/nscollectionview)
//! - [NSSearchField](https://developer.apple.com/documentation/appkit/nssearchfield)
//!
//! ## See Also
//! - [NSCollectionViewDataSource](https://developer.apple.com/documentation/appkit/nscollectionviewdatasource)
//! - [NSCollectionViewDelegate](https://developer.apple.com/documentation/appkit/nscollectionviewdelegate)

use super::model::*;
use crate::collection_view::{NSCollectionView, NSCollectionViewDataSource};
use crate::graphics::image::NSImage;
use crate::prelude::*;
use crate::theme::{self, keys};
use std::sync::{Arc, Mutex};

pub struct NSAssetLibraryViewController {
    pub view: NSView,
    pub search_field: NSSearchField,
    pub collection_view: NSCollectionView,
    pub state: NSAssetLibraryState,
}

impl NSAssetLibraryViewController {
    pub fn new(
        rect: NSRect,
        _icon_resolver: Arc<dyn Fn(&str, f32, f32) -> Option<Arc<NSImage>> + Send + Sync>,
    ) -> Self {
        let mut view = NSView::new(rect).with_name("AssetLibraryPanel");
        view.set_background_color(Some(theme::color(keys::CLR_BG_PANEL)));

        // Search Field
        let mut search_field = NSSearchField::new(NSRect::new(
            NSLayoutConstants::MARGIN,
            NSLayoutConstants::MARGIN,
            rect.width() - NSLayoutConstants::MARGIN * 2.0,
            NSLayoutConstants::SEARCH_FIELD_HEIGHT,
        ));
        search_field.set_placeholder_string("Search Assets...");
        search_field.view_mut().set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MAX_Y_MARGIN,
        );
        view.add_subview(search_field.view().clone());

        // Tab Bar (Categories) - Applications should add categories using add_category()
        let tabs_y = NSLayoutConstants::MARGIN * 2.0 + NSLayoutConstants::SEARCH_FIELD_HEIGHT;
        let mut stack = NSStackView::horizontal();
        stack.view_mut().set_frame(NSRect::new(
            NSLayoutConstants::MARGIN,
            tabs_y,
            rect.width() - NSLayoutConstants::MARGIN * 2.0,
            NSLayoutConstants::TAB_BAR_HEIGHT,
        ));
        stack.set_spacing(NSLayoutConstants::MARGIN_XSMALL);
        stack.view_mut().set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MAX_Y_MARGIN,
        );
        view.add_subview(stack.view().clone());

        // Collection View (The Apple Way)
        let grid_y = tabs_y + NSLayoutConstants::TAB_BAR_HEIGHT + NSLayoutConstants::MARGIN;
        let grid_rect = NSRect::new(
            NSLayoutConstants::MARGIN,
            grid_y,
            rect.width() - NSLayoutConstants::MARGIN * 2.0,
            (rect.height() - grid_y).max(0.0),
        );
        let mut collection_view = NSCollectionView::new(grid_rect);
        collection_view.view_mut().set_name("AssetGrid");
        collection_view.view_mut().set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        view.add_subview(collection_view.view().clone());

        let mut vc = Self {
            view,
            search_field,
            collection_view,
            state: NSAssetLibraryState::new(),
        };
        vc.reload_data();
        vc
    }

    pub fn set_category(&mut self, category: NSAssetCategory) {
        self.state.active_category = category;
        self.reload_data();
    }

    pub fn import_asset(&mut self, _path: &str) {
        // Implementation for importing assets
        self.reload_data();
    }

    /// Add a category button to the asset library.
    /// Applications should use this to configure categories with their specific names.
    ///
    /// # Example
    /// ```rust,ignore
    /// asset_library.add_category("Shapes");
    /// asset_library.add_category("Materials");
    /// asset_library.add_category("Audio");
    /// ```
    pub fn add_category(&mut self, name: &str) {
        // Find the category stack view and add a button
        // Note: This is a simplified implementation - a real implementation would
        // need to store a reference to the stack view
        let btn = NSButton::new(NSRect::zero()).with_title(name);
        // TODO: Add button to the category stack
    }

    /// Set the assets to display in the collection view.
    /// Applications should provide their own asset data.
    ///
    /// # Example
    /// ```rust,ignore
    /// let assets = vec![
    ///     NSAssetItem { name: "Cube", icon_id: "Cube", category: NSAssetCategory::Primitives },
    ///     NSAssetItem { name: "Sphere", icon_id: "Sphere", category: NSAssetCategory::Primitives },
    /// ];
    /// asset_library.set_assets(assets);
    /// ```
    pub fn set_assets(&mut self, assets: Vec<NSAssetItem>) {
        let ds = AssetDataSource { assets };
        self.collection_view.set_data_source(ds);
    }

    pub fn reload_data(&mut self) {
        // Applications should call set_assets() to provide their own asset data
        // This method is kept for backward compatibility but does nothing by default
    }
}

struct AssetDataSource {
    assets: Vec<NSAssetItem>,
}

impl NSCollectionViewDataSource for AssetDataSource {
    fn number_of_items(&self) -> usize {
        self.assets.len()
    }

    fn view_for_item(&self, index: usize) -> NSView {
        let asset = &self.assets[index];
        let mut view = NSView::new(NSRect::sized(64.0, 64.0));
        view.set_name(&format!("AssetItem_{}", asset.name));
        let mut label = NSTextField::label(asset.name);
        label
            .control_mut()
            .view_mut()
            .set_frame(NSRect::new(4.0, 40.0, 56.0, 20.0));
        view.add_subview(label.view_clone());
        view
    }
}
