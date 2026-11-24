//! The activity bar: a narrow column of icons, the same in every app.
//! The app's own mark is at its head; its rows stand at its top, in its
//! middle and at its foot, where the app's data seats them; the user is
//! at its very foot. A press on a row that has a menu shows that menu
//! beside the bar, in a second column (`menu.rs`), and a press on it
//! again puts the menu away. It is puikit's shell
//! (`src/components/navigation/AppShell`); the rows are the app's data,
//! two tables, and the user is another.
//!
//! At a desk it stands at the left. In a hand it lies at the bottom, its
//! rows squares in a line, and a menu is drawn out over the space. This
//! file is the bar's rows; `plan.rs` is where each is, and `draw.rs`
//! draws them. An app may add rows while it runs (`PRail`): the places
//! a person is in, each with its picture.
//!
//! In a theme that gives the window a caption (`caption.rs`) the mark and
//! the button that puts a menu away are there, over the bar, and the
//! bar and its menu have no head of their own.

mod draw;
mod plan;

pub use plan::PRail;

use super::PCaption;
use super::menu::{self, PItem, PSeat};
use crate::pui::core::{PPaint, PRect};
use crate::pui::window::{PDeed, PEntry, PWindow};
use crate::service::PTable;
use crate::theme::PTheme;

/// The theme's tokens for the bar: how thick it is, how wide a menu is
/// beside it, and its fill.
pub const LOW: &str = "activity.low";
pub const WIDE: &str = "activity.width";
pub const FILL: &str = "activity.fill";
/// A faint line down the bar's side toward the page, as plain looks
/// draw one (Apple's sidebar has a hairline at its edge): its colour,
/// and how thick it is. A look without the colour draws none.
pub const EDGE: &str = "activity.line";
pub const RIM: &str = "activity.rim";
/// How round the corners of the bar are, where it stands clear of the
/// window's edges (`activity.margin`, which the frame reads).
pub const CORNER: &str = "activity.corner";
/// The seat a table gives the row that is the app's own mark.
pub const HEAD: &str = "head";
/// The icon of the button that puts a menu away.
pub const PIN: &str = "panel-left";

/// The activity bar.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PActivity {
    /// The app's own mark, at the head: its name, and the icon that is
    /// its logo. With no icon its first letter stands in.
    pub logo: PEntry,
    /// What a press on the mark does, where the table names something.
    pub home: Option<PDeed>,
    /// Its rows, each with its seat and its menu.
    pub item: Vec<PItem>,
    /// The rows the app adds while it runs (`PRail`), after its top rows;
    /// and how many of them the wheel has rolled out of sight.
    pub live: Vec<PItem>,
    pub skip: usize,
    /// The user, at its foot: their name, and the modal a press opens.
    pub user: Option<PItem>,
    /// The spaces that stand with no menu beside the bar: a page that is
    /// its own columns, as a library's tree and what is in it.
    pub bare: Vec<String>,
}

/// The lines of a table that say something.
fn line(text: &str) -> impl Iterator<Item = Vec<&str>> {
    let said = text.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#'));
    said.map(|line| line.split('\t').map(str::trim).collect())
}

/// The theme's tokens for how much of its ground a thing in the bar
/// fills: a row's icon, and a row's picture. Each is a share of the
/// ground's side (0.667 leaves a sixth of it clear each side). A theme
/// without one has an icon at its own size and a picture seven fifths
/// of that, as before there was a word for it.
pub const SHARE: &str = "activity.icon";
pub const FACE: &str = "activity.face";

/// How large something is on a ground of that side: the share of it the
/// theme says under that token, in whole dots; or `plain` with no word.
pub(super) fn share(paint: &dyn PPaint, token: &str, ground: f32, plain: f32) -> f32 {
    paint.theme().size(token).map_or(plain, |share| (ground * share).round())
}

/// The square of that side in the middle of a rectangle.
pub(super) fn square(rect: PRect, side: f32) -> PRect {
    PRect::new(rect.center().x - side / 2.0, rect.center().y - side / 2.0, side, side)
}

impl PActivity {
    /// A bar from a table: one row a line, as its slug, its label, its
    /// icon, its seat (`top`, `center`, `bottom`), and the kind and the
    /// target of its deed (`space chat`), between tabs. The row whose
    /// seat is `head` is the app's own mark, and its deed what a press on
    /// the mark does. A row may name no deed: it is one that only shows
    /// its menu.
    pub fn read(text: &str) -> PActivity {
        let mut bar = PActivity::default();
        for cell in line(text) {
            let word = |index: usize| cell.get(index).copied().unwrap_or_default();
            let entry = PEntry::new(word(0), word(1), word(2));
            match word(3) {
                HEAD => (bar.logo, bar.home) = (entry, PDeed::named(word(4), word(5))),
                seat => bar.item.push(PItem { entry, deed: PDeed::named(word(4), word(5)), seat: PSeat::named(seat), ..PItem::default() }),
            }
        }
        bar
    }

    /// The same bar with the menus of its rows, from a table: one row of
    /// a menu a line, as the slug of the row of the bar it belongs to,
    /// and then what `PItem::hold` reads.
    /// The same bar, with no menu beside it while one of these spaces
    /// (their names, between spaces) is shown.
    pub fn bare(mut self, space: &str) -> PActivity {
        self.bare = space.split_whitespace().map(str::to_string).collect();
        self
    }

    pub fn hold(mut self, text: &str) -> PActivity {
        for cell in line(text) {
            let parent = self.item.iter_mut().find(|item| Some(&item.entry.slug.as_str()) == cell.first());
            parent.into_iter().for_each(|item| item.hold(&cell[1..]));
        }
        self
    }

    /// The same bar with the user at its foot. The user is a table: their
    /// `name`, the `modal` a press opens, and their picture where they
    /// have one (`face`). A table without the first two gives no user.
    pub fn with(self, user: &PTable) -> PActivity {
        let (name, modal) = (user.get("name"), user.get("modal"));
        let known = !name.is_empty() && !modal.is_empty();
        let button = known.then(|| PItem { entry: PEntry::new("user", name, ""), deed: Some(PDeed::Open(modal.to_string())), seat: PSeat::Bottom, face: super::picture(user), ..PItem::default() });
        PActivity { user: button, ..self }
    }

    /// Whether it has nothing on it: such a bar takes no room.
    pub fn is_empty(&self) -> bool {
        self.item.is_empty() && self.user.is_none()
    }

    /// How thick the bar is: what the theme says, and nothing with
    /// nothing on it.
    pub fn thick(&self, theme: &dyn PTheme) -> f32 {
        match self.is_empty() {
            true => 0.0,
            false => theme.size(LOW).unwrap_or_default(),
        }
    }

    /// The row whose menu is shown: the one the window picked, while its
    /// menu is not put away. `lying` is for a bar in a hand, where a
    /// menu is drawn out over the space and starts put away.
    pub fn shown(&self, window: &PWindow, lying: bool) -> Option<&PItem> {
        let open = match lying {
            true => window.drawer(),
            false => !window.fold(),
        };
        let bare = self.bare.iter().any(|space| window.shows(&PDeed::Show(space.clone())));
        let open = open && !bare;
        self.item.iter().find(|item| open && item.entry.slug == window.pick() && !item.group.is_empty())
    }

    /// How wide a menu is beside the bar: what the theme says while one
    /// is shown, and nothing otherwise.
    pub fn wide(&self, window: &PWindow, lying: bool, theme: &dyn PTheme) -> f32 {
        match self.shown(window, lying) {
            Some(_) => theme.size(WIDE).unwrap_or_default(),
            None => 0.0,
        }
    }

    /// The row a window picks as it opens: of those with a menu, the
    /// first whose own deed is what the window shows, or else the first.
    pub fn first(&self, window: &PWindow) -> &str {
        let menu = || self.item.iter().filter(|item| !item.group.is_empty());
        let shown = menu().find(|item| item.deed.as_ref().is_some_and(|deed| window.shows(deed)));
        shown.or_else(|| menu().next()).map_or("", |item| item.entry.slug.as_str())
    }

    /// The deed of the row of that name: picking a row does it as well.
    pub fn deed(&self, slug: &str) -> Option<PDeed> {
        self.item.iter().find(|item| item.entry.slug == slug).and_then(|item| item.deed.clone())
    }

    /// What a press on a row does. One with a menu shows it, or puts it
    /// away when it is the one shown; one with none does its own deed.
    fn press(&self, item: &PItem, window: &PWindow, lying: bool) -> Option<PDeed> {
        let open = self.shown(window, lying).is_some_and(|shown| shown.entry.slug == item.entry.slug);
        match (item.group.is_empty(), open, lying) {
            (true, ..) => item.deed.clone(),
            (false, true, true) => Some(PDeed::Side),
            (false, true, false) => Some(PDeed::Fold),
            (false, false, _) => Some(PDeed::Pick(item.entry.slug.clone())),
        }
    }

    /// A menu's own rectangle `side` as its head, a row as high as the
    /// bar is thick, whose right end is the button that puts it away;
    /// and the room under that for its groups. Under a caption it has no
    /// head, and the room is all of it.
    fn part(side: PRect, theme: &dyn PTheme) -> (PRect, PRect) {
        let low = theme.size(LOW).filter(|_| PActivity::headed(theme)).unwrap_or_default();
        let pin = PRect::new(side.origin.x + side.size.width - low, side.origin.y, low, low);
        (pin, PRect::new(side.origin.x, side.origin.y + low, side.size.width, (side.size.height - low).max(0.0)))
    }

    /// Where a press does something, and what: on the bar, and on the
    /// menu shown in `side`. Standing, the mark puts the menu away and
    /// brings it back.
    pub fn spot(&self, bar: PRect, side: PRect, window: &PWindow, theme: &dyn PTheme) -> Vec<(PRect, PDeed)> {
        let plan = self.plan(bar, theme);
        let head = (!plan.lying && PActivity::headed(theme)).then_some((plan.head, PDeed::Fold));
        let item = plan.item.iter().filter_map(|(rect, item)| Some((*rect, self.press(item, window, plan.lying)?)));
        let user = self.user.iter().filter_map(|user| Some((plan.user, user.deed.clone()?)));
        let mut out: Vec<(PRect, PDeed)> = head.into_iter().chain(item).chain(user).collect();
        if let Some(shown) = self.shown(window, plan.lying) {
            let (pin, room) = PActivity::part(side, theme);
            out.extend(PActivity::headed(theme).then(|| (pin, self.press(shown, window, plan.lying).unwrap_or(PDeed::Fold))));
            out.extend(menu::spot(&shown.group, room, theme));
        }
        out
    }

    /// The room the menu shown in `side` leaves for the app's side part.
    pub fn room(&self, side: PRect, window: &PWindow, lying: bool, theme: &dyn PTheme) -> PRect {
        let shown = self.shown(window, lying);
        shown.map(|shown| menu::room(&shown.group, PActivity::part(side, theme).1, theme)).unwrap_or_default()
    }
}
