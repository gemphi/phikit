//! Sidebar part spec - N-panel tabs in the Center region's right pane.
//!
//! Ref: Blender's N-panel (Item, Tool, View tabs)

pub mod def;
pub mod set;

/// A single sidebar tab.
#[derive(Clone, Copy, Debug)]
pub struct SidebarTab {
    pub idx: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
}
