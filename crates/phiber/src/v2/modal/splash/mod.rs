//! Welcome window — the Phiber home screen.
//!
//! This was Blender's splash (`WM_OT_splash`, `WM_MT_splash`): a banner
//! over New File | Recent Files and Open… | links. It is a **home** now:
//!
//! ```text
//!                 P H I B E R            ← on the window's own backdrop
//!      Model, shade, animate and render — the Phi way.
//!          ( New File | Recent Files | Learn )   ← one ground, the tabs
//!        ┌──┐ ┌──┐ ┌──┐ ┌──┐                  ← the tab's cards
//!        └──┘ └──┘ └──┘ └──┘
//! ```
//!
//! 1. **The logo stands on the backdrop.** Nothing is painted under it or
//!    its line: it is the window's name, not a card.
//! 2. **The tabs are the only bar.** One ground holds them
//!    (`BStrip::Tabs`); no other background is drawn above the cards.
//! 3. **The tab changes what is under it** (`tab::BWelcomeTab`): New File's
//!    workspaces; Open…, Recover and the recent files; or Learn.
//! 4. **Every choice is a card**, four to a row, centred
//!    (`BView::Thumbnail`).
//!
//! Opened by `Phiber ▸ Welcome…`, and shown once at startup (the seed
//! writes `ctx.modal = "splash"`). Dismissed like any modal.

pub mod home;
/// Six terrains traced off the welcome video's frames.
pub mod terrain;
pub mod recent;
pub mod set;
pub mod tab;

use crate::v2::state::BKey as _;
use crate::v2::state::OpKey;
use crate::v2::state::ModalKey;
use crate::v2::state::FileKey;
use crate::v2::state::BState;
use crate::v2::state::val::V_EMPTY;
pub use tab::BWelcomeTab;
use crate::v2::state::val::V_MODAL_FILE_BROWSER;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BStrip;
use crate::v2::view::BThumb;
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

/// A card's edge on the Welcome window, in px: four to a row fit the
/// window's column with room either side.
pub const CARD: f32 = 112.0;
/// How many cards stand in one row.
pub const ACROSS: usize = 4;

/// One choice as a card: its icon (or the file's own preview), its name,
/// and a line under it.
fn tile(icon: &str, label: &str, subtitle: &str, preview: &str, action: Option<BAction>) -> BView {
    BView::thumbnail(BThumb {
        preview_path: preview.to_string(),
        icon: icon.to_string(),
        label: label.to_string(),
        subtitle: subtitle.to_string(),
        size: CARD,
        selected: false,
        action,
    })
}

fn link_tile(l: &BSplashEntry) -> BView {
    tile(l.icon, l.label, "", "", Some(l.link_action()))
}

/// `views` centred on their own line, as tall as they are: a row of three
/// groups (`draw::row`) — an empty one either side and these between — so
/// the middle group sits on the window's centre.
fn centred(views: Vec<BView>) -> BView {
    BView::row([vec![BView::indent(0), BView::spacer()], views, vec![BView::spacer(), BView::indent(0)]].concat())
}

/// `tiles`, `ACROSS` to a row, each row centred.
fn grid(tiles: Vec<BView>) -> Vec<BView> {
    tiles.chunks(ACROSS).map(|row| centred(row.to_vec())).collect()
}

/// New File: one card per workspace a file can start in. A card starts the
/// file (`wm.read_homefile`), which closes this window, as File ▸ New does.
fn new_file() -> Vec<BView> {
    TEMPLATES.iter().map(|t| tile(t.icon, t.label, "", "", Some(BAction::Set(OpKey::NewFile.path(), t.target.to_string())))).collect()
}

/// Recent Files: Open… and Recover Last Session (the file opened last, open
/// again), then the files opened
/// last, each on its own preview with the folder it is in under its name —
/// or Getting Started when nothing has been opened yet.
fn recent(s: &BState) -> Vec<BView> {
    let mut tiles = vec![tile("folder_open", "Open\u{2026}", "", "", Some(BAction::Set(ModalKey::Open.path(), V_MODAL_FILE_BROWSER.to_string())))];
    let paths = recent::recent_paths(s);
    let Some(last) = paths.first() else {
        tiles.extend(GETTING_STARTED.iter().map(link_tile));
        return tiles;
    };
    // The file the last session had open, opened again.
    let name = recent::display_name(last);
    tiles.push(tile("history", "Recover Last Session", name, "", Some(BAction::Set(FileKey::Selected.path(), last.clone()))));
    tiles.extend(paths.iter().take(RECENT_ROWS).map(|path| {
        let name = recent::display_name(path);
        let folder = recent::display_name(recent::parent_dir(path));
        tile("file_blend", name, folder, path, Some(BAction::Set(FileKey::Selected.path(), path.clone())))
    }));
    if paths.len() > RECENT_ROWS {
        tiles.push(tile("viewzoom", "More\u{2026}", "", "", Some(BAction::Set(ModalKey::Open.path(), V_MODAL_FILE_BROWSER.to_string()))));
    }
    tiles
}

/// Learn: the quests, the library, the manual and the links.
fn learn() -> Vec<BView> {
    let rows = home::GROUPS.iter().flat_map(|group| group.rows.iter());
    let mut tiles: Vec<BView> = rows.map(|r| tile(r.icon, r.label, "", "", r.goes.map(home::BHomeTarget::action))).collect();
    tiles.extend(FOOT_LINKS.iter().map(link_tile));
    tiles
}

/// What Phiber is, in one line under the logo.
pub const TAGLINE: &str = "Model, shade, animate and render \u{2014} the Phi way.";

impl BViewItem for BSplashModal {
    /// The logo and its line on the backdrop, the tabs on their one ground,
    /// then the open tab's cards.
    fn build(&self, s: &BState) -> BView {
        let open = BWelcomeTab::of(s);
        let tabs = BWelcomeTab::ALL.into_iter().map(|t| t.view(open)).collect();
        let mut cards: Vec<BView> = match open {
            BWelcomeTab::New => grid(new_file()),
            BWelcomeTab::Recent => grid(recent(s)),
            BWelcomeTab::Learn => grid(learn()),
        };
        // Learn ends on the version, one more line under the cards.
        if open == BWelcomeTab::Learn {
            cards.push(centred(vec![BView::label(format!("{APP_NAME} v{}", env!("CARGO_PKG_VERSION")))]));
        }
        // Every row of cards is a line of the window's own column, as tall
        // as its cards (`draw::modal::welcome`): a column of rows inside it
        // shared its height out between them and squashed them together.
        let mut lines = vec![
            centred(vec![BView::icon(crate::v2::view::draw::brand::WORDMARK_3D)]),
            centred(vec![BView::label(TAGLINE.to_string())]),
            BCard::strip(BStrip::Tabs, tabs).into(),
        ];
        lines.extend(cards);
        BView::column(lines)
    }
}

/// Action that dismisses the welcome window without changing anything else.
pub fn dismiss() -> BAction {
    BAction::Set(ModalKey::Open.path(), V_EMPTY.to_string())
}
