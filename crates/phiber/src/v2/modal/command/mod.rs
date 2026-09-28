//! Command Palette — searchable operator/menu launcher.
//! Blender ref: F3 "Menu Search" and "Operator Search" (Developer Extras).

pub mod palette;
pub mod set;

use crate::v2::state::BKey as _;
use crate::v2::state::AppKey;
use crate::v2::state::BState;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BColumn;
use crate::v2::view::prim::BAction;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
pub use set::BCommandField;
pub use set::BCommandKind;
pub use set::SOURCES;

#[derive(Clone, Copy, Debug, Default)]
pub struct BCommandModal;

impl BViewItem for BCommandModal {
    fn build(&self, s: &BState) -> BView {
        let query = s.get(AppKey::CommandQuery).unwrap_or("").to_string();
        let search = BView::text_field(
            "Search menus…",
            query.clone(),
            crate::v2::state::edit::is_editing(s, AppKey::CommandQuery),
            BAction::Set(AppKey::CommandQuery.path(), String::new()),
        );
        BView::column(vec![search, BView::from(BCard::column(BColumn::Body, palette::rows(&query)))])
    }
}
