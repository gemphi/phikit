//! Workspace Layout — panel docking and arrangement system.
//!
//! Provides a flexible workspace where panels can dock left, right, bottom,
//! float, or hide entirely, similar to modern IDE layouts.
//!
//! This is standalone infrastructure — not a panel itself.
//! It defines *where* panels go, not *what* they render.
//!
//! ## Module layout
//!
//! - `layout` — pure constants (panel sizes, scale factors)
//! - `constraints` — LayoutConstraints struct + bottom panel alignment
//! - `presets` — WorkspacePreset impl (all named arrangements)
//! - `metrics` — runtime metric functions (theme-token-driven)
//! - `state` — global active-layout state management

pub mod layout;
pub mod constraints;
mod presets;
pub mod metrics;
pub mod state;

/// Re-export layout constants as `defaults` for backward compatibility.
pub use layout as defaults;

// Re-export public API
pub use constraints::{LayoutConstraints, align_bottom_panel_columns, align_bottom_panel_columns_from_positions};
pub use metrics::*;
pub use state::*;

// ═══════════════════════════════════════════════════════════════════════
// TYPES — pure data definitions, no logic
// ═══════════════════════════════════════════════════════════════════════

/// Where a panel can be docked in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockPosition {
    Left,
    Right,
    Bottom,
    Top,
    Center,
    Floating,
    Hidden,
}

/// A panel's layout slot within the workspace.
#[derive(Debug, Clone)]
pub struct PanelSlot {
    pub id: &'static str,
    pub dock: DockPosition,
    pub order: u8,
    pub visible: bool,
    pub min_size: f32,
    pub preferred_size: f32,
    pub max_size: f32,
    pub collapsible: bool,
}

/// The full workspace arrangement — one of these is active at a time.
#[derive(Debug, Clone)]
pub struct WorkspacePreset {
    pub name: &'static str,
    pub slots: Vec<PanelSlot>,
    pub sidebar_position: DockPosition,
    pub inspector_position: DockPosition,
    pub bottom_panel_visible: bool,
    pub toolbar_visible: bool,
    pub activity_bar_position: DockPosition,
}
