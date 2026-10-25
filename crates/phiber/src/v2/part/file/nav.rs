//! The file browser's folder navigation — Blender's `FILEBROWSER_PT_directory_path`
//! nav buttons (`file.previous`, `file.next`, `file.parent`, `file.refresh`),
//! split so the strip can lay them out as its left column: the three folder
//! moves at the column's start (`view`), the refresh at its end (`refresh`),
//! under which the source list opens (`host/file`).

use super::history;
use super::listing;
use super::op::{NAV_NEW_FOLDER, NAV_NEXT, NAV_PARENT, NAV_PREVIOUS, NAV_REFRESH};
use super::set::{BFileNavSpec, NAV};
use crate::v2::state::BKey as _;
use crate::v2::state::{BState, FileKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

fn button(s: &BState, spec: &BFileNavSpec) -> BView {
    let enabled = match spec.action {
        NAV_PREVIOUS => history::can_back(s),
        NAV_NEXT => history::can_forward(s),
        NAV_PARENT => listing::parent_of(&listing::current_dir(s)).is_some(),
        _ => true,
    };
    let control = BControl::new().with_icon(spec.icon).with_tooltip(spec.tooltip);
    BView::Button(if enabled {
        control.with_action(BAction::Set(FileKey::Nav.path(), spec.action.to_string()))
    } else {
        control
    })
}

/// Previous, next and parent — the start of the strip's left column.
/// Refresh and the new folder close it (`BFileRefresh`).
pub struct BFileNav;

impl BFileNav {
    pub const fn new(_host: crate::v2::host::BHostType) -> Self {
        Self
    }
}

impl crate::v2::view::BViewItem for BFileNav {
    fn build(&self, s: &BState) -> BView {
        BView::row(
            NAV.iter().filter(|spec| ![NAV_REFRESH, NAV_NEW_FOLDER].contains(&spec.action)).map(|spec| button(s, spec)).collect(),
        )
    }
}

/// Refresh and New Folder — the end of the strip's left column, flush with
/// the source list's edge under it.
pub struct BFileRefresh;

impl BFileRefresh {
    pub const fn new(_host: crate::v2::host::BHostType) -> Self {
        Self
    }
}

impl crate::v2::view::BViewItem for BFileRefresh {
    fn build(&self, s: &BState) -> BView {
        // Refresh, then Blender's new-folder button beside it.
        BView::row(NAV.iter().filter(|spec| [NAV_REFRESH, NAV_NEW_FOLDER].contains(&spec.action)).map(|spec| button(s, spec)).collect())
    }
}
