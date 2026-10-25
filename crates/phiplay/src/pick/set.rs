//! The picker's rows, in the order they are shown and decoded — a row a
//! later one depends on comes first (the sport before the surface it lays,
//! the venue before the seats it fits).

use super::{BPick, BPickSpec};

const fn row(label: &'static str, slug: &'static str, icon: &'static str, caption: &'static str) -> BPickSpec {
    BPickSpec { label, slug, icon, caption }
}

impl BPick {
    pub const ALL: [BPick; 9] = [BPick::Sport, BPick::Home, BPick::Away, BPick::Uniform, BPick::Sneaker, BPick::Theme, BPick::Venue, BPick::Turf, BPick::Seat];

    pub fn spec(self) -> &'static BPickSpec {
        match self {
            BPick::Sport => &SPORT,
            BPick::Home => &HOME,
            BPick::Away => &AWAY,
            BPick::Uniform => &UNIFORM,
            BPick::Sneaker => &SNEAKER,
            BPick::Theme => &THEME,
            BPick::Venue => &VENUE,
            BPick::Turf => &TURF,
            BPick::Seat => &SEAT,
        }
    }
}

static SPORT: BPickSpec = row("Sport", "sport", "ball", "Five games, each to its rulebook");
static HOME: BPickSpec = row("Home", "home", "home", "The side attacking east");
static AWAY: BPickSpec = row("Away", "away", "world", "The side attacking west");
static UNIFORM: BPickSpec = row("Kit", "uniform", "shirt", "Cut to the sport, in each side's colours");
static SNEAKER: BPickSpec = row("Sneakers", "sneaker", "shoe", "Every collar above the ankle");
static THEME: BPickSpec = row("Theme", "theme", "palette", "Palette, light and the show's tempo");
static VENUE: BPickSpec = row("Venue", "venue", "stadium", "The bowl, its shell and its lights");
static TURF: BPickSpec = row("Surface", "turf", "texture", "What the field is laid in");
static SEAT: BPickSpec = row("Seats", "seat", "seat", "The backs every stand is fitted with");
