//! Brand header component for Activity Bar.

use super::super::super::bview::{BAction, BView};
use super::super::super::state::BKey;

pub struct ActivityBrand;

impl ActivityBrand {
    pub fn build(expanded: bool) -> BView {
        let brand_label = if expanded { "PHIBER" } else { "P" };
        let mut brand_btn = BView::button()
            .with_icon("zap")
            .with_tooltip(if expanded {
                "Collapse Bar"
            } else {
                "Phiber 3D Studio"
            })
            .with_action(BAction::Toggle(BKey::ActivityExpanded));

        if expanded {
            brand_btn = brand_btn.with_label(brand_label);
        }

        brand_btn
    }
}
