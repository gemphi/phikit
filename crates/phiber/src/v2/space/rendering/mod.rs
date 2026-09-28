//! Rendering workspace. Structure only: the content — areas and the hosts
//! they offer — is in `set.rs`.

pub mod set;

pub use set::{AREAS, ICON, LABEL};

use crate::v2::space::{BSpaceSpec, BSpaceType};

pub static SPEC: BSpaceSpec = BSpaceSpec {
    kind: BSpaceType::Rendering,
    label: LABEL,
    icon: ICON,
    areas: AREAS,
};
