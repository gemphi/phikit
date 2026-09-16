//! Asset Shelf view builder.
//!
//! Ref: Blender 5.2 Asset Shelf in 3D View and Node Editor.

use crate::bui::bview::{BView, CardShade};
use crate::bui::state::BState;
use crate::store::hosts::BHostType;

pub struct AssetShelfView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> AssetShelfView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let _ = (self.host, self.state);
        BView::card()
            .with_shade(CardShade::Panel)
            .with_header("Asset Shelf")
            .with_body(BView::row(
                Some(4.0),
                vec![
                    BView::item("Pose 01").with_icon("user"),
                    BView::item("Pose 02").with_icon("user"),
                    BView::item("Material Gold").with_icon("circle"),
                    BView::item("Material Glass").with_icon("circle"),
                ],
            ))
    }
}
