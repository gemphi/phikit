//! Command Palette — searchable operator/menu launcher.
//! Blender ref: F3 "Menu Search" and "Operator Search" (Developer Extras).

pub mod find;
pub mod layout;
pub mod palette;
pub mod set;

use crate::v2::state::BKey as _;
use crate::v2::state::AppKey;
use crate::v2::state::BState;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BColumn;
use crate::v2::view::prim::BStrip;
use crate::v2::view::prim::{BAction, BControl};
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
pub use set::BCommandField;
pub use set::BCommandKind;
pub use set::SOURCES;

#[derive(Clone, Copy, Debug, Default)]
pub struct BCommandModal;

impl BViewItem for BCommandModal {
    /// The field, the tabs that narrow what it searches, and the results —
    /// every place the window can take you (`find`).
    fn build(&self, s: &BState) -> BView {
        let query = s.get(AppKey::CommandQuery).unwrap_or("").to_string();
        let search = BView::filter_field(
            "Search spaces, editors, panels and menus",
            query.clone(),
            crate::v2::state::edit::is_editing(s, AppKey::CommandQuery),
            BAction::Set(AppKey::CommandQuery.path(), String::new()),
        );
        let tab = find::BFindKind::of(s);
        let tabs = find::BFindKind::ALL
            .into_iter()
            .map(|kind| {
                BView::Button(
                    BControl::new()
                        .with_label(kind.label())
                        .with_active(kind == tab)
                        .with_action(BAction::Set(AppKey::CommandTab.path(), kind.slug().to_string())),
                )
            })
            .collect::<Vec<_>>();
        // The tabs on their one ground, centred under the field.
        BView::column(vec![
            // The field takes the slack; the layouts sit at its end.
            BView::from(BCard::strip(BStrip::Toolbar, vec![BView::row([vec![search, BView::spacer()], layout::BFindLayout::toggles(s)].concat())])),
            BView::from(BCard::strip(BStrip::Tabs, tabs)),
            BView::from(BCard::column(BColumn::Body, palette::rows(&query, tab, palette::host(s), layout::BFindLayout::of(s)))),
        ])
    }
}
