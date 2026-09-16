//! Splash / Welcome window — the Phiber welcome screen.
//!
//! Blender ref: `WM_OT_splash`. Three regions:
//!   • left  — "New File" workspace templates, then Open… / Recover
//!   • right — "Recent Projects", newest first
//!   • foot  — documentation and community links
//!
//! Opened by `Blender ▸ Splash Screen`, and shown once at startup (the seed
//! writes `ctx.modal = "splash"`). Dismissed like any modal: backdrop click,
//! the title-bar close button, or picking anything that commits a `Set`.

pub mod recent;
pub mod set;

use crate::v2::part::custom::file::keys as file_keys;
use crate::v2::state::{
    BPath, BState,
    key::{K_MODAL, K_SPACE},
    val::{V_EMPTY, V_MODAL_FILE_BROWSER},
};
use crate::v2::view::prim::{BAction, BPanelRole, BRowShade};
use crate::v2::view::{BView, BViewItem};

pub use set::{BSplashEntry, EMPTY_RECENT, LINKS, RECENT_MAX, TEMPLATES};

#[derive(Clone, Copy, Debug, Default)]
pub struct BSplashModal;

fn heading(text: &'static str) -> BView {
    BView::panel(BPanelRole::CardHeader, vec![BView::label(text)])
}

/// "New File" column — one row per workspace template.
fn templates() -> BView {
    let mut rows = vec![heading("New File")];
    for t in TEMPLATES {
        rows.push(BView::button_action(
            t.icon,
            t.label,
            BAction::Set(K_SPACE, t.target.to_string()),
        ));
    }
    rows.push(BView::menu_sep());
    rows.push(BView::button_action(
        "folder",
        "Open\u{2026}",
        BAction::Set(K_MODAL, V_MODAL_FILE_BROWSER.to_string()),
    ));
    BView::panel(BPanelRole::Card, rows)
}

/// "Recent Projects" column. Each row loads that path; the reducer records
/// the visit and dismisses the welcome window.
fn recents(s: &BState) -> BView {
    let paths = recent::recent_paths(s);
    let mut rows = vec![heading("Recent Projects")];
    if paths.is_empty() {
        rows.push(BView::list_row(
            vec![BView::label(EMPTY_RECENT)],
            BRowShade::Flat,
        ));
    } else {
        for (i, path) in paths.iter().enumerate() {
            let shade = if i % 2 == 1 {
                BRowShade::Alt
            } else {
                BRowShade::Flat
            };
            rows.push(BView::list_row(
                vec![
                    BView::button_action(
                        "file",
                        recent::display_name(path).to_string(),
                        BAction::Set(file_keys::k_selected(), path.clone()),
                    ),
                    BView::label(recent::parent_dir(path).to_string()),
                ],
                shade,
            ));
        }
    }
    BView::panel(BPanelRole::Card, rows)
}

/// Bottom link row.
fn links() -> BView {
    let mut row = Vec::with_capacity(LINKS.len());
    for l in LINKS {
        row.push(BView::button_action(
            l.icon,
            l.label,
            BAction::Set(BPath::owned(vec!["link"]), l.target.to_string()),
        ));
    }
    BView::panel(BPanelRole::Footer, row)
}

fn brand_header() -> BView {
    BView::panel(
        BPanelRole::Header,
        vec![BView::icon("phiber"), BView::label("Phiber")],
    )
}

fn supporting_copy() -> BView {
    BView::label("Create something new or continue from a recent project.")
}

impl BViewItem for BSplashModal {
    fn build(&self, s: &BState) -> BView {
        BView::column(vec![
            brand_header(),
            supporting_copy(),
            BView::row(vec![templates(), recents(s)]),
            links(),
        ])
    }
}

/// Action that dismisses the welcome window without changing anything else.
pub fn dismiss() -> BAction {
    BAction::Set(K_MODAL, V_EMPTY.to_string())
}
