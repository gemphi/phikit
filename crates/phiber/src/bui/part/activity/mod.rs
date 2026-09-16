//! Activity Bar part — extreme left full-height global navigation bar with social drawer.

pub mod brand;
pub mod drawer;
pub mod primary;

pub use brand::ActivityBrand;
pub use drawer::ActivityDrawer;
pub use primary::ActivityPrimary;

use super::super::bview::{BView, CardShade};
use super::super::state::{BKey, BState};

pub struct ActivityView<'a> {
    pub state: &'a BState,
}

impl<'a> ActivityView<'a> {
    pub fn new(state: &'a BState) -> Self {
        Self { state }
    }

    pub fn build(self) -> BView {
        let active = self.state.get(&BKey::ActivityTab).unwrap_or("home");
        let drawer_open = self.state.get(&BKey::ActivityDrawer) == Some("true");
        let expanded = self.state.get(&BKey::ActivityExpanded) == Some("true");
        let active_space = self.state.get(&BKey::CtxSpace).unwrap_or("Layout");

        let brand_btn = ActivityBrand::build(expanded);
        let top_section = ActivityPrimary::top_section(active_space, active, drawer_open);
        let center_section = ActivityPrimary::center_section(active);
        let bottom_section = ActivityPrimary::bottom_section();

        let width = if expanded { 140.0 } else { 44.0 };

        let primary_card = BView::card()
            .with_shade(CardShade::Activity)
            .with_width(Some(width))
            .with_body(BView::column_center(
                Some(6.0),
                vec![brand_btn, top_section, BView::empty(), center_section],
            ))
            .with_footer_action(bottom_section);

        if drawer_open {
            let secondary_drawer = ActivityDrawer::build(self.state, active);
            BView::row(None, vec![primary_card, secondary_drawer])
        } else {
            primary_card
        }
    }
}
