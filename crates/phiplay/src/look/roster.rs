//! A team's sixteen players: who each is and what she looks like, drawn
//! from the looks and hairstyles by a mix seeded from the team, so a team
//! is sixteen different people and no two teams are the same sixteen.
//!
//! The mix is a permutation, not a dice roll: across a roster every look
//! appears twice and every hairstyle once, so a side on the field is
//! always a mixed one, and the same team always fields the same people.

use super::BLook;
use super::hair::BHair;
use crate::team::BTeam;

/// Players a roster holds — enough for soccer's eleven and its bench.
pub const ROSTER: usize = 16;

/// One player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPlayer {
    pub name: &'static str,
    pub number: u8,
    pub look: BLook,
    pub hair: BHair,
    /// Which of the look's three hair colours.
    pub hair_colour: u8,
}

/// First names, from every continent, one per roster place of each team
/// (sixteen teams × sixteen places draw from these 64 with a stride).
static NAMES: [&str; 64] = [
    "Ayanda", "Beatriz", "Chiara", "Dalia", "Esi", "Freya", "Gulnara", "Hana", "Ines", "Jae", "Kalani", "Lerato", "Mei", "Nadia", "Olu",
    "Priya", "Quinn", "Rosa", "Sipho", "Tala", "Uma", "Valentina", "Wanjiru", "Ximena", "Yara", "Zanele", "Aiko", "Bongi", "Camila",
    "Dumi", "Elif", "Fatou", "Grace", "Hiba", "Ivy", "Jade", "Kiri", "Leila", "Maya", "Nia", "Oksana", "Paloma", "Rania", "Saoirse",
    "Thandi", "Ula", "Vika", "Wren", "Xia", "Yasmin", "Zola", "Amara", "Buhle", "Chuma", "Dineo", "Eshe", "Folake", "Gabi", "Hloni",
    "Imani", "Jomo", "Kaya", "Lindiwe", "Mbali",
];

/// The team's roster, in shirt order.
pub fn roster(team: BTeam) -> [BPlayer; ROSTER] {
    let t = team.index();
    std::array::from_fn(|i| {
        // Strides coprime to 8, 16 and 64 make each mix a permutation.
        BPlayer {
            name: NAMES[(t * 16 + i * 5) % NAMES.len()],
            number: NUMBERS[(i + t) % NUMBERS.len()],
            look: BLook::ALL[(i * 3 + t) % 8],
            hair: BHair::ALL[(i * 7 + t * 5) % 16],
            hair_colour: ((i + t * 2) % 3) as u8,
        }
    })
}

/// The numbers a roster wears: the ones players ask for, no repeats.
static NUMBERS: [u8; 16] = [7, 11, 23, 3, 14, 30, 5, 9, 10, 4, 21, 8, 12, 15, 1, 33];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_roster_is_sixteen_different_people_and_every_team_its_own() {
        for team in BTeam::ALL {
            let r = roster(team);
            let mut numbers: Vec<_> = r.iter().map(|p| p.number).collect();
            numbers.sort();
            numbers.dedup();
            assert_eq!(numbers.len(), ROSTER, "no two players share a number");
            for look in BLook::ALL {
                assert_eq!(r.iter().filter(|p| p.look == look).count(), 2, "every look twice");
            }
            for hair in BHair::ALL {
                assert_eq!(r.iter().filter(|p| p.hair == hair).count(), 1, "every hairstyle once");
            }
        }
        // 256 places from 64 names: each name used by four teams, never
        // twice in one.
        for team in BTeam::ALL {
            let mut names: Vec<_> = roster(team).iter().map(|p| p.name).collect();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), ROSTER, "{:?}", team);
        }
        assert_eq!(roster(BTeam::Aurora), roster(BTeam::Aurora), "the same team, the same people");
        assert_ne!(roster(BTeam::Aurora)[0], roster(BTeam::Baobab)[0]);
    }
}
