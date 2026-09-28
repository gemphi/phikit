//! Help modal — Blender's Help menu items as an overlay dialog.

pub mod set;

use crate::v2::modal::splash::set::link_action;
use crate::v2::state::BState;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BControl;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
pub use set::ACTIONS;
pub use set::BHelpEntry;
pub use set::WEB_LINKS;

#[derive(Clone, Copy, Debug, Default)]
pub struct BHelpModal;

impl BViewItem for BHelpModal {
    /// Blender's Help menu; its pages open in the reader rather than a browser.
    fn build(&self, _s: &BState) -> BView {
        let rows = WEB_LINKS
            .iter()
            .map(|l| {
                BView::Button(
                    BControl::new()
                        .with_icon(l.icon)
                        .with_label(l.label)
                        .with_tooltip(l.target)
                        .with_action(link_action(l.target)),
                )
            })
            .collect();
        BCard::headless().body(rows).into()
    }
}
