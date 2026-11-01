//! PSegmented — segmented control (tab-style picker).
//!
//! Swift: `Picker(selection: $sel) { ... }.pickerStyle(.segmented)`.

use crate::traits::{PStyle, PWidget};
use rust_appkit::prelude::*;
use std::sync::Arc;

/// Segmented control. Swift: `Picker(...).pickerStyle(.segmented)`.
pub struct PSegmented {
    labels: Vec<String>,
    selected: isize,
    width: f32,
    height: f32,
    on_change: Option<Arc<dyn Fn(isize) + Send + Sync>>,
}

impl PSegmented {
    /// Creates a segmented control with the given labels. Swift: `Picker { ForEach options }`
    pub fn new<S: Into<String>>(labels: impl IntoIterator<Item = S>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            selected: 0,
            width: 240.0,
            height: 32.0,
            on_change: None,
        }
    }
    
    /// Sets the initial selected segment. Swift: binding initial value
    pub fn selected(mut self, index: isize) -> Self {
        self.selected = index;
        self
    }
    
    /// Selection change callback. Swift: binding write-back
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(isize) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(callback));
        self
    }
}

impl PStyle for PSegmented {
    fn color(self, _color: NSColor) -> Self { self }
    fn background(self, _color: NSColor) -> Self { self }
    fn corner_radius(self, _radius: f32) -> Self { self }
    fn padding(self, _padding: f32) -> Self { self }
    
    fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    
    fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl PWidget for PSegmented {
    fn build(self) -> NSView {
        let mut seg = NSSegmentedControl::new(NSRect::sized(self.width, self.height));
        seg.set_segment_count(self.labels.len() as isize);
        for (i, label) in self.labels.iter().enumerate() {
            seg.set_label(label, i as isize);
        }
        seg.set_selected_segment(self.selected);
        if let Some(cb) = self.on_change {
            seg.set_action(move |i| cb(i));
        }
        seg.view().clone()
    }
}
