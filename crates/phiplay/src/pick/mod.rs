//! The picker: the match's choice as rows of cards, one row per set —
//! the sport, the two teams, the kit, the shoes, the theme, the venue, the
//! surface and the seats — and a card per option, each with a picture
//! painted from the option's own numbers (`thumb`). Phiber's Play host
//! loops over these; nothing here knows a window.
//!
//! The rows are a declared table (`set.rs`); what a row holds and how a
//! pick changes the choice is here, one match arm per row, so a new set is
//! a new row and a new arm, not a new screen.

mod set;
pub mod thumb;

use crate::choice::BChoice;
use crate::kit::sneaker::BSneaker;
use crate::kit::uniform::BUniform;
use crate::sport::BSport;
use crate::team::BTeam;
use crate::theme::BTheme;
use crate::turf::BTurf;
use crate::venue::BVenue;
use crate::venue::seat::BSeat;

/// A row of the picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPick {
    Sport,
    Home,
    Away,
    Uniform,
    Sneaker,
    Theme,
    Venue,
    Turf,
    Seat,
}

/// What a row is called and drawn with.
#[derive(Clone, Copy, Debug)]
pub struct BPickSpec {
    pub label: &'static str,
    /// The state key's tail and the host's id for the row.
    pub slug: &'static str,
    pub icon: &'static str,
    /// One line under the row's title.
    pub caption: &'static str,
}

/// A card: its title and the line under it.
#[derive(Clone, Debug, PartialEq)]
pub struct BOption {
    pub label: String,
    pub caption: String,
}

impl BPick {
    pub fn by_slug(slug: &str) -> Option<BPick> {
        BPick::ALL.into_iter().find(|p| p.spec().slug == slug)
    }

    /// How many cards the row holds.
    pub fn count(self) -> usize {
        match self {
            BPick::Sport => BSport::ALL.len(),
            BPick::Home | BPick::Away => BTeam::ALL.len(),
            BPick::Uniform => BUniform::ALL.len(),
            BPick::Sneaker => BSneaker::ALL.len(),
            BPick::Theme => BTheme::ALL.len(),
            BPick::Venue => BVenue::ALL.len(),
            BPick::Turf => BTurf::ALL.len(),
            BPick::Seat => BSeat::ALL.len(),
        }
    }

    /// The row's `i`th card.
    pub fn option(self, i: usize) -> BOption {
        let (label, caption): (&str, String) = match self {
            BPick::Sport => {
                let s = BSport::ALL[i].spec();
                (s.label, format!("{} a side · {}", s.side, s.body))
            }
            BPick::Home | BPick::Away => {
                let t = BTeam::ALL[i].spec();
                (t.name, t.code.to_string())
            }
            BPick::Uniform => (BUniform::ALL[i].spec().label, String::new()),
            BPick::Sneaker => {
                let s = BSneaker::ALL[i].spec();
                (s.label, format!("{:.0} cm collar", s.collar * 100.0))
            }
            BPick::Theme => {
                let t = BTheme::ALL[i].spec();
                (t.label, t.caption.to_string())
            }
            BPick::Venue => {
                let v = BVenue::ALL[i].spec();
                (v.label, v.after.to_string())
            }
            BPick::Turf => (BTurf::ALL[i].spec().label, String::new()),
            BPick::Seat => (BSeat::ALL[i].spec().label, String::new()),
        };
        BOption { label: label.to_string(), caption }
    }

    /// The card `choice` has picked in this row.
    pub fn chosen(self, choice: &BChoice) -> usize {
        let at = |found: Option<usize>| found.unwrap_or(0);
        match self {
            BPick::Sport => at(BSport::ALL.iter().position(|&s| s == choice.sport)),
            BPick::Home => at(BTeam::ALL.iter().position(|&t| t == choice.home)),
            BPick::Away => at(BTeam::ALL.iter().position(|&t| t == choice.away)),
            BPick::Uniform => at(BUniform::ALL.iter().position(|&u| u == choice.uniform)),
            BPick::Sneaker => at(BSneaker::ALL.iter().position(|&s| s == choice.sneaker)),
            BPick::Theme => at(BTheme::ALL.iter().position(|&t| t == choice.theme)),
            BPick::Venue => at(BVenue::ALL.iter().position(|&v| v == choice.venue)),
            BPick::Turf => at(BTurf::ALL.iter().position(|&t| t == choice.turf())),
            BPick::Seat => at(BSeat::ALL.iter().position(|&s| s == choice.seat())),
        }
    }

    /// `choice` with card `i` of this row picked. A team picked for both
    /// sides swaps them, so a match is always two teams.
    ///
    /// The table's order is the order rows depend in: the sport before the
    /// surface, the venue before the seats (`set.rs`).
    pub fn choose(self, choice: &mut BChoice, i: usize) {
        let i = i.min(self.count() - 1);
        match self {
            // A new sport lays its own surface, a new venue fits its own
            // seats, until those rows are picked again.
            BPick::Sport => {
                choice.sport = BSport::ALL[i];
                choice.turf = None;
            }
            BPick::Home => {
                let team = BTeam::ALL[i];
                if team == choice.away {
                    choice.away = choice.home;
                }
                choice.home = team;
            }
            BPick::Away => {
                let team = BTeam::ALL[i];
                if team == choice.home {
                    choice.home = choice.away;
                }
                choice.away = team;
            }
            BPick::Uniform => choice.uniform = BUniform::ALL[i],
            BPick::Sneaker => choice.sneaker = BSneaker::ALL[i],
            BPick::Theme => choice.theme = BTheme::ALL[i],
            BPick::Venue => {
                choice.venue = BVenue::ALL[i];
                choice.seat = None;
            }
            BPick::Turf => choice.turf = Some(BTurf::ALL[i]),
            BPick::Seat => choice.seat = Some(BSeat::ALL[i]),
        }
    }
}

/// `choice` written as one line — each row's slug and its card — and read
/// back: how the host keeps a choice in its state.
pub fn encode(choice: &BChoice) -> String {
    BPick::ALL.iter().map(|p| format!("{}={}", p.spec().slug, p.chosen(choice))).collect::<Vec<_>>().join(";")
}

pub fn decode(text: &str) -> BChoice {
    let mut choice = BChoice::default();
    for (slug, i) in text.split(';').filter_map(|kv| kv.split_once('=')) {
        if let (Some(pick), Ok(i)) = (BPick::by_slug(slug), i.parse::<usize>()) {
            pick.choose(&mut choice, i);
        }
    }
    choice
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_offers_its_set_and_a_pick_round_trips() {
        let mut choice = BChoice::default();
        for pick in BPick::ALL {
            assert!(pick.count() >= 5, "{pick:?}");
            for i in 0..pick.count() {
                assert!(!pick.option(i).label.is_empty());
                pick.choose(&mut choice, i);
                assert_eq!(pick.chosen(&choice), i, "{pick:?} {i}");
                assert_ne!(choice.home, choice.away, "two teams");
            }
        }
        assert_eq!(decode(&encode(&choice)), choice);
        assert_eq!(decode(""), BChoice::default());
    }
}
