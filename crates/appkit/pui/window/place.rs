//! Where the areas of the space a window shows go, for the way the app is
//! being used. This is what makes one app a desktop, a mobile and a
//! spatial one: the spaces, zones, areas and hosts are the same, and only
//! this differs.

use super::PWindow;
use crate::pui::core::{PMode, PPoint, PRect, PSize, setting};

/// Where one area is shown.
#[derive(Debug, Clone, PartialEq)]
pub struct PPlace {
    /// Which zone of the space, and which area of that zone.
    pub zone: usize,
    pub area: usize,
    /// The host the area holds.
    pub host: String,
    /// Its rectangle: in the window, or for a panel in space the panel's
    /// own.
    pub rect: PRect,
    /// In space, how many degrees the panel is turned around the person,
    /// to the right of straight ahead. Zero in a flat window.
    pub turn: f32,
}

impl PWindow {
    /// Where each area of the current space is shown in a window of this
    /// size.
    ///
    /// At a desk the zones stand side by side, each as wide as its share
    /// of the whole, and a zone's areas are stacked in it. In a hand there is room for
    /// one thing: the area in front takes the whole window and the others
    /// are not shown (`bring` puts another in front). In space every area
    /// is a panel of its own, the size `data/mode.tsv` gives, and the
    /// panels stand in an arc around the person.
    pub fn place(&self, size: PSize, mode: PMode) -> Vec<PPlace> {
        let Some(space) = self.current() else {
            return Vec::new();
        };
        let held = space.zone.iter().enumerate().flat_map(|(zone, slot)| slot.area.iter().enumerate().map(move |(area, held)| (zone, area, held.host.clone())));
        let at = |zone: usize, area: usize, host: String, rect: PRect, turn: f32| PPlace { zone, area, host, rect, turn };
        match mode {
            PMode::Desktop => {
                let whole = space.zone.iter().map(|zone| zone.share).sum::<f32>().max(f32::EPSILON);
                let wide = |zone: usize| size.width * space.zone[zone].share / whole;
                held.map(|(zone, area, host)| {
                    let (left, height) = ((0..zone).map(wide).sum::<f32>(), size.height / space.zone[zone].area.len().max(1) as f32);
                    at(zone, area, host, PRect::new(left, area as f32 * height, wide(zone), height), 0.0)
                })
                .collect()
            }
            PMode::Mobile => held.skip(self.front).take(1).map(|(zone, area, host)| at(zone, area, host, PRect { origin: PPoint::default(), size }, 0.0)).collect(),
            PMode::Spatial => {
                let panel = PRect::new(0.0, 0.0, setting("spatial.width"), setting("spatial.height"));
                let count = space.zone.iter().map(|zone| zone.area.len()).sum::<usize>();
                let middle = count.saturating_sub(1) as f32 / 2.0;
                held.enumerate().map(|(index, (zone, area, host))| at(zone, area, host, panel, (index as f32 - middle) * setting("spatial.turn"))).collect()
            }
        }
    }

    /// How many areas the current space has.
    pub fn area_count(&self) -> usize {
        self.current().map_or(0, |space| space.zone.iter().map(|zone| zone.area.len()).sum())
    }

    /// On a mobile window, put the area of that number in front. `false`,
    /// and nothing changes, when the space has no such area.
    pub fn bring(&mut self, index: usize) -> bool {
        let known = index < self.area_count();
        if known {
            self.front = index;
        }
        known
    }

    /// The number of the first area that holds the host of that name.
    pub fn index(&self, host: &str) -> Option<usize> {
        let held = self.current()?.zone.iter().flat_map(|zone| &zone.area);
        held.enumerate().find(|(_, area)| area.host == host).map(|(index, _)| index)
    }

    /// The number of the area in front on a mobile window.
    pub fn front(&self) -> usize {
        self.front
    }
}
