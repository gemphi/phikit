//! # NSStatisticsViewController — Performance Monitor Panel
//!
//! Implements the assembly and update logic for the performance monitor.
//! Mirrors Apple's view controller pattern for system statistics overlays.
//!
//! ## Apple Documentation
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//! - [NSTableView](https://developer.apple.com/documentation/appkit/nstableview)
//!
//! ## See Also
//! - [NSTableViewDataSource](https://developer.apple.com/documentation/appkit/nstableviewdatasource)
//! - Custom component for real-time statistics display

use super::model::*;
use crate::prelude::*;
use crate::theme::{self, keys};
use crate::layout::{spacing, hierarchy};
use crate::table_view::{NSTableView, NSTableColumn, NSTableViewDataSource};
use std::sync::{Arc, Mutex};

/// Data source for the statistics table view.
struct StatsDataSource {
    state: Arc<Mutex<NSStatisticsData>>,
}

impl NSTableViewDataSource for StatsDataSource {
    fn number_of_rows(&self) -> usize {
        6 // FPS, Draw Calls, Triangles, Meshes, Entity Count, Render Time
    }

    fn value_for(&self, row: usize, column: &str) -> String {
        let state = self.state.lock().unwrap();
        match column {
            "metric" => match row {
                0 => "FPS".to_string(),
                1 => "Draw Calls".to_string(),
                2 => "Triangles".to_string(),
                3 => "Meshes".to_string(),
                4 => "Entity Count".to_string(),
                5 => "Render Time".to_string(),
                _ => String::new(),
            },
            "value" => match row {
                0 => format!("{:.0}", state.fps),
                1 => format!("{}", state.draw_calls),
                2 => format!("{}", state.triangles),
                3 => format!("{}", state.textures),
                4 => format!("{}", state.entity_count),
                5 => format!("{:.2} ms", state.render_time_ms),
                _ => String::new(),
            },
            _ => String::new(),
        }
    }
}

pub struct NSStatisticsViewController {
    pub view: NSView,
    pub state: Arc<Mutex<NSStatisticsData>>,
    table_view: NSTableView,
}

impl NSStatisticsViewController {
    pub fn new(rect: NSRect) -> Self {
        let mut view = NSView::new(rect).with_name("StatisticsPanel");
        view.set_background_color(Some(hierarchy::bg_color("panel")));
        view.set_corner_radius(crate::theme::ThemeToken::RADIUS_PANEL());
        view.set_glass(true);
        view.set_clips_to_bounds(true);

        let state = Arc::new(Mutex::new(NSStatisticsData::default()));

        // Create NSTableView for stats display
        let table_rect = NSRect::new(
            spacing::padding::PANEL,
            spacing::padding::PANEL,
            rect.width() - spacing::padding::PANEL * 2.0,
            rect.height() - spacing::padding::PANEL * 2.0,
        );
        let mut table_view = NSTableView::new(table_rect);
        
        // Add columns: Metric and Value
        table_view.add_column(NSTableColumn::new("metric", "Metric", table_rect.size.width * 0.5));
        table_view.add_column(NSTableColumn::new("value", "Value", table_rect.size.width * 0.5));
        
        // Set data source
        let data_source = StatsDataSource {
            state: state.clone(),
        };
        table_view.set_data_source(data_source);
        
        // Add table view to the panel
        view.add_subview(table_view.view_clone());

        Self {
            view,
            state,
            table_view,
        }
    }

    pub fn update(&mut self, data: NSStatisticsData) {
        let _ = self.state.lock().ok().map(|mut s| *s = data);
        self.table_view.reload_data();
    }

    pub fn fps_text(&self) -> String {
        let s = self.state.lock().unwrap();
        format!("{:.0} fps", s.fps)
    }

    pub fn draw_calls_text(&self) -> String {
        let s = self.state.lock().unwrap();
        format!("{} draw calls", s.draw_calls)
    }

    pub fn triangle_count_text(&self) -> String {
        let s = self.state.lock().unwrap();
        format!("{} triangles", s.triangles)
    }

    pub fn mesh_count_text(&self) -> String {
        let s = self.state.lock().unwrap();
        format!("{} meshes", s.textures)
    }

    pub fn entity_count_text(&self) -> String {
        let s = self.state.lock().unwrap();
        format!("{} entities", s.entity_count)
    }

    pub fn render_time_text(&self) -> String {
        let s = self.state.lock().unwrap();
        format!("{:.2} ms", s.render_time_ms)
    }
}
