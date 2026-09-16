//! Part specs - typed UI components that live inside panes.
//!
//! Parts are the leaves of the hierarchy. They produce BView nodes.
//! Parts are owned directly by BPaneSpec.parts - no cross-level mappings.
//! The Switcher part replaces the old AreaSwitcher modal - it's now
//! a dropdown menu in the header region, not a modal overlay.

pub mod channels;
pub mod controls;
pub mod custom;
pub mod def;
pub mod execute;
pub mod menubar;
pub mod modebar;
pub mod navbar;
pub mod panel;
pub mod scrubbing;
pub mod set;
pub mod sidebar;
pub mod statusbar;
pub mod switcher;
pub mod toolbar;
pub use def::BPartType;

use crate::store::spec::{BSpec, BSpecType};

/// Part types - UI components that live inside panes.
/// Each maps to a BArea* component in buikit.

impl BPartType {
    pub fn label(self) -> &'static str {
        set::LABELS[self as usize]
    }

    pub fn from_name(name: &str) -> Option<BPartType> {
        set::ALL
            .iter()
            .find(|&&p| format!("{:?}", p) == name)
            .copied()
    }

    pub fn component_name(self) -> &'static str {
        set::COMPONENTS[self as usize]
    }

    /// CSS-like role key for `styled_panel` in the bui layer.
    pub fn role(self) -> &'static str {
        match self {
            BPartType::Menubar => "menubar",
            BPartType::Modebar => "modebar",
            BPartType::Switcher => "switcher",
            BPartType::Toolbar => "toolbar",
            BPartType::Sidebar => "sidebar",
            BPartType::Panel => "panel",
            BPartType::Channels => "channels",
            BPartType::Statusbar => "section",
            BPartType::Spacebar => "spacebar",
            _ => "section",
        }
    }

    /// Typed panel role for the BUI layer.
    pub fn panel_role(self) -> crate::store::BPanelRole {
        crate::store::BPanelRole::from_part(self)
    }

    /// All part types.
    pub fn all() -> &'static [BPartType] {
        set::ALL
    }

    /// Get the full spec for this part type.
    pub fn items(self) -> &'static BPart {
        &set::ALL_SPECS[self as usize]
    }

    pub fn with_set(self) -> &'static BPart {
        &set::ALL_SPECS[self as usize]
    }
}

/// Static spec for a part within a pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BPart {
    pub idx: BPartType,
    /// Parts are nodes; their child set is always a part.
    pub items: &'static [BPart],
}

/// Generic part spec - every concrete part spec implements this.
/// Each sub-module's spec struct (MenubarPartSpec, ToolbarPartSpec, etc.)
/// implements `BPartSpec` in its own mod.rs.
pub trait BPartSpec {
    fn idx(&self) -> BPartType;
    fn items(&self) -> &'static [BPart] {
        &[]
    }
    fn label(&self) -> &'static str {
        self.idx().label()
    }
}

/// Tie every concrete `BPartSpec` into the universal `BSpec` hierarchy.
impl<T: BPartSpec> BSpec for T {
    fn idx(&self) -> BSpecType {
        BSpecType::BPart(self.idx())
    }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.items().iter().map(|p| p as &dyn BSpec).collect()
    }
}

impl BPart {
    pub const fn new(idx: BPartType) -> Self {
        Self { idx, items: &[] }
    }

    pub fn items(&self) -> &'static [BPart] {
        self.items
    }
}

impl BSpec for BPart {
    fn idx(&self) -> BSpecType {
        BSpecType::BPart(self.idx)
    }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.items.iter().map(|p| p as &dyn BSpec).collect()
    }
}
