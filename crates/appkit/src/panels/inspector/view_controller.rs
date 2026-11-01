//! # NSInspectorViewController — Property Inspector Panel
//!
//! Builds and manages the property inspector panel view tree.
//! Follows Apple's NSViewController pattern with automatic layout.
//!
//! ## Apple Documentation
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [NSStackView](https://developer.apple.com/documentation/appkit/nsstackview)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//!
//! ## Architecture
//!
//! The view controller handles:
//! - Building the NSStackView layout with property rows
//! - Rebuilding on selection change
//! - Styling via theme tokens (PanelDefaults)
//! - Property editing callbacks
//!
//! rust-kit-pro's InspectorPanel is a thin wrapper that feeds scene data into this.
//!
//! ## See Also
//! - [NSOutlineView](https://developer.apple.com/documentation/appkit/nsoutlineview)
//! - [NSTableView](https://developer.apple.com/documentation/appkit/nstableview)

use super::model::*;
use crate::prelude::*;
use crate::stack_view::NSStackView;
use crate::theme::{self, keys, ThemeToken as T};
use crate::panels::defaults::LabelDefaults;
use crate::layout::{spacing, hierarchy};
use std::sync::{Arc, Mutex};

/// Callback type for property value changes (Apple pattern: delegate callbacks).
pub type PropertyChangeCallback = Arc<Mutex<dyn Fn(&str, &NSInspectorValue) + Send + Sync>>;

/// Inspector view controller — builds and manages the property editor view.
/// Follows Apple's NSViewController pattern.
pub struct NSInspectorViewController {
    pub view: NSView,
    pub state: NSInspectorState,
    on_property_changed: Option<PropertyChangeCallback>,
}

impl NSInspectorViewController {
    /// Create a new inspector view controller with the given frame.
    pub fn new(rect: NSRect) -> Self {
        let mut view = NSView::new(rect).with_name("InspectorPanel");
        view.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        view.set_background_color(Some(hierarchy::bg_color("inspector")));
        view.set_corner_radius(T::RADIUS_PANEL());
        view.set_glass(true);
        view.set_clips_to_bounds(true);

        Self {
            view,
            state: NSInspectorState::default(),
            on_property_changed: None,
        }
    }

    // ─── State Management (Apple pattern: setters) ──────────────────

    /// Set the property sections to display.
    pub fn set_properties(&mut self, sections: Vec<NSInspectorSection>) {
        self.state.sections = sections;
        self.rebuild();
    }

    /// Update the inspector with new state and rebuild the view.
    pub fn set_state(&mut self, state: NSInspectorState) {
        self.state = state;
        self.rebuild();
    }

    /// Clear the selection (show "No Selection" placeholder).
    pub fn clear_selection(&mut self) {
        self.state = NSInspectorState::default();
        self.rebuild();
    }

    /// Clear the inspector (alias for clear_selection).
    pub fn clear(&mut self) {
        self.clear_selection();
    }

    // ─── Callbacks (Delegate Pattern) ───────────────────────────────

    /// Set callback for property value changes (Apple pattern: setDelegate:).
    pub fn set_on_property_changed<F>(&mut self, callback: F)
    where
        F: Fn(&str, &NSInspectorValue) + Send + Sync + 'static,
    {
        self.on_property_changed = Some(Arc::new(Mutex::new(callback)));
    }

    // ─── Section Management ──────────────────────────────────────────

    /// Toggle a section's collapsed state by index.
    pub fn toggle_section(&mut self, index: usize) {
        if let Some(section) = self.state.sections.get_mut(index) {
            section.collapsed = !section.collapsed;
            self.rebuild();
        }
    }

    // ─── View Building (NSStackView-based automatic layout) ─────────

    /// Rebuild the view with current state using NSStackView for automatic layout.
    fn rebuild(&mut self) {
        self.view.remove_all_subviews();

        let panel_bounds = self.view.bounds();
        let margin = spacing::padding::INSPECTOR;
        let content_width = panel_bounds.width() - margin * 2.0;

        // Create vertical stack for all content
        let mut content = NSStackView::vertical();
        content.set_spacing(spacing::gap::INSPECTOR_SECTION);
        content.set_edge_insets(NSEdgeInsets::new(margin, margin, margin, margin));
        content.view_mut().set_name("InspectorContent");
        content.view_mut().set_frame(panel_bounds);
        content.view_mut().set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );

        // Header or "No Selection"
        match &self.state.entity_name {
            Some(name) => {
                // Entity header
                let mut header = LabelDefaults::header(name);
                header.view_mut().set_frame(NSRect::sized(content_width, 24.0));
                content.add_arranged_subview(header.view_clone());

                // Separator
                content.add_arranged_subview(Self::separator(content_width));

                // Sections
                for section in &self.state.sections {
                    self.add_section_to_stack(&mut content, section, content_width);
                }

                // Entity ID at bottom
                if let Some(id) = self.state.entity_id {
                    content.add_arranged_subview(Self::separator(content_width));
                    content.add_arranged_subview(Self::detail_row("ID", &id.to_string(), content_width));
                }
            }
            None => {
                // No selection placeholder
                let mut lbl = LabelDefaults::tertiary("No Selection");
                lbl.view_mut().set_frame(NSRect::sized(content_width, 22.0));
                content.add_arranged_subview(lbl.view_clone());
            }
        }

        self.view.add_subview(content.view_clone());
    }

    /// Add a section to the stack view.
    fn add_section_to_stack(&self, stack: &mut NSStackView, section: &NSInspectorSection, width: f32) {
        // Section title
        let mut title = LabelDefaults::secondary(&section.title);
        title.view_mut().set_frame(NSRect::sized(width, 22.0));
        stack.add_arranged_subview(title.view_clone());

        if section.collapsed {
            return;
        }

        // Property rows
        for prop in &section.properties {
            let row = self.build_property_row(&prop.label, &prop.value, width);
            stack.add_arranged_subview(row);
        }
    }

    /// Build a property row based on value type.
    fn build_property_row(&self, label: &str, value: &NSInspectorValue, width: f32) -> NSView {
        match value {
            NSInspectorValue::Vec3(v) => Self::vec3_row(label, *v, width),
            NSInspectorValue::Float(f) => Self::float_row(label, *f, width),
            NSInspectorValue::Text(s) => Self::text_row(label, s, width),
            NSInspectorValue::Color(c) => Self::color_row(label, *c, width),
            NSInspectorValue::Bool(b) => Self::bool_row(label, *b, width),
            NSInspectorValue::Int(i) => Self::int_row(label, *i, width),
        }
    }

    // ─── Property Row Builders ───────────────────────────────────────

    fn vec3_row(label: &str, values: [f32; 3], width: f32) -> NSView {
        let mut row = NSStackView::horizontal();
        row.set_spacing(spacing::gap::INSPECTOR_FIELD);
        row.view_mut().set_frame(NSRect::sized(width, 22.0));

        let label_w = 80.0_f32;
        let mut lbl = LabelDefaults::row_label(label);
        lbl.view_mut().set_frame(NSRect::sized(label_w, 22.0));
        row.add_arranged_subview(lbl.view_clone());

        let display = format!("{:.2}  {:.2}  {:.2}", values[0], values[1], values[2]);
        let mut val = LabelDefaults::row_value(&display);
        val.view_mut().set_frame(NSRect::sized(width - label_w - spacing::gap::INSPECTOR_FIELD, 22.0));
        val.view_mut().set_background_color(Some(hierarchy::bg_color("textfield")));
        val.view_mut().set_corner_radius(T::RADIUS_CONTROL());
        row.add_arranged_subview(val.view_clone());

        row.view_clone()
    }

    fn float_row(label: &str, value: f32, width: f32) -> NSView {
        let mut row = NSStackView::horizontal();
        row.set_spacing(spacing::gap::INSPECTOR_FIELD);
        row.view_mut().set_frame(NSRect::sized(width, 22.0));

        let label_w = 80.0_f32;
        let mut lbl = LabelDefaults::row_label(label);
        lbl.view_mut().set_frame(NSRect::sized(label_w, 22.0));
        row.add_arranged_subview(lbl.view_clone());

        let display = format!("{:.3}", value);
        let mut val = LabelDefaults::row_value(&display);
        val.view_mut().set_frame(NSRect::sized(width - label_w - spacing::gap::INSPECTOR_FIELD, 22.0));
        val.view_mut().set_background_color(Some(hierarchy::bg_color("textfield")));
        val.view_mut().set_corner_radius(T::RADIUS_CONTROL());
        row.add_arranged_subview(val.view_clone());

        row.view_clone()
    }

    fn text_row(label: &str, value: &str, width: f32) -> NSView {
        let mut row = NSStackView::horizontal();
        row.set_spacing(spacing::gap::INSPECTOR_FIELD);
        row.view_mut().set_frame(NSRect::sized(width, 22.0));

        let label_w = 80.0_f32;
        let mut lbl = LabelDefaults::row_label(label);
        lbl.view_mut().set_frame(NSRect::sized(label_w, 22.0));
        row.add_arranged_subview(lbl.view_clone());

        let mut val = LabelDefaults::row_value(value);
        val.view_mut().set_frame(NSRect::sized(width - label_w - spacing::gap::INSPECTOR_FIELD, 22.0));
        val.view_mut().set_background_color(Some(hierarchy::bg_color("textfield")));
        val.view_mut().set_corner_radius(T::RADIUS_CONTROL());
        row.add_arranged_subview(val.view_clone());

        row.view_clone()
    }

    fn color_row(label: &str, values: [f32; 4], width: f32) -> NSView {
        let mut row = NSStackView::horizontal();
        row.set_spacing(spacing::gap::INSPECTOR_FIELD);
        row.view_mut().set_frame(NSRect::sized(width, 22.0));

        let label_w = 80.0_f32;
        let mut lbl = LabelDefaults::row_label(label);
        lbl.view_mut().set_frame(NSRect::sized(label_w, 22.0));
        row.add_arranged_subview(lbl.view_clone());

        let display = format!("({:.2}, {:.2}, {:.2}, {:.2})", values[0], values[1], values[2], values[3]);
        let mut val = LabelDefaults::row_value(&display);
        val.view_mut().set_frame(NSRect::sized(width - label_w - spacing::gap::INSPECTOR_FIELD, 22.0));
        val.view_mut().set_background_color(Some(hierarchy::bg_color("textfield")));
        val.view_mut().set_corner_radius(T::RADIUS_CONTROL());
        row.add_arranged_subview(val.view_clone());

        row.view_clone()
    }

    fn bool_row(label: &str, value: bool, width: f32) -> NSView {
        let mut row = NSStackView::horizontal();
        row.set_spacing(spacing::gap::INSPECTOR_FIELD);
        row.view_mut().set_frame(NSRect::sized(width, 22.0));

        let label_w = 80.0_f32;
        let mut lbl = LabelDefaults::row_label(label);
        lbl.view_mut().set_frame(NSRect::sized(label_w, 22.0));
        row.add_arranged_subview(lbl.view_clone());

        let display = if value { "true" } else { "false" };
        let mut val = LabelDefaults::row_value(display);
        val.view_mut().set_frame(NSRect::sized(width - label_w - spacing::gap::INSPECTOR_FIELD, 22.0));
        val.view_mut().set_background_color(Some(hierarchy::bg_color("textfield")));
        val.view_mut().set_corner_radius(T::RADIUS_CONTROL());
        row.add_arranged_subview(val.view_clone());

        row.view_clone()
    }

    fn int_row(label: &str, value: i64, width: f32) -> NSView {
        let mut row = NSStackView::horizontal();
        row.set_spacing(spacing::gap::INSPECTOR_FIELD);
        row.view_mut().set_frame(NSRect::sized(width, 22.0));

        let label_w = 80.0_f32;
        let mut lbl = LabelDefaults::row_label(label);
        lbl.view_mut().set_frame(NSRect::sized(label_w, 22.0));
        row.add_arranged_subview(lbl.view_clone());

        let display = value.to_string();
        let mut val = LabelDefaults::row_value(&display);
        val.view_mut().set_frame(NSRect::sized(width - label_w - spacing::gap::INSPECTOR_FIELD, 22.0));
        val.view_mut().set_background_color(Some(hierarchy::bg_color("textfield")));
        val.view_mut().set_corner_radius(T::RADIUS_CONTROL());
        row.add_arranged_subview(val.view_clone());

        row.view_clone()
    }

    fn detail_row(label: &str, value: &str, width: f32) -> NSView {
        let mut row = NSStackView::horizontal();
        row.set_spacing(spacing::gap::INSPECTOR_FIELD);
        row.view_mut().set_frame(NSRect::sized(width, 22.0));

        let label_w = 80.0_f32;
        let mut lbl = LabelDefaults::row_label(label);
        lbl.view_mut().set_frame(NSRect::sized(label_w, 22.0));
        row.add_arranged_subview(lbl.view_clone());

        let mut val = LabelDefaults::row_value(value);
        val.view_mut().set_frame(NSRect::sized(width - label_w - spacing::gap::INSPECTOR_FIELD, 22.0));
        row.add_arranged_subview(val.view_clone());

        row.view_clone()
    }

    fn separator(width: f32) -> NSView {
        let mut sep = NSView::new(NSRect::sized(width, NSLayoutConstants::GRID_LINE_WIDTH));
        sep.set_background_color(Some(hierarchy::separator_color("inspector")));
        sep
    }

    /// Format a property value for display (legacy method).
    fn format_value(value: &NSInspectorValue) -> String {
        match value {
            NSInspectorValue::Vec3(v) => format!("{:.2}  {:.2}  {:.2}", v[0], v[1], v[2]),
            NSInspectorValue::Float(f) => format!("{:.3}", f),
            NSInspectorValue::Text(s) => s.clone(),
            NSInspectorValue::Color(c) => {
                format!("({:.2}, {:.2}, {:.2}, {:.2})", c[0], c[1], c[2], c[3])
            }
            NSInspectorValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            NSInspectorValue::Int(i) => i.to_string(),
        }
    }
}

impl EditorPanel for NSInspectorViewController {
    fn view(&self) -> &NSView { &self.view }
    fn view_mut(&mut self) -> &mut NSView { &mut self.view }
    
    fn on_selection_changed(&mut self, entity_id: Option<u64>) {
        self.state.entity_id = entity_id;
        if let Some(id) = entity_id {
            // Update entity name from scene
            self.state.entity_name = Some(format!("Entity_{}", id));
        } else {
            self.state.entity_name = None;
        }
        self.rebuild();
    }
    
    fn reload_data(&mut self) {
        self.rebuild();
    }
    
    fn name(&self) -> &str { "Inspector" }
    fn min_size(&self) -> f32 { 260.0 }
    fn preferred_size(&self) -> f32 { 300.0 }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

use crate::panels::EditorPanel;

