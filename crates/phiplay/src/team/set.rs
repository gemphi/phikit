//! The sixteen teams.

use super::{BTeam, BTeamSpec};
use crate::colour::Rgb;

impl BTeam {
    pub fn spec(self) -> &'static BTeamSpec {
        match self {
            BTeam::Aurora => &AURORA,
            BTeam::Baobab => &BAOBAB,
            BTeam::Coral => &CORAL,
            BTeam::Dune => &DUNE,
            BTeam::Ember => &EMBER,
            BTeam::Fjord => &FJORD,
            BTeam::Granite => &GRANITE,
            BTeam::Harbour => &HARBOUR,
            BTeam::Indigo => &INDIGO,
            BTeam::Jacaranda => &JACARANDA,
            BTeam::Kestrel => &KESTREL,
            BTeam::Lumen => &LUMEN,
            BTeam::Monsoon => &MONSOON,
            BTeam::Nebula => &NEBULA,
            BTeam::Obsidian => &OBSIDIAN,
            BTeam::Protea => &PROTEA,
        }
    }
}

const fn team(name: &'static str, code: &'static str, home: u32, away: u32, trim: u32) -> BTeamSpec {
    BTeamSpec { name, code, home: Rgb::hex(home), away: Rgb::hex(away), trim: Rgb::hex(trim) }
}

static AURORA: BTeamSpec = team("Aurora", "AUR", 0x6C4AB6, 0xE9F7EF, 0x7BE0B5);
static BAOBAB: BTeamSpec = team("Baobab", "BAO", 0x6B6A2E, 0xF3E9D2, 0xD9A441);
static CORAL: BTeamSpec = team("Coral", "COR", 0xF26B5B, 0x10454F, 0x2EC4B6);
static DUNE: BTeamSpec = team("Dune", "DUN", 0xD9B77E, 0x5C1A2B, 0x8A2C3F);
static EMBER: BTeamSpec = team("Ember", "EMB", 0xC8262E, 0xF4F1EC, 0x2B2D33);
static FJORD: BTeamSpec = team("Fjord", "FJO", 0x9CC9E8, 0x0F2A4A, 0x1E5A8C);
static GRANITE: BTeamSpec = team("Granite", "GRA", 0x5F6670, 0xF1F3F0, 0xC6F23A);
static HARBOUR: BTeamSpec = team("Harbour", "HAR", 0x1B2F5E, 0xFBF6EE, 0xF07167);
static INDIGO: BTeamSpec = team("Indigo", "IND", 0x2E2A86, 0xFFF4D6, 0xF2C230);
static JACARANDA: BTeamSpec = team("Jacaranda", "JAC", 0x8E5BBF, 0x1F3D2B, 0x7FC97F);
static KESTREL: BTeamSpec = team("Kestrel", "KES", 0x8A5A3B, 0xEAF4FB, 0x7DB9E8);
static LUMEN: BTeamSpec = team("Lumen", "LUM", 0xF5D30F, 0x15161A, 0x15161A);
static MONSOON: BTeamSpec = team("Monsoon", "MON", 0x3F5466, 0xE6FBFF, 0x34D1E0);
static NEBULA: BTeamSpec = team("Nebula", "NEB", 0xC2185B, 0x0E1A3A, 0x7A8CFF);
static OBSIDIAN: BTeamSpec = team("Obsidian", "OBS", 0x15151A, 0xE4E6EA, 0xA8ADB7);
static PROTEA: BTeamSpec = team("Protea", "PRO", 0xF2A7B8, 0x16402E, 0xC2185B);
