//! The palette's tabs: what it searches.

use crate::v2::state::{AppKey, BState};

/// What the palette searches: its tabs, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BFindKind {
    All,
    Space,
    Editor,
    Panel,
    Menu,
}

impl BFindKind {
    pub const ALL: [BFindKind; 5] = [Self::All, Self::Space, Self::Editor, Self::Panel, Self::Menu];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Space => "Spaces",
            Self::Editor => "Editors",
            Self::Panel => "Panels",
            Self::Menu => "Menus",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Space => "space",
            Self::Editor => "editor",
            Self::Panel => "panel",
            Self::Menu => "menu",
        }
    }

    /// The tab in force: what `command.tab` names, else All.
    pub fn of(s: &BState) -> Self {
        let slug = s.get(AppKey::CommandTab).unwrap_or("");
        Self::ALL.into_iter().find(|kind| kind.slug() == slug).unwrap_or(Self::All)
    }
}
