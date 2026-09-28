//! Canonical visibility state for toggleable space zones.

use super::BZoneType;
use crate::v2::state::{BKey as _, BPath, BState, ZoneKey, val::V_FALSE};
use crate::v2::view::BAction;

pub const TOGGLEABLE: [BZoneType; 3] = [BZoneType::Left, BZoneType::Right, BZoneType::Bottom];

pub fn key(zone: BZoneType) -> Option<BPath> {
    TOGGLEABLE.contains(&zone).then(|| ZoneKey::Shown(zone).path())
}

pub fn is_visible(state: &BState, zone: BZoneType) -> bool {
    key(zone).is_none_or(|path| state.get(&path) != Some(V_FALSE))
}

pub fn toggle_action(zone: BZoneType) -> Option<BAction> {
    key(zone).map(BAction::Toggle)
}
