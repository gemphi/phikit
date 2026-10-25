//! A team: a name, a three-letter code and three colours — home, away and
//! trim. Sixteen of them, A to P, each named for something that is never a
//! real club, so no badge is borrowed. Numbers are printed in white or ink,
//! whichever reads on the shirt (`BTeamSpec::number_ink`).

mod set;

use crate::colour::{INK, Rgb, WHITE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BTeam {
    Aurora,
    Baobab,
    Coral,
    Dune,
    Ember,
    Fjord,
    Granite,
    Harbour,
    Indigo,
    Jacaranda,
    Kestrel,
    Lumen,
    Monsoon,
    Nebula,
    Obsidian,
    Protea,
}

#[derive(Clone, Copy, Debug)]
pub struct BTeamSpec {
    pub name: &'static str,
    pub code: &'static str,
    pub home: Rgb,
    pub away: Rgb,
    pub trim: Rgb,
}

impl BTeam {
    pub const ALL: [BTeam; 16] = [
        BTeam::Aurora,
        BTeam::Baobab,
        BTeam::Coral,
        BTeam::Dune,
        BTeam::Ember,
        BTeam::Fjord,
        BTeam::Granite,
        BTeam::Harbour,
        BTeam::Indigo,
        BTeam::Jacaranda,
        BTeam::Kestrel,
        BTeam::Lumen,
        BTeam::Monsoon,
        BTeam::Nebula,
        BTeam::Obsidian,
        BTeam::Protea,
    ];

    /// Its place in `ALL`, which seeds everything drawn from it.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }
}

impl BTeamSpec {
    /// The shirt a side wears: home, or away when the other side's home
    /// colour is too close to its own to tell apart.
    pub fn shirt(&self, other: &BTeamSpec) -> Rgb {
        if distance(self.home, other.home) < CLASH { self.away } else { self.home }
    }

    /// White or ink, whichever reads on `shirt`.
    pub fn number_ink(shirt: Rgb) -> Rgb {
        if shirt.luminance() > 0.3 { INK } else { WHITE }
    }
}

/// How close two shirts may be before the visitors change: a distance in
/// sRGB, where 0.25 is about the gap between scarlet and orange.
const CLASH: f32 = 0.25;

fn distance(a: Rgb, b: Rgb) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2) + (a.2 - b.2).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixteen_teams_with_their_own_codes_and_no_two_shirts_alike() {
        let mut codes: Vec<_> = BTeam::ALL.iter().map(|t| t.spec().code).collect();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), 16);
        // Every pairing puts two shirts on the floor that can be told apart.
        for a in BTeam::ALL {
            for b in BTeam::ALL {
                if a == b {
                    continue;
                }
                let (sa, sb) = (a.spec().home, b.spec().shirt(a.spec()));
                assert!(distance(sa, sb) >= CLASH, "{} v {}: shirts too alike", a.spec().name, b.spec().name);
            }
        }
    }
}
