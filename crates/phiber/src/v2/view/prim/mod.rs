//! Primitives — constructors legal only inside `Part.body`.
//!
//! Split by widget family — one file per primitive (row, table, tree,
//! panel, rail, menu, …), mirroring the v1 `buikit` component layout
//! (`components/rail`, `components/tree`, `components/tab`, …) and the
//! render layer's `prim/*` decomposition. Every family stays well under
//! 150 lines; this module only declares and re-exports, so call sites
//! keep using `view::prim::<Name>` / `BView::<ctor>` unchanged.

pub mod accordion;
pub mod action;
pub mod button;
pub mod canvas;
pub mod control;
pub mod field;
pub mod icon;
pub mod menu;
pub mod panel;
pub mod rail;
pub mod row;
pub mod tab;
pub mod table;
pub mod tree;

pub use action::{BAction, BZoneToggle};
pub use control::BControl;
pub use menu::BPopoverSide;
pub use panel::BPanelRole;
pub use rail::{BRailEntry, BRailGroup, BRailKind, BRailSide, BRailView};
pub use row::BRowShade;
pub use table::{BColAlign, BColWidth, BSortDirection, BTableCell};
