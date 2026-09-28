//! The activity bar's panel — the options behind one primary icon.
//!
//! Every list here is a projection of a table the app already keeps, so the
//! panel cannot offer a workspace the space bar does not have, a settings
//! category the Preferences nav bar does not have, or a channel and a peer
//! the collaboration service has not written into `collab.*`.
//!
//! Choosing an option writes one key, `ctx.activity.pick`, as
//! `"<entry>:<option>"`. `choose::BActivityPickRule` turns that into the
//! keys the choice actually means — the same shape the compass's viewpoint
//! caps use, and the reason a row here needs no compound action.

use crate::v2::state::{BKey as _, ActivityKey, AppKey, CtxKey};
use super::set::BActivityEntry;
use super::set::BActivityOptions;
use super::set::ENTRIES;
use crate::v2::host::BHostType;
use crate::v2::modal::setting::CATEGORIES;
use crate::v2::modal::splash::recent;
use crate::v2::space::BSpaceType;
use crate::v2::state::{
    BPath, BState,
};
use crate::v2::view::prim::{BAction, BControl};

/// Peer and channel slots (`collab.peer.<n>`, `collab.channel.<n>`).
pub const SLOTS: [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15"];

/// The Preferences nav-bar tab key, so a category chosen here selects the
/// same tab the Preferences screen reads.
pub fn prefs_tab_key() -> BPath {
    BPath::owned(vec![BHostType::Prefs.slug(), "navbar", "tab"])
}

/// The session's channels as `(name, owner)`, in slot order.
pub fn channels(s: &BState) -> Vec<(String, String)> {
    SLOTS
        .iter()
        .map_while(|n| s.get(AppKey::CollabChannel(n)).map(str::to_string))
        .map(|line| {
            let (name, owner) = line.split_once('\t').unwrap_or((line.as_str(), ""));
            (name.to_string(), owner.to_string())
        })
        .collect()
}

/// Who is in the session as `(name, state)`, in slot order.
pub fn peers(s: &BState) -> Vec<(String, String)> {
    SLOTS
        .iter()
        .map_while(|n| s.get(AppKey::CollabPeer(n)).map(str::to_string))
        .map(|line| {
            let (name, state) = line.split_once('\t').unwrap_or((line.as_str(), ""));
            (name.to_string(), state.to_string())
        })
        .collect()
}

/// One entry's options, in the order its own screen lists them. An empty
/// table shows as one row saying so, never as an empty panel.
pub fn options_for(entry: &BActivityEntry, s: &BState) -> Vec<BControl> {
    let pick = |option: &str| BAction::Set(ActivityKey::Pick.path(), format!("{}:{}", entry.label, option));
    let row = |icon: &str, label: &str, option: &str| BControl::new().with_icon(icon).with_label(label).with_action(pick(option));
    let none = |label: &str| vec![BControl::new().with_icon("info").with_label(label)];
    match entry.options {
        BActivityOptions::Home => {
            let mut rows = vec![row("file", "New File", "new"), row("folder", "Open...", "open")];
            for path in recent::recent_paths(s) {
                rows.push(row("blender", recent::display_name(&path), &format!("recent:{path}")));
            }
            rows
        }
        BActivityOptions::Channels => {
            let list = channels(s);
            if list.is_empty() {
                return none("No channels yet");
            }
            list.iter().map(|(name, _)| row("hash", name, name)).collect()
        }
        BActivityOptions::Spaces => {
            let active = s.get(CtxKey::Space);
            BSpaceType::ALL
                .iter()
                .map(|space| row(space.icon(), space.label(), space.label()).with_active(Some(space.slug()) == active))
                .collect()
        }
        BActivityOptions::Streams => {
            let list = peers(s);
            if list.is_empty() {
                return none("No one else is in this session");
            }
            list.iter().map(|(name, state)| row("user", &format!("{name}  ·  {state}"), name)).collect()
        }
        BActivityOptions::Settings => {
            let active = s.get(&prefs_tab_key());
            CATEGORIES
                .iter()
                .map(|category| row(category.icon, category.label, category.label).with_active(Some(category.label) == active))
                .collect()
        }
        // The profile panel is its own view (`profile.rs`); its sections fold
        // in place rather than pick.
        BActivityOptions::Profile | BActivityOptions::None => Vec::new(),
    }
}

/// True when this entry opens a panel rather than a destination.
pub fn is_grouping(entry: &BActivityEntry) -> bool {
    entry.options != BActivityOptions::None
}

/// The entry named by a label, when it is one that groups.
pub fn grouping_named(label: &str) -> Option<&'static BActivityEntry> {
    ENTRIES.iter().find(|e| e.label == label && is_grouping(e))
}

/// The entry whose panel is open, if any.
pub fn open_group(s: &BState) -> Option<&'static BActivityEntry> {
    grouping_named(s.get(ActivityKey::Group)?)
}
