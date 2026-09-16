//! Canonical visibility state for toggleable space zones.

use super::BZoneType;
use crate::v2::state::{
    BPath, BState,
    key::{K_ZONE_BOTTOM, K_ZONE_LEFT, K_ZONE_RIGHT},
    val::V_FALSE,
};
use crate::v2::view::BAction;

pub const TOGGLEABLE: [BZoneType; 3] = [BZoneType::Left, BZoneType::Right, BZoneType::Bottom];

pub fn key(zone: BZoneType) -> Option<BPath> {
    match zone {
        BZoneType::Left => Some(K_ZONE_LEFT),
        BZoneType::Right => Some(K_ZONE_RIGHT),
        BZoneType::Bottom => Some(K_ZONE_BOTTOM),
        _ => None,
    }
}

pub fn is_visible(state: &BState, zone: BZoneType) -> bool {
    key(zone).is_none_or(|path| state.get(&path) != Some(V_FALSE))
}

pub fn toggle_action(zone: BZoneType) -> Option<BAction> {
    key(zone).map(BAction::Toggle)
}
