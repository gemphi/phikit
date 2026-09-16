//! Custom set — the hosts that provide a center canvas. Sourced from
//! `BHostType::SWITCHABLE`; Topbar and Statusbar hosts have no center.

use crate::v2::host::BHostType;

pub fn hosts_with_center() -> &'static [BHostType] {
    BHostType::SWITCHABLE
}
