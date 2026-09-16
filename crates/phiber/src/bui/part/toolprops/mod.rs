//! Tool Properties / Adjust Last Operation view builder.
//!
//! Ref: Blender's F9 Adjust Last Operation & Tool Settings panel.

use crate::bui::bview::{BView, CardShade};
use crate::bui::state::BState;
use crate::store::hosts::BHostType;

pub struct ToolPropsView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> ToolPropsView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let _ = (self.host, self.state);
        BView::card()
            .with_shade(CardShade::Panel)
            .with_header("Adjust Last Operation")
            .with_body(BView::list().with_items(vec![
                BView::prop_row("Vertices", "32"),
                BView::prop_row("Radius", "1.0 m"),
                BView::prop_row("Depth", "2.0 m"),
                BView::prop_row("Cap Fill Type", "NGON"),
                BView::prop_row("Generate UVs", "true"),
            ]))
    }
}
