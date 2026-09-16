//! `BStatusbarList` — iterates `set::SEGMENTS` and delegates each segment's
//! rendering (including runtime value resolution) to `BStatusbarItem`.
//!
//! While a blend import is running, a leading progress bar + stage label is
//! prepended (Blender's `WM_progress` in the status bar), then a flexible
//! spacer pushes the ordinary segments to the far end.

use super::{BStatusbarItem, set::SEGMENTS};
use crate::v2::scene::{view_load_progress, view_load_stage};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BStatusbarList;

impl BStatusbarList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BStatusbarList {
    fn items(&self, s: &BState) -> Vec<BView> {
        let mut out: Vec<BView> = Vec::new();

        // Import-progress readout, when an import is in flight. The progress
        // fraction gate is what decides visibility: absent => no import.
        if let Some(fraction) = view_load_progress(s) {
            let stage = view_load_stage(s).unwrap_or("Loading");
            out.push(BView::progress_bar(fraction, stage));
            out.push(BView::spacer());
        }

        out.extend(
            SEGMENTS
                .iter()
                .map(|spec| BStatusbarItem::new(*spec).build(s)),
        );
        out
    }
}
