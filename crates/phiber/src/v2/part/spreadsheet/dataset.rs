//! The dataset region — Blender's `spreadsheet_dataset_draw.cc`: the active
//! object's geometry as a tree, `Mesh` over one row per domain, each row
//! naming the domain and how many elements it has, the chosen one active.
//! Clicking a row is what changes the table beside it
//! (`SpaceSpreadsheet.attribute_domain`).
//!
//! Blender puts this in the Spreadsheet's left region, which is why it is a
//! part of its own rather than the generic Toolbar: the Spreadsheet has no
//! tools.

use super::set::{BSpreadsheetDomain, active_object};
use crate::v2::host::BHostType;
use crate::v2::state::{BPath, BState};
use crate::v2::view::prim::{BAction, BCard, BColumn, BControl};
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BDataset;

impl BDataset {
    pub fn new() -> Self {
        Self
    }

    /// The host key the chosen domain lives at.
    fn key() -> BPath {
        BPath::owned(vec![BHostType::Spreadsheet.slug(), "domain"])
    }
}

impl BViewItem for BDataset {
    fn build(&self, s: &BState) -> BView {
        let Some(object) = active_object(s) else {
            // No active object: Blender's dataset region is empty.
            return BView::from(BCard::column(BColumn::Body, Vec::new()));
        };
        let active = BSpreadsheetDomain::now(s);
        let mut list = vec![BView::label(format!("{}  ·  Mesh", object.name))];
        list.extend(BSpreadsheetDomain::OFFERED.iter().copied().map(|domain| {
            BView::Button(
                BControl::new()
                    .with_label(format!("{}  {}", domain.label(), domain.count(&object.mesh)))
                    .with_active(domain == active)
                    .with_action(BAction::Set(Self::key(), domain.slug().to_string())),
            )
        }));
        BView::from(BCard::column(BColumn::Body, list))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BDatasetList;

impl BDatasetList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BDatasetList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BDataset::new().build(s)]
    }
}
