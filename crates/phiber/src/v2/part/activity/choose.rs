//! What choosing something on the activity rail does.
//!
//! An entry is either a grouping — it opens a secondary strip beside the
//! rail, and clicking it again closes the strip — or a destination, which
//! raises its declared modal. An option chosen from the strip arrives as
//! `"<entry>:<option>"` and lands where the grouping says it lands.

use super::option;
use super::set::BActivityOptions;
use crate::v2::space::BSpaceType;
use crate::v2::state::val::V_MODAL_SETTINGS;
use crate::v2::state::{ActivityKey, BKey, BPath, BState, CtxKey, ModalKey, rule::BRule};

/// `activity.tab` → open/close a grouping's strip, or raise the entry's modal.
pub struct BActivityTabRule;

impl BRule for BActivityTabRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == ActivityKey::Tab.path()
    }
    fn after(&self, _: &BPath, label: &str, s: &mut BState) {
        if option::grouping_named(label).is_some() {
            if s.get(ActivityKey::Group) == Some(label) {
                s.remove(ActivityKey::Group);
            } else {
                s.set(ActivityKey::Group, label.to_string());
            }
            return;
        }
        s.remove(ActivityKey::Group);
        let entry = super::set::ENTRIES.iter().find(|e| e.label == label);
        if let Some(modal) = entry.and_then(|e| e.modal) {
            s.set(ModalKey::Open, modal);
        }
    }
}

/// `activity.pick` → the space, or the Preferences tab *and* its screen, so
/// one click lands somewhere. The pick itself is not kept.
pub struct BActivityPickRule;

impl BRule for BActivityPickRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == ActivityKey::Pick.path()
    }
    fn after(&self, _: &BPath, pick: &str, s: &mut BState) {
        let Some((entry, option)) = pick.split_once(':') else { return };
        let Some(entry) = option::grouping_named(entry) else { return };
        match entry.options {
            BActivityOptions::Spaces => {
                if let Some(space) = BSpaceType::ALL.iter().find(|sp| sp.label() == option) {
                    s.set(CtxKey::Space, space.slug());
                }
            }
            BActivityOptions::Settings => {
                s.set(&option::prefs_tab_key(), option.to_string());
                s.set(ModalKey::Open, V_MODAL_SETTINGS);
            }
            // Home: a new file raises the splash, Open… the browser, a recent
            // file opens (the commit the splash's own rows make).
            BActivityOptions::Home => match option {
                "new" => s.set(ModalKey::Open, crate::v2::state::val::V_MODAL_SPLASH),
                "open" => s.set(crate::v2::state::FileKey::Purpose, "open"),
                recent => {
                    if let Some(path) = recent.strip_prefix("recent:") {
                        s.set(crate::v2::state::FileKey::Selected, path.to_string());
                    }
                }
            },
            // A channel or a peer opens the collaboration window on it.
            BActivityOptions::Channels | BActivityOptions::Streams => {
                if let Some(modal) = entry.modal {
                    s.set(ModalKey::Open, modal);
                }
            }
            BActivityOptions::Profile | BActivityOptions::None => {}
        }
        s.remove(ActivityKey::Pick);
    }
}
