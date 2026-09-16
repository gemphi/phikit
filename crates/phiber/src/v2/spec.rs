//! `BSpec` — the single trait every store spec at every level implements.

use crate::v2::area::BAreaType;
use crate::v2::host::BHostType;
use crate::v2::modal::BModalType;
use crate::v2::pane::BPaneType;
use crate::v2::part::BPartType;
use crate::v2::region::BRegionType;
use crate::v2::space::BSpaceType;
use crate::v2::zone::BZoneType;

/// Unified level tag — one arm per hierarchy tier (plus Modal peer).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSpecKind {
    Space(BSpaceType),
    Zone(BZoneType),
    Area(BAreaType),
    Host(BHostType),
    Region(BRegionType),
    Pane(BPaneType),
    Part(BPartType),
    Modal(BModalType),
}

/// Every static spec exposes identity, label, icon. Children live on the
/// spec via inherent typed accessors.
pub trait BSpec: Copy + std::fmt::Debug + 'static {
    fn kind(&self) -> BSpecKind;
    fn label(&self) -> &'static str;
    fn icon(&self) -> &'static str {
        ""
    }
}
