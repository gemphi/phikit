//! Button primitive — an optionally-labeled, optionally-dispatching control.

use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

impl BView {
    pub fn button(icon: impl Into<String>) -> Self {
        Self::Button(BControl::new().with_icon(icon))
    }
    pub fn button_action(
        icon: impl Into<String>,
        label: impl Into<String>,
        action: BAction,
    ) -> Self {
        Self::Button(
            BControl::new()
                .with_icon(icon)
                .with_label(label)
                .with_action(action),
        )
    }
    pub fn button_hover(
        icon: impl Into<String>,
        label: impl Into<String>,
        action: BAction,
        hover: BAction,
    ) -> Self {
        Self::Button(
            BControl::new()
                .with_icon(icon)
                .with_label(label)
                .with_action(action)
                .with_hover(hover),
        )
    }
}
