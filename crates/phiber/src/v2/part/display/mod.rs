//! How a host shows its controls: icons with their labels, or icons alone.
//!
//! Every view keeps one setting, `<host>.display.labels` — `auto`, `icons`
//! or `labels` — so a header can be tweaked like any other property: icons
//! alone to save room, labels to read, or auto (the default), which shows
//! labels while they fit and drops to icons when the view narrows, as a
//! page drops to icons under a media query. What does not fit even as icons
//! folds into a menu (`part::filter::folded`).

use crate::v2::host::BHostType;
use crate::v2::state::{BPath, BState};

/// The three answers, in the order a View menu lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BLabels {
    Auto,
    Icons,
    Labels,
}

impl BLabels {
    pub const ALL: [BLabels; 3] = [Self::Auto, Self::Icons, Self::Labels];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Icons => "icons",
            Self::Labels => "labels",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Icons => "Icons Only",
            Self::Labels => "Icons and Labels",
        }
    }

    /// `<host>.display.labels`.
    pub fn key(host: BHostType) -> BPath {
        BPath::owned(vec![host.slug(), "display", "labels"])
    }

    /// The host's setting, Auto until one is picked.
    pub fn of(s: &BState, host: BHostType) -> Self {
        let slug = s.get(&Self::key(host)).unwrap_or("");
        Self::ALL.into_iter().find(|labels| labels.slug() == slug).unwrap_or(Self::Auto)
    }

    /// Whether a control shows its label: as set, or — on Auto — when
    /// `fits`, the caller's own measure of the room it has.
    pub fn shown(s: &BState, host: BHostType, fits: bool) -> bool {
        match Self::of(s, host) {
            Self::Auto => fits,
            Self::Icons => false,
            Self::Labels => true,
        }
    }
}

/// A View menu's "Labels" entries for `host`: the three answers, the one in
/// force ticked.
pub fn menu(s: &BState, host: BHostType) -> Vec<crate::v2::view::BView> {
    use crate::v2::view::BView;
    use crate::v2::view::prim::{BAction, BControl};
    let now = BLabels::of(s, host);
    BLabels::ALL
        .into_iter()
        .map(|labels| {
            BView::MenuItem(
                BControl::new()
                    .with_icon(if labels == now { "check" } else { "" })
                    .with_active(labels == now)
                    .with_label(labels.label())
                    .with_action(BAction::Set(BLabels::key(host), labels.slug().to_string())),
            )
        })
        .collect()
}

/// The top bar's space tabs: Window ▸ Space Tabs.
pub fn topbar_menu(s: &BState) -> Vec<crate::v2::view::BView> {
    menu(s, BHostType::Topbar)
}

/// The window's width, when the window has reported it.
pub fn window_width(s: &BState) -> Option<f32> {
    use crate::v2::state::AppKey;
    s.get(AppKey::WindowWidth).and_then(|w| w.parse::<f32>().ok())
}
