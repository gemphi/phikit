//! Opening a file — the progress screen.
//!
//! Blender shows the file it is reading and how far it has got; so does this,
//! over whatever is on screen, with the same card shape as every other modal.
//! It appears while an import is in flight and goes away when the scene
//! settles, so the user never stares at an unchanged viewport wondering.

use crate::v2::scene::load_cancel_key;
use crate::v2::scene::loaded_path_key;
use crate::v2::scene::selected_key;
use crate::v2::scene::view_load_progress;
use crate::v2::scene::view_load_stage;
use crate::v2::state::BState;
use crate::v2::state::val::V_TRUE;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BColumn;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;

#[derive(Clone, Copy, Debug, Default)]
pub struct BProgressModal;

/// The file being opened, by its name alone.
fn opening(s: &BState) -> String {
    let path = s
        .get(&selected_key())
        .filter(|p| !p.is_empty())
        .or_else(|| s.get(&loaded_path_key()))
        .unwrap_or("");
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}

impl BViewItem for BProgressModal {
    fn build(&self, s: &BState) -> BView {
        let fraction = view_load_progress(s).unwrap_or(0.0);
        let stage = view_load_stage(s).unwrap_or("Opening").to_string();
        BView::from(BCard::column(BColumn::Body, vec![
                BView::row(vec![BView::icon("folder_open"), BView::label(opening(s))]),
                BView::progress_bar(fraction, stage),
                BView::row(vec![BView::Button(
                    BControl::new()
                        .with_label("Cancel")
                        .with_action(BAction::Set(load_cancel_key(), V_TRUE.to_string())),
                )]),
            ],))
    }
}
