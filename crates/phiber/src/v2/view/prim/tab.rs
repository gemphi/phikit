//! Tab primitive — strip button that blends with the surrounding pane on
//! active/hover (Blender's "paper tab" continuation effect).

use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

impl BView {
    pub fn tab(icon: impl Into<String>, active: bool, action: BAction) -> Self {
        Self::Tab(BControl::new().with_icon(icon).with_active(active).with_action(action))
    }

    /// Icon-only tab with a hover tooltip shown to the right of the strip.
    pub fn tab_with_tooltip(
        icon: impl Into<String>,
        tooltip: impl Into<String>,
        active: bool,
        action: BAction,
    ) -> Self {
        Self::Tab(
            BControl::new()
                .with_icon(icon)
                .with_active(active)
                .with_action(action)
                .with_tooltip(tooltip),
        )
    }
    pub fn tab_labeled(
        icon: impl Into<String>,
        label: impl Into<String>,
        active: bool,
        action: BAction,
    ) -> Self {
        Self::Tab(
            BControl::new()
                .with_icon(icon)
                .with_label(label)
                .with_active(active)
                .with_action(action),
        )
    }
}
