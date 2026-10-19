//! Canonical visibility state for toggleable space zones.

use super::BZoneType;
use crate::v2::state::{BKey as _, BPath, BState, ZoneKey, val::V_FALSE};
use crate::v2::view::BAction;

/// The zones a space can put away, in the order the topbar lists them.
///
/// A space is divided left / centre / right across and top / centre / bottom
/// down, so a zone sits in one of five places. Three of them can be put
/// away: Left, Right and Bottom. The other two cannot, for the same reason
/// in two forms — **put away, they take the way back with them.**
///
/// Centre is the content. A space with its centre gone has nothing left to
/// show.
///
/// Top is the Topbar, and the Topbar is where the zone toggles live. Switch
/// it off and the control that would switch it on goes with it: the window
/// opens on its editor headers with no menus, no workspace tabs and no way
/// to get them back short of editing saved state by hand. Blender has the
/// same rule and arrives at it the same way — `screen_opengl_render` can
/// shrink the topbar to nothing, and there is still no operator to remove
/// it. An edge is only safely hideable when the thing that restores it is
/// somewhere else.
///
/// `BZoneGlyph` still draws Top and Centre their own pictures, because a
/// glyph that cannot say "top" cannot say which of the five it means.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::ZoneKey;
    use crate::v2::state::val::V_FALSE;

    #[test]
    fn the_topbar_cannot_be_put_away_even_by_state_that_says_it_was() {
        // A session saved while Top was toggleable carries
        // `ctx.zone.top: "false"`. It must not keep the Topbar hidden —
        // there would be no way to bring it back.
        let mut s = BState::new();
        s.set(ZoneKey::Shown(BZoneType::Top), V_FALSE);
        assert!(is_visible(&s, BZoneType::Top), "the Topbar is always up");
        assert!(key(BZoneType::Top).is_none(), "and has no toggle to be off");
        assert!(toggle_action(BZoneType::Top).is_none());
    }

    #[test]
    fn the_three_edges_that_keep_their_way_back_still_toggle() {
        let mut s = BState::new();
        for zone in [BZoneType::Left, BZoneType::Right, BZoneType::Bottom] {
            assert!(is_visible(&s, zone), "{zone:?} starts up");
            s.set(ZoneKey::Shown(zone), V_FALSE);
            assert!(!is_visible(&s, zone), "{zone:?} puts away");
            assert!(toggle_action(zone).is_some(), "{zone:?} can be brought back from the Topbar");
        }
        assert!(is_visible(&s, BZoneType::Center), "the centre is the content");
    }
}
