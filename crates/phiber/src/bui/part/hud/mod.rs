//! HUD Overlay view builder.
//!
//! Ref: Blender 5.2 Floating HUD & Gizmo status info.

use crate::bui::bview::{BView, CardShade};
use crate::bui::state::BState;
use crate::store::hosts::BHostType;

pub struct HudView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> HudView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let _ = (self.host, self.state);
        BView::card()
            .with_shade(CardShade::Activity)
            .with_body(BView::label(
                "Transformation: dx: 0.00, dy: 0.00, dz: 0.00 (Global)",
            ))
    }
}
