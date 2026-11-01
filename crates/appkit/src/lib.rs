//! # Windows AppKit
//!
//! A high-performance, Swift-inspired UI framework for Windows, providing
//! a native-feeling macOS experience with AppKit-style APIs.
//!
//! ## Module Organisation
//!
//! - **Core**: application, event, graphics, math, render, responder, view, window
//! - **Controls**: alert, box_view, browser, button, collection_view, color_well, control,
//!   disclosure_triangle, image_view, outline_view, popup_button, progress_indicator,
//!   scroll_view, search_field, segmented_control, slider, split_view, stack_view,
//!   stepper, switch, tab_view, table_view, text_field, text_view
//! - **Layout**: layout_engine, layout_constants, culling
//! - **Theming & Rendering**: theme, gpu, animation, svg_icons
//! - **Panels & Containers**: panels, toolbar, menu, timeline, shader_graph_view
//! - **Services**: application, assets, dialog, ecs, inspector, project, reactive

// Framework crate — many fields/imports are reserved for future use.
#![allow(unused_imports, unused_variables, dead_code, unused_mut)]

// ── Core (src/core/) ─────────────────────────────────────────────────────────
#[path = "core/application/mod.rs"]
pub mod application;
#[path = "core/event/mod.rs"]
pub mod event;
#[path = "core/graphics/mod.rs"]
pub mod graphics;
#[path = "core/math/mod.rs"]
pub mod math;
#[path = "core/render/mod.rs"]
pub mod render;
#[path = "core/responder/mod.rs"]
pub mod responder;
#[path = "core/view/mod.rs"]
pub mod view;
#[path = "core/window/mod.rs"]
pub mod window;

// ── Controls (src/controls/) ─────────────────────────────────────────────────
#[path = "controls/alert/mod.rs"]
pub mod alert;
#[path = "views/box_view/mod.rs"]
pub mod box_view;
#[path = "controls/browser/mod.rs"]
pub mod browser;
#[path = "controls/button/mod.rs"]
pub mod button;
#[path = "controls/color_well/mod.rs"]
pub mod color_well;
#[path = "controls/color_panel/mod.rs"]
pub mod color_panel;
#[path = "controls/combo_box/mod.rs"]
pub mod combo_box;
#[path = "controls/control/mod.rs"]
pub mod control;
#[path = "controls/date_picker/mod.rs"]
pub mod date_picker;
#[path = "controls/disclosure_triangle/mod.rs"]
pub mod disclosure_triangle;
#[path = "controls/level_indicator/mod.rs"]
pub mod level_indicator;
#[path = "controls/open_panel/mod.rs"]
pub mod open_panel;
#[path = "controls/path_control/mod.rs"]
pub mod path_control;
#[path = "controls/popup_button/mod.rs"]
pub mod popup_button;
#[path = "controls/progress_indicator/mod.rs"]
pub mod progress_indicator;
#[path = "controls/save_panel/mod.rs"]
pub mod save_panel;
#[path = "controls/search_field/mod.rs"]
pub mod search_field;
#[path = "controls/segmented_control/mod.rs"]
pub mod segmented_control;
#[path = "controls/slider/mod.rs"]
pub mod slider;
#[path = "controls/stepper/mod.rs"]
pub mod stepper;
#[path = "controls/switch/mod.rs"]
pub mod switch;
#[path = "controls/text_field/mod.rs"]
pub mod text_field;
#[path = "controls/text_view/mod.rs"]
pub mod text_view;
#[path = "controls/token_field/mod.rs"]
pub mod token_field;

// ── Views (src/views/) ───────────────────────────────────────────────────────
pub mod views;

// Re-export view modules at crate root for backward compatibility
pub use views::{
    collection_view, image_view, menu, outline_view, render_view, 
    scroll_view, shader_graph_view, sidebar, split_view, stack_view, 
    tab_view, table_view, timeline, toolbar,
};

#[path = "panels/mod.rs"]
pub mod panels;

// ── Layout (src/core/) ───────────────────────────────────────────────────────
#[path = "core/layout_engine/mod.rs"]
pub mod layout_engine;
#[path = "core/layout_constants/mod.rs"]
pub mod layout_constants;
pub mod layout;  // Unified spacing and hierarchy system

// ── Rendering (src/gpu/) ─────────────────────────────────────────────────────
#[path = "core/animation/mod.rs"]
pub mod animation;
pub mod glass;
pub mod gpu;
#[path = "core/graphics/icons/mod.rs"]
pub mod svg_icons;
#[path = "gpu/culling.rs"]
pub mod culling;
pub mod theme;

// ── Workspace (src/workspace/) ────────────────────────────────────────────────
pub mod workspace;

// ── Services (src/services/) ─────────────────────────────────────────────────
#[path = "services/dialog/mod.rs"]
pub mod dialog;
#[path = "services/ecs/mod.rs"]
pub mod ecs;
#[path = "services/project/mod.rs"]
pub mod project;

// ── Framework types ──────────────────────────────────────────────────────────
mod error;
pub mod prelude;

pub use error::{NSError, NSResult};
pub use layout_constants::NSLayoutConstants;

// ── Compat re-exports ────────────────────────────────────────────────────────
pub use window::controller::WindowController;
pub use window::controller::{WindowProvider, WindowControllerActions};
pub use window::{NSWindowController, NSViewController};

pub use glam;

// ── Public API re-exports ────────────────────────────────────────────────────
// TODO: activity_bar module is empty, needs implementation
// pub use activity_bar::{NSActivityBar, NSActivityBarItem, ActivityBarGroup};
pub use alert::NSAlert;
pub use application::{NSAppDelegate, NSApplication};
pub use box_view::NSBox;
pub use browser::{NSBrowser, NSBrowserDelegate};
pub use button::{NSBezelStyle, NSButton, NSButtonType, NSImagePosition};
pub use collection_view::NSCollectionView;
pub use color_panel::NSColorPanel;
pub use combo_box::NSComboBox;
pub use control::{NSAction, NSActionWith, NSControl, NSControlTarget, NSPredicate};
pub use date_picker::{NSDatePicker, NSDatePickerDelegate, NSDatePickerElementFlags, NSDatePickerMode, NSDatePickerStyle};
pub use event::NSEvent;
pub use graphics::{
    NSAutoresizingMask, NSColor, NSEdgeInsets, NSLayoutAttribute, NSLayoutRelation, NSPoint,
    NSRect, NSSize, NSUserInterfaceLayoutOrientation,
};
pub use image_view::NSImageView;
pub use menu::{Menu, MenuBar, MenuBarView, MenuItem};
pub use open_panel::NSOpenPanel;
pub use outline_view::{NSOutlineView, NSOutlineViewDataSource};
pub use responder::{NSResponder, NSResponderChain};
pub use save_panel::NSSavePanel;
pub use scroll_view::NSScrollView;
pub use search_field::NSSearchField;
pub use segmented_control::NSSegmentedControl;
pub use sidebar::{NSSidebar, NSSidebarConfig, NSSidebarItem};
pub use slider::NSSlider;
pub use split_view::{
    DividerDrag, NSSplitView, NSSplitViewController, NSSplitViewDividerStyle, NSSplitViewItem,
};
pub use stack_view::{NSStackView, NSStackViewDistribution};
pub use tab_view::{NSTabView, NSTabViewItem};
pub use text_field::{NSTextAlignment, NSTextField};
pub use text_view::{NSTextView, NSTextViewDelegate};
pub use token_field::{NSTokenField, NSTokenFieldDelegate};
pub use theme::{AppearanceName, NSAppearance};
pub use timeline::NSTimelineView;
pub use toolbar::{NSToolbar, NSToolbarItem};
pub use view::{NSLayoutConstraint, NSLayoutMode, NSThemeable, NSView, NSViewDelegate};
pub use window::{NSWindow, NSWindowDelegate, NSWindowStyle};
