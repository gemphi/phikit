//! Welcome window — the Phiber home screen.
//!
//! This was Blender's splash (`WM_OT_splash`, `WM_MT_splash`): a banner
//! over New File | Recent Files and Open… | links. It is a **home** now,
//! and the three changes are deliberate:
//!
//! ```text
//! ┌──────────── the mark: two bands, then the face ────────────┐
//! │                    ▂▂▂▂▂▂▂▂▂▂▂▂                            │
//! │                 ▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂▂                         │
//! │              ▂▂▂▂▂▂  ⬡ Phiber  ▂▂▂▂▂▂                      │
//! ├─ Quick access ──┬─ Upcoming ──┬─ Quests ──┬─ Center ───────┤
//! │  Recent         │             │           │  the library   │
//! │  Favourites     │             │           │                │
//! │  Shared         │             │           │                │
//! └─────────────────┴─────────────┴───────────┴────────────────┘
//! ```
//!
//! 1. **The mark is the layers** (`mark.rs`) — the same `BDeck` the manual
//!    draws every figure with, read as a cascade: two bands, then the face
//!    carrying the logo. Half a hexagon above, the face below; the shape is
//!    the bands and there is no outline.
//! 2. **No header backgrounds.** Every card is `BCard::headless()` and its
//!    title is a label in the body. A filled header bar across the top of
//!    the brand is the one chrome shade this window must not have.
//! 3. **Four groups** (`home.rs`) — Quick access, Upcoming, Quests and
//!    Center, the library.
//!
//! Opened by `Phiber ▸ Splash Screen`, and shown once at startup (the seed
//! writes `ctx.modal = "splash"`). Dismissed like any modal.

pub mod home;
pub mod mark;
/// Six terrains traced off the welcome video's frames.
pub mod terrain;
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

/// A titled group of the welcome window.
///
/// Headless, with the title as a label in the body: this window draws no
/// filled header bar anywhere (`mark.rs`), so a titled `BCard::panel`
/// would be the one shade out of place on it.
fn card(title: &'static str, rows: Vec<BView>) -> BView {
    let mut body = vec![BView::label(title.to_string())];
    body.extend(rows);
    BCard::headless().body(body).into()
}

fn link_button(l: &BSplashEntry) -> BView {
    BView::Button(
        BControl::new()
            .with_icon(l.icon)
            .with_label(l.label)
            .with_action(l.link_action()),
    )
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
        // The mark first, then the groups across, then what the file
        // actions and links were. Recent keeps its own card because it
        // reads the store rather than a static table.
        let mut rows = vec![mark::card(), BView::row(home::GROUPS.iter().map(home::BHomeGroup::view).collect())];
        // New File sits with Open… rather than in its own split: the four
        // groups took the top row, and a template is a file action like the
        // two beside it. Dropping it would have taken the workspace
        // templates off the welcome window altogether, which is what the
        // dead-code warning on `new_file` was actually reporting.
        rows.push(BView::row(vec![recent_or_getting_started(s), new_file(), open_card()]));
        rows.push(links_card());
        BView::column(rows)
    }
}

/// Action that dismisses the welcome window without changing anything else.
pub fn dismiss() -> BAction {
    BAction::Set(ModalKey::Open.path(), V_EMPTY.to_string())
}
