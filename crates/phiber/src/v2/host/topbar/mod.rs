//! Top Bar host. Structure only: what the host *is*, and where its
//! content lives. The regions, panes and parts are in `set.rs`, and the
//! widgets they name are in `part::`.

pub mod set;

use crate::v2::host::{BHostSpec, BHostType};

pub use set::REGIONS;

/// Key used for this host inside a `BPath`.
pub const SLUG: &str = "topbar";

pub static SPEC: BHostSpec = BHostSpec {
    kind: BHostType::Topbar,
    label: "Top Bar",
    icon: "topbar",
    regions: set::REGIONS,
};
