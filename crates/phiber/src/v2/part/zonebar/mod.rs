//! Topbar zone-visibility controls for zones declared by the active space.

use crate::v2::host::BHostType;
use crate::v2::space::{BSpaceList, BSpaceType};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BZoneToggle};
use crate::v2::zone::{BZoneType, visibility};

#[derive(Clone, Copy, Debug)]
pub struct BZonebar {
    pub host: BHostType,
}

impl BZonebar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    pub fn declared_zones(space: BSpaceType) -> Vec<BZoneType> {
        visibility::TOGGLEABLE
            .into_iter()
            .filter(|zone| !space.spec().areas_in(*zone).is_empty())
            .collect()
    }
}

impl BViewItem for BZonebar {
    fn build(&self, state: &BState) -> BView {
        if self.host != BHostType::Topbar {
            return BView::empty();
        }
        let controls = Self::declared_zones(BSpaceList::active(state))
            .into_iter()
            .map(|zone| BZoneToggle {
                zone,
                visible: visibility::is_visible(state, zone),
                action: visibility::toggle_action(zone).expect("declared toggleable zone"),
            })
            .collect();
        BView::zone_toggle_group(controls)
    }
}
