//! NLA host. Structure only: what the host *is*, and where its
//! content lives. The regions, panes and parts are in `set.rs`, and the
//! widgets they name are in `part::`.

pub mod set;

use crate::v2::host::{BHostSpec, BHostType};

pub use set::REGIONS;

/// Key used for this host inside a `BPath`.
pub const SLUG: &str = "nla";

pub static SPEC: BHostSpec = BHostSpec {
    kind: BHostType::Nla,
    label: "NLA",
    icon: "nla",
    regions: set::REGIONS,
};
