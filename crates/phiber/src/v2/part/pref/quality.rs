//! Preferences ▸ Viewport ▸ Quality (`USERPREF_PT_viewport_quality`):
//! Viewport Anti-Aliasing, the sample count a Solid or Wireframe view is
//! drawn with (`scene::viewport_aa`). Blender's panel lists Viewport
//! Anti-Aliasing first, then Smooth Wires and the Overlay / Edit Mode
//! toggles, which Phiber's wires do not have a switch for yet.

use crate::v2::host::BHostType;
use crate::v2::part::props::BPanelItem;
use crate::v2::scene::viewport_aa::BViewportAa;
use crate::v2::state::val::V_FALSE;
use crate::v2::state::{BKey as _, BState, ButsKey};
use crate::v2::view::BView;
use crate::v2::view::prim::BCard;

/// The Quality panel.
pub struct BViewportQuality;

impl BViewportQuality {
    pub const TITLE: &'static str = "Quality";

    pub fn view(state: &BState) -> BView {
        let key = BPanelItem::expand_key(BHostType::Prefs, "Viewport", Self::TITLE);
        let open = state.get(&key) != Some(V_FALSE);
        if !open {
            return BCard::panel(Self::TITLE).fold(key, false).into();
        }
        let select = BViewportAa::control(ButsKey::Dropdown("pref.viewport_aa").path(), state);
        BCard::panel(Self::TITLE).fold(key, true).body(vec![BView::ctrl(select)]).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::AppKey;

    #[test]
    fn the_card_offers_blenders_counts_over_the_preference() {
        let mut s = BState::new();
        let BView::Card(card) = BViewportQuality::view(&s) else { panic!() };
        let BView::Ctrl(ctrl) = &card.body[0] else { panic!("{:?}", card.body) };
        assert_eq!(ctrl.key, BViewportAa::key());
        assert_eq!(ctrl.chosen().map(|o| o.label.as_str()), Some("8 Samples"), "eight until set, as `U.viewport_aa`");
        s.set(AppKey::ViewportAa, "16");
        let BView::Card(card) = BViewportQuality::view(&s) else { panic!() };
        let BView::Ctrl(ctrl) = &card.body[0] else { panic!() };
        assert_eq!(ctrl.chosen().map(|o| o.key.as_str()), Some("16"));
    }
}
