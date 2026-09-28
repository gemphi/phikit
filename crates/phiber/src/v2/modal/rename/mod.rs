//! Rename Active Item — Blender's F2 popup (`TOPBAR_PT_name`,
//! `wm.call_panel`): one text field holding the active object's name, the
//! caret already in it; Return renames, Escape leaves the name alone. The
//! same popup renames whatever is active where the pointer is in Blender;
//! here it is the active object.
//!
//! Opening is `ctx.op.rename` (empty) → `BRenameOpenRule`; the field is
//! `ctx.rename.text`, and committing it (Return, or the OK button) runs
//! `BRenameRule`, which renames through `scene::edit::BSceneEdit::rename`
//! and carries the selection, the active object and the row's toggles to
//! the new name. Content is `set.rs`.

pub mod set;

use crate::v2::state::{BKey as _, AppKey, BState, CtxKey};
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::prim::{BAction, BControl};
pub use set::{BRenameOpenRule, BRenameRule, MARKER_TITLE, TITLE};

#[derive(Clone, Copy, Debug, Default)]
pub struct BRenameModal;

impl BViewItem for BRenameModal {
    fn build(&self, s: &BState) -> BView {
        let text = s.get(AppKey::RenameText).unwrap_or("").to_string();
        let marker = s.get(AppKey::RenameMarker).is_some();
        let subject = if marker { format!("Frame {}", s.get(AppKey::RenameMarker).unwrap_or("")) } else { s.get(CtxKey::Object).unwrap_or("").to_string() };
        let field = BView::text_field(
            "Name",
            text.clone(),
            crate::v2::state::edit::is_editing(s, AppKey::RenameText),
            BAction::Set(AppKey::RenameText.path(), String::new()),
        );
        let ok = BView::Button(BControl::new().with_label("OK").with_action(BAction::Set(AppKey::RenameText.path(), text)));
        BView::column(vec![
            BView::row(vec![BView::icon(if marker { "marker" } else { "mesh" }), BView::label(if subject.is_empty() { "No active object".to_string() } else { subject })]),
            BView::row(vec![field, ok]),
        ])
    }
}
