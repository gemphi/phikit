//! The dialog's content: Blender's `wm_revert_mainfile_invoke` —
//! `WM_operator_confirm_ex` with the title, the warning and a Revert button,
//! beside Cancel in the platform's order (the same order as
//! `file_close::buttons`).

use crate::v2::state::{BKey as _, ModalKey, OpKey, val::V_EMPTY};
use crate::v2::view::prim::{BAction, BControl};
use crate::v2::view::BView;

pub const TITLE: &str = "Revert to the Saved File";
pub const MESSAGE: &str = "Any unsaved changes will be lost.";
/// `ctx.op.revert`'s value once the dialog is answered.
pub const CONFIRMED: &str = "confirm";

/// Revert, and Cancel — Revert first on Windows, last elsewhere.
pub fn buttons() -> [BView; 2] {
    let revert = BView::Button(BControl::new().with_label("Revert").with_active(true).with_action(BAction::Set(OpKey::Revert.path(), CONFIRMED.to_string())));
    let cancel = BView::Button(BControl::new().with_label("Cancel").with_action(BAction::Set(ModalKey::Open.path(), V_EMPTY.to_string())));
    if cfg!(windows) { [revert, cancel] } else { [cancel, revert] }
}
