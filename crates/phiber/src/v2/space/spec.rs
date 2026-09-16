//! `BSpaceSpec` — one space with its area set inline.

use super::BSpaceType;
use crate::v2::area::BAreaSpec;
use crate::v2::spec::{BSpec, BSpecKind};
use crate::v2::zone::BZoneType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSpaceSpec {
    pub kind: BSpaceType,
    pub label: &'static str,
    pub icon: &'static str,
    pub areas: &'static [BAreaSpec],
}

impl BSpec for BSpaceSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Space(self.kind)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

impl BSpaceSpec {
    /// Areas whose slot sits inside the given zone.
    pub fn areas_in(&self, zone: BZoneType) -> &[BAreaSpec] {
        // Slices contiguously by zone; areas are declared zone-grouped in set.rs.
        // (Not sorted here — the layout file authors keep zone-groups adjacent.)
        // Callers filter as needed; this returns a filtered leak-free view via
        // a small helper below.
        static EMPTY: &[BAreaSpec] = &[];
        // Compute inline: find first + last index with matching zone.
        let mut start = None;
        let mut end = 0usize;
        for (i, a) in self.areas.iter().enumerate() {
            if a.zone == zone {
                if start.is_none() {
                    start = Some(i);
                }
                end = i + 1;
            }
        }
        match start {
            Some(s) => &self.areas[s..end],
            None => EMPTY,
        }
    }
}
