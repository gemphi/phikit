//! Activity Bar model — data definitions for the navigation strip.
use crate::svg_icons::IconId;

/// An item in the activity bar.
#[derive(Debug, Clone)]
pub struct NSActivityBarItem {
    pub id: &'static str,
    pub label: &'static str,
    pub icon_id: IconId,
    pub group: ActivityBarGroup,
}

/// Which vertical group an item belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityBarGroup {
    /// Pinned to the top of the bar.
    Top,
    /// Centered vertically in the bar.
    Middle,
    /// Pinned to the bottom of the bar.
    Bottom,
}

/// Activity bar state.
#[derive(Debug, Clone)]
pub struct NSActivityBarState {
    pub items: Vec<NSActivityBarItem>,
    pub active_id: &'static str,
}

impl NSActivityBarState {
    /// Default activity bar matching the Reality Composer Pro layout.
    pub fn new() -> Self {
        Self {
            active_id: "home",
            items: vec![
                // Top group: recent activity indicator
                NSActivityBarItem {
                    id: "clock",
                    label: "Recent",
                    icon_id: IconId::Clock,
                    group: ActivityBarGroup::Top,
                },
                // Middle group: workspace navigation
                NSActivityBarItem {
                    id: "home",
                    label: "Home",
                    icon_id: IconId::Scene,
                    group: ActivityBarGroup::Middle,
                },
                NSActivityBarItem {
                    id: "hierarchy",
                    label: "Hierarchy",
                    icon_id: IconId::World,
                    group: ActivityBarGroup::Middle,
                },
                NSActivityBarItem {
                    id: "chat",
                    label: "Chat",
                    icon_id: IconId::Chat,
                    group: ActivityBarGroup::Middle,
                },
                NSActivityBarItem {
                    id: "run",
                    label: "Run",
                    icon_id: IconId::Play,
                    group: ActivityBarGroup::Middle,
                },
                NSActivityBarItem {
                    id: "extensions",
                    label: "Extensions",
                    icon_id: IconId::Package,
                    group: ActivityBarGroup::Middle,
                },
                // Bottom group: user/settings
                NSActivityBarItem {
                    id: "settings",
                    label: "Settings",
                    icon_id: IconId::Settings,
                    group: ActivityBarGroup::Bottom,
                },
                NSActivityBarItem {
                    id: "user",
                    label: "Account",
                    icon_id: IconId::Sphere,
                    group: ActivityBarGroup::Bottom,
                },
            ],
        }
    }
}
