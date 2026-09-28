//! Drivers regions — the same table as `action`, re-exported rather
//! than repeated: two hosts that lay out identically should not be able
//! to drift apart.

pub use crate::v2::host::action::set::REGIONS;
