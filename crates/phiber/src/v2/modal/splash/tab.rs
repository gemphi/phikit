//! The Welcome window's tabs: what the grid under them shows.

use crate::v2::state::BKey as _;
use crate::v2::state::{AppKey, BState};
use crate::v2::view::prim::{BAction, BControl};
use crate::v2::view::BView;

/// The Welcome window's tabs, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BWelcomeTab {
    /// One card per workspace a new file can start in.
    New,
    /// Open…, Recover Last Session, and the files opened last.
    Recent,
    /// The quests, the library, the manual and the links.
    Learn,
}

impl BWelcomeTab {
    pub const ALL: [BWelcomeTab; 3] = [Self::New, Self::Recent, Self::Learn];

    pub fn label(self) -> &'static str {
        match self {
            Self::New => "New File",
            Self::Recent => "Recent Files",
            Self::Learn => "Learn",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Recent => "recent",
            Self::Learn => "learn",
        }
    }

    /// The tab in force: what `welcome.tab` names, else New File.
    pub fn of(s: &BState) -> Self {
        let slug = s.get(AppKey::WelcomeTab).unwrap_or("");
        Self::ALL.into_iter().find(|tab| tab.slug() == slug).unwrap_or(Self::New)
    }

    /// The tab as a button: lit when it is the one in force.
    pub fn view(self, open: Self) -> BView {
        BView::Button(BControl::new().with_label(self.label()).with_active(self == open).with_action(BAction::Set(AppKey::WelcomeTab.path(), self.slug().to_string())))
    }
}
