//! # NSProjectBrowserViewController — 3-Pane File Browser Panel
//!
//! Builds and manages the 3-pane project browser view.
//! Mirrors Apple's Reality Composer Pro project browser:
//!   Navigator (folder tree) | File View (grid/list) | File Inspector
//!
//! ## Apple Documentation
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [NSSplitView](https://developer.apple.com/documentation/appkit/nssplitview)
//! - [NSOutlineView](https://developer.apple.com/documentation/appkit/nsoutlineview)
//! - [NSCollectionView](https://developer.apple.com/documentation/appkit/nscollectionview)
//!
//! ## Architecture
//!
//! The view controller handles:
//! - Building the 3-pane split layout
//! - Folder navigation and file selection
//! - Display mode switching (grid/list/tile)
//! - Search filtering
//!
//! rust-kit-pro's ProjectBrowserView is a thin wrapper that feeds project data here.
//!
//! ## See Also
//! - [NSBrowser](https://developer.apple.com/documentation/appkit/nsbrowser)
//! - [NSPathControl](https://developer.apple.com/documentation/appkit/nspathcontrol)

use super::model::*;
use crate::prelude::*;
use crate::theme::{self, keys};

/// Project browser view controller — manages the 3-pane file browser.
pub struct NSProjectBrowserViewController {
    pub view: NSView,
    pub state: NSProjectBrowserState,
    /// Width of the navigator (left) pane.
    pub navigator_width: f32,
    /// Width of the file inspector (right) pane.
    pub inspector_width: f32,
}

impl NSProjectBrowserViewController {
    /// Create a new project browser view controller.
    ///
    /// `sidebar_w` and `inspector_w` should match the main split above
    /// so columns align visually.
    pub fn new(rect: NSRect, sidebar_w: f32, inspector_w: f32) -> Self {
        let state = NSProjectBrowserState::default();
        let view = Self::build_view(rect, &state, sidebar_w, inspector_w);
        Self {
            view,
            state,
            navigator_width: sidebar_w,
            inspector_width: inspector_w,
        }
    }

    /// Update the state and rebuild the view.
    pub fn set_state(&mut self, state: NSProjectBrowserState) {
        self.state = state;
        self.rebuild();
    }

    /// Navigate to a folder by path.
    pub fn navigate_to_folder(&mut self, folder_path: &str) {
        self.state.active_folder = folder_path.to_string();
        self.state.selected_file_index = None;
        self.state.file_inspector = None;
        self.rebuild();
    }

    /// Select a file by index.
    pub fn select_file(&mut self, index: usize) {
        if index < self.state.files.len() {
            self.state.selected_file_index = Some(index);
            let file = &self.state.files[index];
            self.state.file_inspector = Some(NSFileInspectorData {
                name: file.name.clone(),
                file_type: file.file_type.label().to_string(),
                size: format_file_size(file.size_bytes),
                path: file.path.clone(),
                modified: String::new(),
            });
            self.rebuild();
        }
    }

    /// Set the display mode (grid/list/tile).
    pub fn set_display_mode(&mut self, mode: NSBrowserDisplayMode) {
        self.state.display_mode = mode;
        self.rebuild();
    }

    /// Set the search filter.
    pub fn set_search_query(&mut self, query: &str) {
        self.state.search_query = query.to_lowercase();
        self.rebuild();
    }

    /// Rebuild the view from current state.
    pub fn rebuild(&mut self) {
        let frame = self.view.frame();
        self.view = Self::build_view(frame, &self.state, self.navigator_width, self.inspector_width);
    }

    // ─── View Building ───────────────────────────────────────────────

    fn build_view(
        frame: NSRect,
        state: &NSProjectBrowserState,
        nav_w: f32,
        insp_w: f32,
    ) -> NSView {
        let mut view = NSView::new(frame).with_name("ProjectBrowserView");
        view.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );

        let w = frame.width();
        let h = frame.height();
        let center_w = (w - nav_w - insp_w).max(100.0);

        // ── Navigator pane (left) ────────────────────────────────────
        let mut nav_pane = NSView::new(NSRect::sized(nav_w, h))
            .with_name("BrowserNavigator");
        nav_pane.set_background_color(Some(theme::color(keys::CLR_BG_PANEL)));
        nav_pane.set_autoresizing_mask(NSAutoresizingMask::HEIGHT_SIZABLE);

        let mut y = h - 8.0;
        for folder in &state.folders {
            y -= 22.0;
            if y < 0.0 { break; }
            let prefix = if folder.expanded { "▼ " } else { "▶ " };
            let label_text = format!("{}{}", prefix, folder.name);
            let is_active = folder.path == state.active_folder;

            let mut lbl = NSTextField::label(&label_text);
            lbl.set_font_size(11.0);
            lbl.set_text_color(if is_active {
                theme::color(keys::CLR_TEXT_LABEL)
            } else {
                theme::color(keys::CLR_TEXT_SECONDARY)
            });
            lbl.set_bordered(false);
            let mut lv = lbl.view_clone();
            lv.set_frame(NSRect::new(12.0, y, nav_w - 24.0, 20.0));
            lv.set_background_color(None);
            nav_pane.add_subview(lv);
        }
        view.add_subview(nav_pane);

        // ── File view (center) ───────────────────────────────────────
        let file_x = nav_w;
        let mut file_pane = NSView::new(NSRect::new(file_x, 0.0, center_w, h))
            .with_name("BrowserFileView");
        file_pane.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );

        // Toolbar at top
        let toolbar_h = 32.0_f32;
        let mut toolbar = NSView::new(NSRect::new(0.0, h - toolbar_h, center_w, toolbar_h))
            .with_name("BrowserToolbar");
        toolbar.set_background_color(Some(theme::color(keys::CLR_BG_PANEL)));
        toolbar.set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE);
        file_pane.add_subview(toolbar);

        // File grid below toolbar
        let grid_h = h - toolbar_h;
        let grid_item_size = 64.0_f32;
        let grid_gap = 8.0_f32;
        let grid_margin = 12.0_f32;
        let cols = ((center_w - grid_margin * 2.0 + grid_gap) / (grid_item_size + grid_gap))
            .floor() as usize;

        let filtered_files: Vec<&NSProjectFile> = if state.search_query.is_empty() {
            state.files.iter().collect()
        } else {
            state.files.iter()
                .filter(|f| f.name.to_lowercase().contains(&state.search_query))
                .collect()
        };

        for (i, file) in filtered_files.iter().enumerate() {
            if cols == 0 { break; }
            let col = i % cols;
            let row = i / cols;
            let fx = grid_margin + (grid_item_size + grid_gap) * col as f32;
            let fy = grid_h - grid_margin - (grid_item_size + grid_gap + NSLayoutConstants::ASSET_GRID_LABEL_HEIGHT) * (row as f32 + 1.0);
            if fy < 0.0 { break; }

            let is_selected = state.selected_file_index == Some(i);

            let mut item = NSView::new(NSRect::new(fx, fy, grid_item_size, grid_item_size))
                .with_name(&file.name);
            item.set_background_color(Some(if is_selected {
                theme::color(keys::CLR_BG_SELECTED)
            } else {
                theme::color(keys::CLR_BG_PANEL)
            }));
            item.set_corner_radius(6.0);
            file_pane.add_subview(item);

            // File name label below
            let mut name_lbl = NSTextField::label(&file.name);
            name_lbl.set_font_size(9.0);
            name_lbl.set_text_color(theme::color(keys::CLR_TEXT_SECONDARY));
            name_lbl.set_bordered(false);
            let mut nv = name_lbl.view_clone();
            nv.set_frame(NSRect::new(fx, fy - 16.0, grid_item_size, 14.0));
            nv.set_background_color(None);
            file_pane.add_subview(nv);
        }

        view.add_subview(file_pane);

        // ── Inspector pane (right) ───────────────────────────────────
        let insp_x = w - insp_w;
        let mut insp_pane = NSView::new(NSRect::new(insp_x, 0.0, insp_w, h))
            .with_name("BrowserInspector");
        insp_pane.set_background_color(Some(theme::color(keys::CLR_BG_PANEL)));
        insp_pane.set_autoresizing_mask(
            NSAutoresizingMask::MIN_X_MARGIN | NSAutoresizingMask::HEIGHT_SIZABLE,
        );

        if let Some(info) = &state.file_inspector {
            let mut iy = h - 16.0;
            let props = [
                ("Name", info.name.as_str()),
                ("Type", info.file_type.as_str()),
                ("Size", info.size.as_str()),
                ("Path", info.path.as_str()),
            ];
            for (label, value) in &props {
                iy -= 20.0;
                if iy < 0.0 { break; }

                let mut lbl = NSTextField::label(label);
                lbl.set_font_size(10.0);
                lbl.set_text_color(theme::color(keys::CLR_TEXT_TERTIARY));
                lbl.set_bordered(false);
                let mut lv = lbl.view_clone();
                lv.set_frame(NSRect::new(12.0, iy, 60.0, 18.0));
                lv.set_background_color(None);
                insp_pane.add_subview(lv);

                let mut val = NSTextField::label(value);
                val.set_font_size(10.0);
                val.set_text_color(theme::color(keys::CLR_TEXT_LABEL));
                val.set_bordered(false);
                let mut vv = val.view_clone();
                vv.set_frame(NSRect::new(74.0, iy, insp_w - 86.0, 18.0));
                vv.set_background_color(None);
                insp_pane.add_subview(vv);
            }
        } else {
            let mut placeholder = NSTextField::label("Select a file");
            placeholder.set_font_size(11.0);
            placeholder.set_text_color(theme::color(keys::CLR_TEXT_TERTIARY));
            placeholder.set_bordered(false);
            let mut pv = placeholder.view_clone();
            pv.set_frame(NSRect::new(12.0, h / 2.0, insp_w - 24.0, 20.0));
            pv.set_background_color(None);
            insp_pane.add_subview(pv);
        }

        view.add_subview(insp_pane);

        view
    }
}

/// Format bytes into a human-readable size string.
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

impl EditorPanel for NSProjectBrowserViewController {
    fn view(&self) -> &NSView { &self.view }
    fn view_mut(&mut self) -> &mut NSView { &mut self.view }
    fn on_selection_changed(&mut self, _entity_id: Option<u64>) {
        // Project browser doesn't track entity selection
    }
    fn reload_data(&mut self) {
        self.rebuild();
    }
    fn name(&self) -> &str { "Project Browser" }
    fn min_size(&self) -> f32 { 400.0 }
    fn preferred_size(&self) -> f32 { 600.0 }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

use crate::panels::EditorPanel;
