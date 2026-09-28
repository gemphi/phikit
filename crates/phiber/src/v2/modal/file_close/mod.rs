//! "Save changes before closing?" — Blender's `wm_close_file_dialog`
//! (`block_create__close_file_dialog`): the question, the file's name
//! (`Untitled.blend` when it has none), and Save / Don't Save / Cancel. It
//! comes up when the document has unsaved changes and the user quits,
//! makes a new file or opens another (`wm_operator_close_file_dialog_if_
//! needed`); what happens after the answer is `ctx.modal.file_close.then`,
//! kept by the app (`app/op/close.rs`). Content is `set.rs`.

pub mod set;

use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
pub use set::{BCloseAnswer, TITLE};

#[derive(Clone, Copy, Debug, Default)]
pub struct BFileCloseModal;

impl BViewItem for BFileCloseModal {
    fn build(&self, s: &BState) -> BView {
        BView::column(vec![
            BView::row(vec![BView::icon("file_blend"), BView::label(set::file_name(s))]),
            BView::row(set::buttons().into_iter().map(|answer| answer.button()).collect()),
        ])
    }
}
