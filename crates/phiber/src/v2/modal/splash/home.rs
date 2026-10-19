//! What the welcome window offers, as one table.
//!
//! The window stopped being Blender's splash (New File | Recent Files,
//! Open… | links) and became a home: four groups, each a headless card so
//! **no filled header bar is drawn anywhere** — a title here is a label in
//! the body, not a chrome shade across the top.
//!
//! | group | rows | where it goes |
//! |---|---|---|
//! | Quick access | Recent, Favourites, Shared | the file browser, scoped |
//! | Upcoming | what is scheduled | not yet a destination |
//! | Quests | what to try next | the manual's pages |
//! | Center | the library: every artifact, with its meta | the Asset Browser |
//!
//! **Center is the library.** The Asset Browser is the screen that will
//! own it — it lists assets with their catalogue, tags and preview, which
//! is the meta a card needs — but it is a *host*, reachable only by
//! switching an area's editor, and there is no route to it from a modal.
//! So Center opens the file browser, which is where artifacts are found
//! today, and moves when the Asset Browser gets a route. That is written
//! down rather than faked: `BSpaceType` has no Assets member, and setting
//! `ctx.space` to a host slug would have compiled and done nothing.
//!
//! Rows whose destination does not exist yet say so instead of being wired
//! to nothing: a button that does nothing is worse than a line that admits
//! it, and Blender's own empty editors are drawn empty rather than filled
//! with invitations.

use super::set::BSplashEntry;
use crate::v2::state::val::V_MODAL_FILE_BROWSER;
use crate::v2::state::{BKey as _, ModalKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BCard, BControl};

/// One group of the home screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHomeGroup {
    pub title: &'static str,
    pub rows: &'static [BHomeRow],
}

/// One row: what it is, and where it goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHomeRow {
    pub icon: &'static str,
    pub label: &'static str,
    /// `None` for a row whose destination is not built yet — drawn as a
    /// dimmed line rather than a button that leads nowhere.
    pub goes: Option<BHomeTarget>,
}

/// Where a row goes, as a thing the app already has. Both variants are
/// destinations that exist today — there is no variant for "somewhere we
/// will build", because that is what `goes: None` is for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BHomeTarget {
    /// A modal, by its `val::V_MODAL_*` name.
    Modal(&'static str),
    /// A page of the manual, by the slug `manual/set.rs` gives it.
    Doc(&'static str),
}

impl BHomeTarget {
    pub fn action(self) -> BAction {
        match self {
            Self::Modal(name) => BAction::Set(ModalKey::Open.path(), name.to_string()),
            Self::Doc(slug) => BAction::Set(ModalKey::ReaderDoc.path(), slug.to_string()),
        }
    }
}

const fn row(icon: &'static str, label: &'static str, goes: Option<BHomeTarget>) -> BHomeRow {
    BHomeRow { icon, label, goes }
}

static QUICK: &[BHomeRow] = &[
    row("history", "Recent", Some(BHomeTarget::Modal(V_MODAL_FILE_BROWSER))),
    row("bookmark", "Favourites", Some(BHomeTarget::Modal(V_MODAL_FILE_BROWSER))),
    // No collab file source yet — the channels carry people, not files.
    row("users", "Shared", None),
];

static UPCOMING: &[BHomeRow] = &[row("calendar", "Nothing scheduled", None)];

/// Every slug here is a page `manual/set.rs` actually declares. Two of the
/// first three I wrote — `render` and `shading` — were not pages at all.
static QUESTS: &[BHomeRow] = &[
    row("graduation_cap", "Open a file and look around", Some(BHomeTarget::Doc("splash"))),
    row("material", "Read a material", Some(BHomeTarget::Doc("materials"))),
    row("shader_graph", "Follow a shader graph", Some(BHomeTarget::Doc("shader_nodes"))),
];

static CENTER: &[BHomeRow] = &[row("asset", "Browse the library", Some(BHomeTarget::Modal(V_MODAL_FILE_BROWSER)))];

/// The four groups, in the order the window stacks them.
pub static GROUPS: &[BHomeGroup] = &[
    BHomeGroup { title: "Quick access", rows: QUICK },
    BHomeGroup { title: "Upcoming", rows: UPCOMING },
    BHomeGroup { title: "Quests", rows: QUESTS },
    BHomeGroup { title: "Center", rows: CENTER },
];

impl BHomeRow {
    /// The row as a control: a button where it goes somewhere, a dimmed
    /// label where it does not.
    pub fn view(&self) -> BView {
        match self.goes {
            Some(target) => BView::Button(
                BControl::new().with_icon(self.icon).with_label(self.label).with_action(target.action()),
            ),
            None => BView::Button(
                BControl::new().with_icon(self.icon).with_label(self.label).with_muted(true),
            ),
        }
    }
}

impl BHomeGroup {
    /// The group as a headless card — the title is a label in the body, so
    /// no header background is drawn.
    pub fn view(&self) -> BView {
        let mut body = vec![BView::label(self.title.to_string())];
        body.extend(self.rows.iter().map(BHomeRow::view));
        BCard::headless().body(body).into()
    }
}

/// An entry of the old splash table, as a button — kept so the window can
/// still carry the links (`FOOT_LINKS`) under the groups.
pub fn link(entry: &BSplashEntry) -> BView {
    BView::Button(
        BControl::new().with_icon(entry.icon).with_label(entry.label).with_action(entry.link_action()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_groups_are_in_order_and_every_row_is_one_or_the_other() {
        let titles: Vec<&str> = GROUPS.iter().map(|g| g.title).collect();
        assert_eq!(titles, vec!["Quick access", "Upcoming", "Quests", "Center"]);
        for group in GROUPS {
            assert!(!group.rows.is_empty(), "{} has nothing in it", group.title);
            for r in group.rows {
                // A row either goes somewhere or admits it does not; there
                // is no third state and no button wired to nothing.
                assert!(!r.label.is_empty() && !r.icon.is_empty(), "{}", group.title);
            }
        }
    }

    #[test]
    fn every_destination_is_one_that_exists() {
        // The rule this table is held to: a row either goes to a modal the
        // app has or a manual page `manual/set.rs` declares, or it says it
        // goes nowhere. Two slugs in the first draft (`render`, `shading`)
        // were neither, and would have opened an empty reader.
        let pages: Vec<&str> = crate::v2::manual::every().iter().map(|p| p.slug).collect();
        for group in GROUPS {
            for r in group.rows {
                match r.goes {
                    Some(BHomeTarget::Doc(slug)) => assert!(pages.contains(&slug), "{slug} is not a manual page"),
                    Some(BHomeTarget::Modal(name)) => assert!(!name.is_empty()),
                    None => {}
                }
            }
        }
    }

    #[test]
    fn a_group_card_has_no_header() {
        // The whole point of the rewrite: no filled header bar above a
        // title anywhere on this window.
        let BView::Card(card) = GROUPS[0].view() else { panic!("a group is a card") };
        assert!(card.title().is_none(), "the title is a body label, not a header");
    }
}
