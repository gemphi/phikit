//! Activity Bar set — global navigation entries (Phiber-native).
//! Vertical strip at the leftmost of the app, laid out as Slack lays out
//! its rail: each primary is an icon with its name under it, and choosing
//! one opens its panel beside the rail — Home's files, Chat's channels,
//! Spaces' workspaces, Stream's sessions, Settings' categories, and the
//! Profile card.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

/// What an activity entry's secondary strip carries.
///
/// A primary icon is a *grouping*, so clicking it opens a second strip beside
/// the rail holding that group's options. The options are never invented
/// here: each variant names a table the app already keeps, so the strip and
/// the screen it stands for can never disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BActivityOptions {
    /// New file and the recent files, as the splash lists them.
    Home,
    /// The session's channels (`collab.channel.*`).
    Channels,
    /// The workspaces, as the space bar lists them.
    Spaces,
    /// Who is in the session (`collab.peer.*`).
    Streams,
    /// The Preferences categories, as its nav bar lists them.
    Settings,
    /// The person: their picture, name and the sections under them.
    Profile,
    /// No grouping: the entry goes straight to its destination.
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BActivityEntry {
    pub icon: &'static str,
    pub label: &'static str,
    pub group: BActivityGroup,
    /// What this entry's secondary strip carries.
    pub options: BActivityOptions,
    /// When `Some`, clicking the entry opens the named modal via `ModalKey::Open.path()`
    /// (values from `state::val::V_MODAL_*`). `None` entries track only the
    /// active tab state — reserved for entries whose destinations don't yet
    /// have a modal renderer wired.
    pub modal: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BActivityGroup {
    Top,
    Center,
    Bottom,
}

impl BSpec for BActivityEntry {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Activity)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

/// An entry that opens a secondary strip instead of going straight to a modal.
const fn grouping(
    icon: &'static str,
    label: &'static str,
    group: BActivityGroup,
    options: BActivityOptions,
) -> BActivityEntry {
    BActivityEntry { icon, label, group, options, modal: None }
}

use crate::v2::state::val::{V_MODAL_COLLAB, V_MODAL_PROFILE, V_MODAL_SPLASH};

pub static ENTRIES: &[BActivityEntry] = &[
    // Every primary opens its panel beside the rail; the modal it names is
    // what an option in that panel raises (`choose.rs`).
    BActivityEntry { icon: "home", label: "Home", group: BActivityGroup::Top, options: BActivityOptions::Home, modal: Some(V_MODAL_SPLASH) },
    BActivityEntry { icon: "chat", label: "Chat", group: BActivityGroup::Top, options: BActivityOptions::Channels, modal: Some(V_MODAL_COLLAB) },
    grouping("grid", "Spaces", BActivityGroup::Top, BActivityOptions::Spaces),
    BActivityEntry { icon: "stream", label: "Stream", group: BActivityGroup::Center, options: BActivityOptions::Streams, modal: Some(V_MODAL_COLLAB) },
    grouping("settings", "Settings", BActivityGroup::Bottom, BActivityOptions::Settings),
    BActivityEntry { icon: "user", label: "Profile", group: BActivityGroup::Bottom, options: BActivityOptions::Profile, modal: Some(V_MODAL_PROFILE) },
];
