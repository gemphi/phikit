//! Primitives — constructors legal only inside `Part.body`.
//!
//! Split by widget family — one file per primitive (row, table, tree,
//! panel, rail, menu, …), mirroring the v1 `buikit` component layout
//! (`components/rail`, `components/tree`, `components/tab`, …) and the
//! render layer's `prim/*` decomposition. Every family stays well under
//! 150 lines; this module only declares and re-exports, so call sites
//! keep using `view::prim::<Name>` / `BView::<ctor>` unchanged.

pub mod action;
pub mod canvas;
pub mod card;
pub mod control;
pub mod ctrl;
pub mod field;
pub mod icon;
pub mod menu;
pub mod split;
pub mod prose;
pub mod rail;
pub mod row;
pub mod table;
pub mod thumb;
pub mod tree;

pub use action::{BAction, BZoneToggle};
pub use card::{BCard, BCardHead, BCardKind, BColumn, BFold, BStrip, with_peers};
pub use control::BControl;
pub use ctrl::{BCtrl, BCtrlKind, BOption};
pub use menu::{BPick, BPopoverSide};
pub use prose::BProse;
pub use rail::{BRailEntry, BRailGroup, BRailKind, BRailSide, BRailView};
pub use row::BRowShade;
pub use table::{BColAlign, BColWidth, BSortDirection, BTableCell};
pub use tree::BTreeDrag;
pub use thumb::BThumb;
