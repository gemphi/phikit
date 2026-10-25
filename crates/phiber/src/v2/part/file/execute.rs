//! Execute region — the filename field plus the Cancel / <Execute> pair.
//! Blender ref: `file_panels.cc:138` (`file_panel_execution_buttons_draw`).
//!
//! The execute button's label comes from the browser's purpose
//! (`FileSelectParams::title`) rather than being hardcoded to "Open", and the
//! button order follows the platform: Windows puts the affirmative action
//! first, everywhere else it comes last.

use crate::v2::state::{BKey as _, ModalKey};
use crate::v2::state::FileKey;
use super::listing;
use super::purpose::purpose;
use crate::v2::state::{BState, val::V_EMPTY};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

fn filename(s: &BState) -> BView {
    let name = s
        .get(&FileKey::Filename.path())
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .or_else(|| {
            Some(super::op::picked(s)).filter(|p| !p.is_empty())
                .map(|p| p.rsplit(['/', '\\']).next().unwrap_or(p).to_string())
        })
        .unwrap_or_default();
    // Typed, not a read-out: the user names the file here, and Return
    // commits it the same way the directory and search fields do.
    let key = FileKey::Filename.path();
    let editing = crate::v2::state::edit::is_editing(s, &key);
    BView::text_field("File Name", name, editing, BAction::Set(key, String::new()))
}

/// Commit: resolve `<dir>/<filename>` and write it as the selection, which
/// the action reducer turns into an import + recent-list entry.
fn execute_btn(s: &BState) -> BView {
    let dir = listing::current_dir(s);
    let target = Some(super::op::picked(s))
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            let name = s.get(&FileKey::Filename.path()).unwrap_or("");
            if name.is_empty() {
                String::new()
            } else {
                listing::join(&dir, name)
            }
        });
    // Blender's execute button is the dialog's one lit button.
    BView::Button(
        BControl::new()
            .with_label(purpose(s).exec_label())
            .with_active(true)
            .with_action(BAction::Set(FileKey::Selected.path(), target)),
    )
}

/// Where the execute region is hosted: the modal dialog (Cancel closes the
/// dialog) or an editor area (Cancel drops the purpose so the bar hides —
/// Blender's `file.cancel` in an area only clears `FileSelectParams`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BExecuteHost {
    Modal,
    Area,
}

fn cancel_btn(host: BExecuteHost) -> BView {
    let action = match host {
        BExecuteHost::Modal => BAction::Set(ModalKey::Open.path(), V_EMPTY.to_string()),
        BExecuteHost::Area => BAction::Reset(FileKey::Purpose.path()),
    };
    BView::Button(BControl::new().with_label("Cancel").with_action(action))
}

/// The execute bar for the browser wherever it is: in the dialog it is
/// always up and Cancel closes the dialog; in an editor area it is up only
/// while the browser has a purpose (Open, Save As, Import, Export —
/// Blender's `FILEBROWSER_PT_execute`) and Cancel drops that purpose.
pub fn view(s: &BState) -> BView {
    if in_dialog(s) {
        return view_in(s, BExecuteHost::Modal);
    }
    if has_purpose(s) {
        return view_in(s, BExecuteHost::Area);
    }
    BView::empty()
}

/// Whether the browser on screen is the modal dialog.
pub fn in_dialog(s: &BState) -> bool {
    s.get(ModalKey::Open) == Some(crate::v2::state::val::V_MODAL_FILE_BROWSER)
}

/// Whether the browser is in a named-file mode.
pub fn has_purpose(s: &BState) -> bool {
    use super::purpose::BFilePurpose;
    s.get(&FileKey::Purpose.path()).is_some()
        && matches!(
            purpose(s),
            BFilePurpose::Open | BFilePurpose::SaveAs | BFilePurpose::SaveCopy | BFilePurpose::Import | BFilePurpose::Export | BFilePurpose::SaveImage | BFilePurpose::Mixdown | BFilePurpose::OpenImage
        )
}

/// Whether the purpose names a new file (Save As, Export…), so the bar
/// carries a name field; opening only needs the pick.
fn names_a_file(s: &BState) -> bool {
    use super::purpose::BFilePurpose;
    matches!(purpose(s), BFilePurpose::SaveAs | BFilePurpose::SaveCopy | BFilePurpose::Export | BFilePurpose::SaveImage | BFilePurpose::Mixdown)
}

/// The bar in three groups, as every strip is: on the left what will be
/// acted on — the name being typed when the purpose names a file, else the
/// file picked; in the middle what the dialog does; on the right Cancel and
/// the action. The buttons hug their words rather than the bar's width, and
/// a path to open is typed into the directory field or found with the search
/// at the top, not into a field the width of the window here.
pub fn view_in(s: &BState, host: BExecuteHost) -> BView {
    let picked = Some(super::op::picked(s)).filter(|p| !p.is_empty()).map(|p| p.rsplit(['/', '\\']).next().unwrap_or(p).to_string());
    let lead = if names_a_file(s) {
        filename(s)
    } else {
        BView::pair("Selected", picked.unwrap_or_else(|| "nothing yet".to_string()))
    };
    let mut list = vec![lead, BView::spacer(), BView::label(purpose(s).title().to_string()), BView::spacer()];
    if cfg!(windows) {
        list.push(execute_btn(s));
        list.push(cancel_btn(host));
    } else {
        list.push(cancel_btn(host));
        list.push(execute_btn(s));
    }
    BView::row(list)
}
