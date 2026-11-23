//! A space: a workspace tab, one arrangement of the window. `PSpace`.
//! Phiber's `BSpaceType`: Layout, Modeling, Shading are spaces.

use super::{PArea, PEntry, PLevel, PSet, PZone};

/// A workspace tab: one arrangement of the window.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PSpace {
    pub entry: PEntry,
    /// Its zones, the slots of its grid.
    pub zone: Vec<PZone>,
}

impl PLevel for PSpace {
    const WORD: &'static str = "space";

    fn of(entry: PEntry) -> PSpace {
        PSpace { entry, zone: Vec::new() }
    }

    fn entry(&self) -> &PEntry {
        &self.entry
    }
}

impl PSpace {
    /// Lay the spaces of a set out from a table: one area a line, as the
    /// space, the zone and the host it holds, between tabs, and after
    /// them, if wanted, the zone's share of the width. A zone is made the
    /// first time it is named. A line that is empty, starts with `#` or
    /// names a space the set does not have is skipped; the answer is how
    /// many areas were placed.
    pub fn arrange(set: &mut PSet<PSpace>, text: &str) -> usize {
        let mut placed = 0;
        for line in text.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')) {
            let mut cell = line.split('\t').map(str::trim);
            let (space, zone, host) = (cell.next().unwrap_or_default(), cell.next().unwrap_or_default(), cell.next().unwrap_or_default());
            let Some(space) = set.find_mut(space) else { continue };
            let known = space.zone.iter().position(|slot| slot.slug() == zone);
            let index = known.unwrap_or_else(|| {
                space.zone.push(PZone::of(PEntry::new(zone, zone, "")));
                space.zone.len() - 1
            });
            let share = cell.next().and_then(|share| share.parse().ok());
            share.into_iter().for_each(|share| space.zone[index].share = share);
            space.zone[index].area.push(PArea::holding(host));
            placed += 1;
        }
        placed
    }
}
