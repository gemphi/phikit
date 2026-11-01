//! Floating Panels Ã¢â‚¬â€ Core implementation
//!
//! Detachable windows for bottom tabs, with close/dock/maximize controls.

use crate::theme;
use crate::*;
use std::sync::{Arc, Mutex};

/// A floating panel window with traffic-light controls.
pub struct FloatingPanel {
    pub id: u32,
    pub title: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub view: NSView,
    on_close: Option<Arc<dyn Fn() + Send + Sync>>,
    on_dock: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl FloatingPanel {
    pub fn new(id: u32, title: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut view = NSView::new(NSRect::new(x, y, width, height))
            .with_name(&format!("FloatingPanel_{}", id));
        view.set_background_color(Some(theme::color(theme::CLR_BG_PANEL)));

        // Header with traffic lights
        let mut header = NSView::new(NSRect::new(0.0, height - 30.0, width, 30.0));
        header.set_background_color(Some(theme::color(theme::CLR_BG_HEADER)));
        header.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MIN_Y_MARGIN,
        );

        let mut close_btn = NSButton::circle(12.0, theme::color(theme::CLR_TRAFFIC_CLOSE));
        close_btn
            .view_mut()
            .set_frame(NSRect::new(10.0, 9.0, 12.0, 12.0));
        header.add_subview(close_btn.view_clone());

        let mut dock_btn = NSButton::circle(12.0, theme::color(theme::CLR_TRAFFIC_MIN));
        dock_btn
            .view_mut()
            .set_frame(NSRect::new(28.0, 9.0, 12.0, 12.0));
        header.add_subview(dock_btn.view_clone());

        let mut max_btn = NSButton::circle(12.0, theme::color(theme::CLR_TRAFFIC_ZOOM));
        max_btn
            .view_mut()
            .set_frame(NSRect::new(46.0, 9.0, 12.0, 12.0));
        header.add_subview(max_btn.view_clone());

        let title_label = NSTextField::label(title);
        title_label
            .view()
            .set_frame(NSRect::new((width - 150.0) / 2.0, 5.0, 150.0, 20.0));
        title_label.view().set_autoresizing_mask(
            NSAutoresizingMask::MIN_X_MARGIN | NSAutoresizingMask::MAX_X_MARGIN,
        );
        header.add_subview(title_label.view_clone());

        view.add_subview(header);

        let mut content = NSView::new(NSRect::sized(width, height - 30.0));
        content.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        content.set_name("FloatingPanelContent");
        view.add_subview(content);

        Self {
            id,
            title: title.to_string(),
            x,
            y,
            width,
            height,
            view,
            on_close: None,
            on_dock: None,
        }
    }

    pub fn set_on_close(&mut self, callback: Arc<dyn Fn() + Send + Sync>) {
        self.on_close = Some(callback);
    }

    pub fn set_on_dock(&mut self, callback: Arc<dyn Fn() + Send + Sync>) {
        self.on_dock = Some(callback);
    }

    pub fn content_view(&self) -> NSView {
        self.view
            .subviews()
            .iter()
            .find(|v| v.name == "FloatingPanelContent")
            .cloned()
            .unwrap_or_else(|| NSView::new(NSRect::sized(self.width, self.height - 30.0)))
    }
}

/// Manager for all floating panels
pub struct FloatingPanelManager {
    panels: Arc<Mutex<Vec<FloatingPanel>>>,
    container_view: NSView,
}

impl FloatingPanelManager {
    pub fn new(container_rect: NSRect) -> Self {
        let container_view = NSView::new(container_rect).with_name("FloatingPanelContainer");
        Self {
            panels: Arc::new(Mutex::new(Vec::new())),
            container_view,
        }
    }

    pub fn create_panel(&mut self, id: u32, title: &str) {
        let panels = self.panels.lock().unwrap();
        let offset = panels.len() as f32 * 30.0;
        drop(panels);

        let x = 80.0 + offset;
        let y = 120.0 + offset;

        let mut panel = FloatingPanel::new(id, title, x, y, 700.0, 400.0);

        let panels_arc = self.panels.clone();
        let panel_id = id;
        panel.set_on_close(Arc::new(move || {
            let mut panels = panels_arc.lock().unwrap();
            panels.retain(|p| p.id != panel_id);
        }));

        let panels_arc2 = self.panels.clone();
        let panel_id2 = id;
        panel.set_on_dock(Arc::new(move || {
            let mut panels = panels_arc2.lock().unwrap();
            panels.retain(|p| p.id != panel_id2);
        }));

        self.panels.lock().unwrap().push(panel);
    }

    pub fn close_panel(&mut self, id: u32) {
        let mut panels = self.panels.lock().unwrap();
        panels.retain(|p| p.id != id);
    }

    pub fn dock_panel(&mut self, id: u32) {
        self.close_panel(id);
    }

    pub fn container_view(&self) -> &NSView {
        &self.container_view
    }

    pub fn refresh(&mut self) {
        self.container_view.remove_all_subviews();
        let panels = self.panels.lock().unwrap();
        for panel in panels.iter() {
            self.container_view.add_subview(panel.view.clone());
        }
    }
}
