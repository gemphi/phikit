//! Activity Bar set — global navigation entries (Phiber-native).
//! Vertical strip at the leftmost of the app.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BActivityEntry {
    pub icon: &'static str,
    pub label: &'static str,
    pub group: BActivityGroup,
    /// When `Some`, clicking the entry opens the named modal via `K_MODAL`
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

const fn e(
    icon: &'static str,
    label: &'static str,
    group: BActivityGroup,
    modal: Option<&'static str>,
) -> BActivityEntry {
    BActivityEntry { icon, label, group, modal }
}

use crate::v2::state::val::{V_MODAL_PROFILE, V_MODAL_SETTINGS};

pub static ENTRIES: &[BActivityEntry] = &[
    // Top group — placeholders until their destinations exist.
    e("home",     "Home",     BActivityGroup::Top,    None),
    e("chat",     "Chat",     BActivityGroup::Top,    None),
    e("grid",     "Spaces",   BActivityGroup::Top,    None),
    e("stream",   "Stream",   BActivityGroup::Center, None),
    // Bottom group — open real modals so clicks do something visible.
    e("settings", "Settings", BActivityGroup::Bottom, Some(V_MODAL_SETTINGS)),
    e("user",     "Profile",  BActivityGroup::Bottom, Some(V_MODAL_PROFILE)),
];
