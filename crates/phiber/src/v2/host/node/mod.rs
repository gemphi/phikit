//! Node Editor host. Structure only: what the host *is*, and where its
//! content lives. The regions, panes and parts are in `set.rs`, and the
//! widgets they name are in `part::`.

pub mod set;

use crate::v2::host::{BHostSpec, BHostType};

pub use set::REGIONS;

/// Key used for this host inside a `BPath`.
pub const SLUG: &str = "node";

pub static SPEC: BHostSpec = BHostSpec {
    kind: BHostType::Node,
    label: "Node Editor",
    icon: "node",
    regions: set::REGIONS,
};
