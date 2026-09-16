//! Icon primitive — a named theme icon, no state, no action.

use crate::v2::view::BView;

impl BView {
    pub fn icon(name: impl Into<String>) -> Self {
        Self::Icon { name: name.into() }
    }
}
