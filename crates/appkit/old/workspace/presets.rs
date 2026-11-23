//! Workspace preset implementations — named panel arrangements.
//!
//! Each preset defines a complete workspace layout with panel positions,
//! visibility, and sizing. Similar to IDE layout presets (Debug, Edit, etc.).

use super::{WorkspacePreset, PanelSlot, DockPosition};

impl WorkspacePreset {
    /// Compact layout — minimal panels, maximized viewport.
    pub fn compact() -> Self {
        Self {
            name: "Compact",
            slots: vec![
                PanelSlot {
                    id: "hierarchy",
                    dock: DockPosition::Left,
                    order: 0,
                    visible: true,
                    min_size: 180.0,
                    preferred_size: 220.0,
                    max_size: 400.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "inspector",
                    dock: DockPosition::Right,
                    order: 0,
                    visible: true,
                    min_size: 220.0,
                    preferred_size: 260.0,
                    max_size: 400.0,
                    collapsible: true,
                },
            ],
            sidebar_position: DockPosition::Left,
            inspector_position: DockPosition::Right,
            bottom_panel_visible: false,
            toolbar_visible: true,
            activity_bar_position: DockPosition::Left,
        }
    }

    /// Classic macOS layout — traditional IDE arrangement.
    pub fn classic() -> Self {
        Self {
            name: "Classic",
            slots: vec![
                PanelSlot {
                    id: "hierarchy",
                    dock: DockPosition::Left,
                    order: 0,
                    visible: true,
                    min_size: 200.0,
                    preferred_size: 260.0,
                    max_size: 500.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "inspector",
                    dock: DockPosition::Right,
                    order: 0,
                    visible: true,
                    min_size: 260.0,
                    preferred_size: 300.0,
                    max_size: 500.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "timeline",
                    dock: DockPosition::Bottom,
                    order: 0,
                    visible: true,
                    min_size: 120.0,
                    preferred_size: 180.0,
                    max_size: 400.0,
                    collapsible: true,
                },
            ],
            sidebar_position: DockPosition::Left,
            inspector_position: DockPosition::Right,
            bottom_panel_visible: true,
            toolbar_visible: true,
            activity_bar_position: DockPosition::Left,
        }
    }

    /// Modern layout — streamlined with floating panels.
    pub fn modern() -> Self {
        Self {
            name: "Modern",
            slots: vec![
                PanelSlot {
                    id: "hierarchy",
                    dock: DockPosition::Left,
                    order: 0,
                    visible: true,
                    min_size: 200.0,
                    preferred_size: 240.0,
                    max_size: 400.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "inspector",
                    dock: DockPosition::Right,
                    order: 0,
                    visible: true,
                    min_size: 240.0,
                    preferred_size: 280.0,
                    max_size: 450.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "tools",
                    dock: DockPosition::Left,
                    order: 1,
                    visible: true,
                    min_size: 60.0,
                    preferred_size: 80.0,
                    max_size: 120.0,
                    collapsible: false,
                },
            ],
            sidebar_position: DockPosition::Left,
            inspector_position: DockPosition::Right,
            bottom_panel_visible: false,
            toolbar_visible: true,
            activity_bar_position: DockPosition::Left,
        }
    }

    /// PhiPro — distraction-free, viewport-only layout.
    pub fn phipro() -> Self {
        Self {
            name: "PhiPro",
            slots: vec![],
            sidebar_position: DockPosition::Hidden,
            inspector_position: DockPosition::Hidden,
            bottom_panel_visible: false,
            toolbar_visible: false,
            activity_bar_position: DockPosition::Hidden,
        }
    }

    /// Minimal layout — essential panels only.
    pub fn minimal() -> Self {
        Self {
            name: "Minimal",
            slots: vec![
                PanelSlot {
                    id: "hierarchy",
                    dock: DockPosition::Left,
                    order: 0,
                    visible: true,
                    min_size: 180.0,
                    preferred_size: 200.0,
                    max_size: 350.0,
                    collapsible: true,
                },
            ],
            sidebar_position: DockPosition::Left,
            inspector_position: DockPosition::Hidden,
            bottom_panel_visible: false,
            toolbar_visible: true,
            activity_bar_position: DockPosition::Left,
        }
    }

    /// Editorial layout — optimized for content creation.
    pub fn editorial() -> Self {
        Self {
            name: "Editorial",
            slots: vec![
                PanelSlot {
                    id: "hierarchy",
                    dock: DockPosition::Left,
                    order: 0,
                    visible: true,
                    min_size: 220.0,
                    preferred_size: 280.0,
                    max_size: 500.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "inspector",
                    dock: DockPosition::Right,
                    order: 0,
                    visible: true,
                    min_size: 280.0,
                    preferred_size: 340.0,
                    max_size: 600.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "timeline",
                    dock: DockPosition::Bottom,
                    order: 0,
                    visible: true,
                    min_size: 160.0,
                    preferred_size: 220.0,
                    max_size: 500.0,
                    collapsible: true,
                },
            ],
            sidebar_position: DockPosition::Left,
            inspector_position: DockPosition::Right,
            bottom_panel_visible: true,
            toolbar_visible: true,
            activity_bar_position: DockPosition::Left,
        }
    }

    /// Default layout — balanced general-purpose arrangement.
    /// 
    /// Note: This replaces the application-specific `default_rcp()` method
    /// to maintain proper layer boundaries (framework should not reference
    /// application-specific names).
    pub fn default() -> Self {
        Self {
            name: "Default",
            slots: vec![
                PanelSlot {
                    id: "hierarchy",
                    dock: DockPosition::Left,
                    order: 0,
                    visible: true,
                    min_size: 200.0,
                    preferred_size: 260.0,
                    max_size: 500.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "inspector",
                    dock: DockPosition::Right,
                    order: 0,
                    visible: true,
                    min_size: 260.0,
                    preferred_size: 300.0,
                    max_size: 500.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "timeline",
                    dock: DockPosition::Bottom,
                    order: 0,
                    visible: true,
                    min_size: 140.0,
                    preferred_size: 200.0,
                    max_size: 450.0,
                    collapsible: true,
                },
                PanelSlot {
                    id: "tools",
                    dock: DockPosition::Left,
                    order: 1,
                    visible: true,
                    min_size: 60.0,
                    preferred_size: 72.0,
                    max_size: 100.0,
                    collapsible: false,
                },
            ],
            sidebar_position: DockPosition::Left,
            inspector_position: DockPosition::Right,
            bottom_panel_visible: true,
            toolbar_visible: true,
            activity_bar_position: DockPosition::Left,
        }
    }
}
