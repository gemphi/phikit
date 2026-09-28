//! Splash / Welcome window — the Phiber welcome screen.
//!
//! Blender ref: `WM_OT_splash` + `WM_MT_splash` (`bl_operators/wm.py:3431`).
//! Every block is one rounded `Card`, laid out exactly like Blender's splash:
//!
//! ```text
//! ┌─ banner ──────────────────────────────── Phiber  v0.1.0 ─┐
//! ├─ New File ─────────────┬─ Recent Files ─────────────────┤
//! │  General               │  scene.blend      ~/Projects   │
//! │  Shading …             │  … (5 rows)          More…     │
//! ├─ Open… ────────────────┼─ What's New ───────────────────┤
//! │  Recover Last Session  │  Support Phiber                │
//! └────────────────────────┴────────────────────────────────┘
//! ```
//! With no recent files the right card becomes "Getting Started" links, as
//! Blender does.
//!
//! Opened by `Phiber ▸ Splash Screen`, and shown once at startup (the seed
//! writes `ctx.modal = "splash"`). Dismissed like any modal: backdrop click,
//! the title-bar close button, or picking anything that commits a `Set`.

pub mod recent;
pub mod set;

use crate::v2::state::BKey as _;
use crate::v2::state::CtxKey;
use crate::v2::state::ModalKey;
use crate::v2::state::FileKey;
use crate::v2::state::BState;
use crate::v2::state::val::V_EMPTY;
use crate::v2::state::val::V_MODAL_FILE_BROWSER;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BRowShade;
use crate::v2::view::prim::BStrip;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
pub use set::APP_NAME;
pub use set::BSplashEntry;
pub use set::EMPTY_RECENT;
pub use set::FOOT_LINKS;
pub use set::GETTING_STARTED;
pub use set::LINKS;
pub use set::RECENT_MAX;
pub use set::RECENT_ROWS;
pub use set::TEMPLATES;

#[derive(Clone, Copy, Debug, Default)]
pub struct BSplashModal;

/// A titled group of the welcome window. Blender's splash draws these as
/// plain columns of a popup block; Phiber keeps them as panels that do not
/// fold (`BCard` with no `fold`).
fn card(title: &'static str, rows: Vec<BView>) -> BView {
    BCard::panel(title).body(rows).into()
}

fn link_button(l: &BSplashEntry) -> BView {
    BView::Button(
        BControl::new()
            .with_icon(l.icon)
            .with_label(l.label)
            .with_action(l.link_action()),
    )
}

/// Banner — the mark and wordmark, centred both ways in the strip (Blender
/// paints these on the splash image). The name is the wordmark, not a
/// label: the brand is drawn, not set. The version sits with the links.
fn banner() -> BView {
    BView::from(BCard::strip(BStrip::Header,
        vec![
            BView::spacer(),
            BView::icon("phiber_banner"),
            BView::icon("phiber_wordmark_banner"),
            BView::spacer(),
        ],
    ))
}

/// "New File" — one row per workspace template.
fn new_file() -> BView {
    card(
        "New File",
        TEMPLATES
            .iter()
            .map(|t| {
                BView::Button(
                    BControl::new()
                        .with_icon(t.icon)
                        .with_label(t.label)
                        .with_action(BAction::Set(CtxKey::Space.path(), t.target.to_string())),
                )
            })
            .collect(),
    )
}

/// "Recent Files" (5 rows + More…), or "Getting Started" when there are none.
fn recent_or_getting_started(s: &BState) -> BView {
    let paths = recent::recent_paths(s);
    if paths.is_empty() {
        return card("Getting Started", GETTING_STARTED.iter().map(link_button).collect());
    }
    let mut rows: Vec<BView> = paths
        .iter()
        .take(RECENT_ROWS)
        .enumerate()
        .map(|(i, path)| {
            let shade = if i % 2 == 1 { BRowShade::Alt } else { BRowShade::Flat };
            BView::list_row(
                vec![
                    BView::Button(
                        BControl::new()
                            .with_icon("file_blend")
                            .with_label(recent::display_name(path).to_string())
                            .with_action(BAction::Set(FileKey::Selected.path(), path.clone())),
                    ),
                    BView::label(recent::parent_dir(path).to_string()),
                ],
                shade,
            )
        })
        .collect();
    if paths.len() > RECENT_ROWS {
        rows.push(BView::Button(
            BControl::new()
                .with_icon("viewzoom")
                .with_label("More\u{2026}")
                .with_action(BAction::Set(ModalKey::Open.path(), V_MODAL_FILE_BROWSER.to_string())),
        ));
    }
    card("Recent Files", rows)
}

/// Second split, left: Open… and Recover Last Session.
fn open_card() -> BView {
    BCard::headless()
        .body(vec![
            BView::Button(
                BControl::new()
                    .with_icon("file_folder")
                    .with_label("Open\u{2026}")
                    .with_action(BAction::Set(ModalKey::Open.path(), V_MODAL_FILE_BROWSER.to_string())),
            ),
            BView::Button(
                BControl::new()
                    .with_icon("history")
                    .with_label("Recover Last Session")
                    .with_action(BAction::Set(
                        FileKey::RecoverLastSession.path(),
                        "true".to_string(),
                    )),
            ),
        ])
        .into()
}

/// Second split, right: What's New and the support link.
fn links_card() -> BView {
    let mut rows: Vec<BView> = FOOT_LINKS.iter().map(link_button).collect();
    rows.push(BView::label(format!("Phiber v{}", env!("CARGO_PKG_VERSION"))));
    BCard::headless().body(rows).into()
}

impl BViewItem for BSplashModal {
    fn build(&self, s: &BState) -> BView {
        BView::column(vec![
            banner(),
            BView::row(vec![new_file(), recent_or_getting_started(s)]),
            BView::row(vec![open_card(), links_card()]),
        ])
    }
}

/// Action that dismisses the welcome window without changing anything else.
pub fn dismiss() -> BAction {
    BAction::Set(ModalKey::Open.path(), V_EMPTY.to_string())
}
