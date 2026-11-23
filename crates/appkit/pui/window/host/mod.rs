//! A host: one kind of editor. It declares its regions. `PHost`. The hosts
//! every app has live here: a file browser, preferences.

mod lay;

pub use lay::PSlot;

use super::{PEntry, PLevel, PPane, PRegion, PSet, PSide};

/// One kind of editor. It declares its regions and nothing of what is in them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PHost {
    pub entry: PEntry,
    /// Its regions: a header, a centre, a footer.
    pub region: Vec<PRegion>,
}

impl PLevel for PHost {
    const WORD: &'static str = "host";

    fn of(entry: PEntry) -> PHost {
        PHost { entry, region: Vec::new() }
    }

    fn entry(&self) -> &PEntry {
        &self.entry
    }
}

impl PHost {
    /// Declare the regions of the hosts of a set from a table: one part a
    /// line, as the host, the region, the side of its pane and the part,
    /// between tabs. A region and a pane are made the first time they are
    /// named. A line that is empty, starts with `#`, or names a host the
    /// set does not have or a side that is none is skipped; the answer is
    /// how many parts were put in.
    pub fn declare(set: &mut PSet<PHost>, text: &str) -> usize {
        let mut put = 0;
        for line in text.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')) {
            let mut cell = line.split('\t').map(str::trim);
            let (host, region, side, part) = (cell.next().unwrap_or_default(), cell.next().unwrap_or_default(), cell.next().unwrap_or_default(), cell.next().unwrap_or_default());
            let (Some(host), Some(side)) = (set.find_mut(host), PSide::named(side)) else { continue };
            let known = host.region.iter().position(|slice| slice.name == region);
            let index = known.unwrap_or_else(|| {
                host.region.push(PRegion::new(region, Vec::new()));
                host.region.len() - 1
            });
            let slice = &mut host.region[index];
            let known = slice.pane.iter().position(|pane| pane.side == side);
            let index = known.unwrap_or_else(|| {
                slice.pane.push(PPane::new(side, &[]));
                slice.pane.len() - 1
            });
            slice.pane[index].part.push(part.to_string());
            put += 1;
        }
        put
    }
}
