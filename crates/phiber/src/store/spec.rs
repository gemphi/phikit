//! BSpec - spec of specs. Each spec has an idx and items.

use crate::store::areas::BAreaType;
use crate::store::hosts::BHostType;
use crate::store::modals::BModalType;
use crate::store::panes::BPaneType;
use crate::store::parts::BPartType;
use crate::store::regions::BRegionType;
use crate::store::spaces::BSpaceType;
use crate::store::zones::BZoneType;

/// Unified marker - identifies a spec's level + variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSpecType {
    BSpace(BSpaceType),
    BZone(BZoneType),
    BArea(BAreaType),
    BHost(BHostType),
    BRegion(BRegionType),
    BPane(BPaneType),
    BPart(BPartType),
    BModal(BModalType),
}

/// A spec of specs. Each spec has an idx and items.
pub trait BSpec {
    fn idx(&self) -> BSpecType;
    fn items(&self) -> Vec<&dyn BSpec>;
}
