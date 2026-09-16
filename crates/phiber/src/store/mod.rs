//! Store - all static specs for the Phiber editor.
//!
//! Hierarchy (top-down, matching the docs):
//!   spaces/   → BSpaceType, BSpaceSpec (workspace tabs, zone/area grids)
//!   zones/    → BZoneType enum, BZoneSpec (grid positions)
//!   areas/    → BAreaType, BAreaSpec (idx, items)
//!   hosts/    → BHostType, BHostSpec (per-host-type specs)
//!   regions/  → BRegionType (Header, Center, Footer), BRegionSpec
//!   panes/    → BPaneType, BPaneSpec
//!   parts/    → BPartType (Menubar, Modebar, Switcher, ...), BPart
//!   modals/   → help, profile, settings modal specs
//!
//! Store modules are plural. State modules are singular.
//! Old code moved to `old/` - will be migrated incrementally.

pub mod areas;
pub mod hosts;
pub mod menus;
pub mod modals;
pub mod panel_role;
pub mod panes;
pub mod parts;
pub mod regions;
pub mod spaces;
pub mod spec;
pub mod zones;

pub use modals::{BHelpType, BModalType, BProfileType, BSettingsType};
/// Flat re-exports of items that are referenced directly as `crate::store::...`.
pub use panel_role::BPanelRole;
