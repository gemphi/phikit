//! How the palette lays its results out: a line each, or a card each at
//! three sizes — the palette is a host like any other, and every host can
//! show its items as a list or as cards.

use crate::v2::state::BKey as _;
use crate::v2::state::{AppKey, BState};
use crate::v2::view::prim::{BAction, BControl};
use crate::v2::view::BView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BFindLayout {
    Line,
    Medium,
    Large,
    Huge,
}

impl BFindLayout {
    pub const ALL: [BFindLayout; 4] = [Self::Line, Self::Medium, Self::Large, Self::Huge];

    pub fn label(self) -> &'static str {
        match self {
            Self::Line => "Lines",
            Self::Medium => "Medium Cards",
            Self::Large => "Large Cards",
            Self::Huge => "Extra Large Cards",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::Huge => "xl",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Line => "list",
            Self::Medium => "grid",
            Self::Large => "grid_2x2",
            Self::Huge => "square",
        }
    }

    /// A card's edge in px, or `None` for a line.
    pub fn card(self) -> Option<f32> {
        match self {
            Self::Line => None,
            Self::Medium => Some(64.0),
            Self::Large => Some(96.0),
            Self::Huge => Some(160.0),
        }
    }

    /// The layout in force: what `command.layout` names, else lines.
    pub fn of(s: &BState) -> Self {
        let slug = s.get(AppKey::CommandLayout).unwrap_or("");
        Self::ALL.into_iter().find(|l| l.slug() == slug).unwrap_or(Self::Line)
    }

    /// The four as icon toggles, the one in force lit.
    pub fn toggles(s: &BState) -> Vec<BView> {
        let open = Self::of(s);
        Self::ALL
            .into_iter()
            .map(|l| {
                BView::Button(
                    BControl::new()
                        .with_icon(l.icon())
                        .with_tooltip(l.label())
                        .with_active(l == open)
                        .with_action(BAction::Set(AppKey::CommandLayout.path(), l.slug().to_string())),
                )
            })
            .collect()
    }
}
