//! The Play host's picker, Horizon-style: what the match is, with the
//! button that plays it; a row of the sets (sport, home, away, kit,
//! sneakers, theme, venue, surface, seats) to pick from; and the chosen
//! set's cards, each a picture painted from the option itself
//! (`phiplay::pick::thumb`), the picked one marked.
//!
//! Everything is looped from `phiplay::pick`'s table — a new set there is a
//! new row here with nothing written for it — and every card is a plain
//! write of the choice it would make (`PlayKey::Choice`), so picking is a
//! state change like any other and undoes like one.

pub mod thumb;

use crate::v2::state::{BKey as _, BState, OpKey, PlayKey};
use crate::v2::view::prim::{BAction, BCard, BColumn, BControl, BThumb};
use crate::v2::view::{BView, BViewItem};
use phiplay::BChoice;
use phiplay::pick::{BPick, decode, encode};

/// A card's picture edge, px.
pub const CARD: f32 = 112.0;

/// The match the state holds (the default demo match when none).
pub fn choice(state: &BState) -> BChoice {
    state.get(PlayKey::Choice).map(decode).unwrap_or_default()
}

/// The row on show (the sport's when none).
pub fn row(state: &BState) -> BPick {
    state.get(PlayKey::Row).and_then(BPick::by_slug).unwrap_or(BPick::Sport)
}

/// The match in one line: the sport, the sides, where.
pub fn summary(choice: &BChoice) -> String {
    format!(
        "{} · {} v {} · {} · {}",
        choice.sport.spec().label,
        choice.home.spec().name,
        choice.away.spec().name,
        choice.venue.spec().label,
        choice.theme.spec().label
    )
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BPlayItem;

impl BViewItem for BPlayItem {
    fn build(&self, state: &BState) -> BView {
        let choice = choice(state);
        let shown = row(state);
        let play = BControl::new().with_icon("play").with_label("Play Match").with_action(BAction::Set(OpKey::Play.path(), "start".into()));
        let head = BView::row(vec![BView::label(summary(&choice)), BView::spacer(), BView::Button(play)]);
        let rows: Vec<BView> = BPick::ALL
            .iter()
            .map(|&pick| {
                let spec = pick.spec();
                let picked = pick.option(pick.chosen(&choice)).label;
                BView::Button(
                    BControl::new()
                        .with_icon(spec.icon)
                        .with_label(spec.label)
                        .with_tooltip(format!("{}: {picked}", spec.label))
                        .with_active(pick == shown)
                        .with_action(BAction::Set(PlayKey::Row.path(), spec.slug.into())),
                )
            })
            .collect();
        let chosen = shown.chosen(&choice);
        let cards: Vec<BView> = (0..shown.count())
            .map(|i| {
                let mut next = choice;
                shown.choose(&mut next, i);
                let option = shown.option(i);
                BView::thumbnail(BThumb {
                    preview_path: thumb::key(shown, i, &choice),
                    icon: shown.spec().icon.into(),
                    label: option.label,
                    subtitle: option.caption,
                    size: CARD,
                    selected: i == chosen,
                    action: Some(BAction::Set(PlayKey::Choice.path(), encode(&next))),
                })
            })
            .collect();
        // One card, no headings: the bottom panel is short, so the match and
        // its button share a line, the sets the next, and the cards fill the
        // rest — as Horizon's picker is a strip of tabs over a shelf.
        let picker = BCard::headless().body(vec![head, BView::row(rows), BView::label(shown.spec().caption), BView::grid(cards)]);
        BView::from(BCard::column(BColumn::Body, vec![picker.into()]))
    }
}
