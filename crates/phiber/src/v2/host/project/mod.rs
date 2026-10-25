//! Project Browser host — Reality Composer Pro's bottom-panel browser over
//! the open scene's assets: a navigator of folders (All Files, then the
//! project's kinds), a grid of cards with its toolbar, and an inspector of
//! the card picked. Structure only; the regions are in `set.rs`, the
//! content in `part::project`.

pub mod set;

use crate::v2::host::{BHostSpec, BHostType};

pub use set::REGIONS;

/// Key used for this host inside a `BPath`.
pub const SLUG: &str = "project";

pub static SPEC: BHostSpec = BHostSpec {
    kind: BHostType::Project,
    label: "Project Browser",
    icon: "folder",
    regions: set::REGIONS,
};
