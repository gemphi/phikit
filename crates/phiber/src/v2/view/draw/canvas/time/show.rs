//! Per-host view toggles for the animation editors — `SpaceAction.flag`'s
//! view bits (`DNA_action_types.h`): `SACTION_SHOW_MARKERS` and friends.
//!
//! Blender keeps these per `SpaceAction`/`SpaceGraph`/`SpaceNla`; phiber's
//! equivalent is per host, same shape as `BToolbarShown` / `BSidebarShown`,
//! so a Timeline and a Dope Sheet open side by side each remember their own.
//! Each flag is "on by default": the key is read as `!= V_FALSE`, matching
//! Blender's default where the Timeline's `show_markers` ships `true`.

use crate::v2::host::BHostType;
use crate::v2::state::{BPath, BState, val::V_FALSE};

/// `SpaceAction.flag & SACTION_SHOW_MARKERS` (`DNA_action_types.h`): the
/// Timeline's marker band, dashed stalks and glyphs are drawn when on. Phiber
/// reads this in `BTimeView::now`: when off, the marker list stays empty and
/// `paint_markers` short-circuits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BShowMarkers(pub BHostType);

impl BShowMarkers {
    pub fn key(self) -> BPath {
        BPath::owned(vec![self.0.slug(), "show", "markers"])
    }

    pub fn key_for(host: BHostType) -> BPath {
        Self(host).key()
    }

    /// On by default — Blender ships `show_markers` as `true` on new editors.
    pub fn on(self, s: &BState) -> bool {
        s.get(&self.key()) != Some(V_FALSE)
    }
}
