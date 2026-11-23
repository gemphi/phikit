//! A zone: a grid slot inside a space. `PZone`.

use super::{PEntry, PLevel, PArea};

/// A grid slot inside a space.
#[derive(Debug, Clone, PartialEq)]
pub struct PZone {
    pub entry: PEntry,
    /// Its areas, each holding one host.
    pub area: Vec<PArea>,
    /// How much of the window's width it takes at a desk, against the
    /// other zones: a zone of 3 beside a zone of 1 is three times as wide.
    pub share: f32,
}

impl PLevel for PZone {
    const WORD: &'static str = "zone";

    fn of(entry: PEntry) -> PZone {
        PZone { entry, area: Vec::new(), share: 1.0 }
    }

    fn entry(&self) -> &PEntry {
        &self.entry
    }
}
