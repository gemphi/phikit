//! A venue: the stadium round the field. Eight of them, each after one of
//! the eight stadiums of the 2022 World Cup in Qatar — the tent, the
//! lantern bowl, the sails, the diamond, the containers, the woven cap, the
//! dunes, the arches — as shapes rather than copies: the idea of each
//! building, built from the same few parameters, in the theme's colours.
//!
//! A venue sizes itself to the field it holds (`BFieldSpec::floor_half`),
//! so the same stadium seats a basketball court or a soccer pitch. Its
//! façade carries a light show (`BLightShow`) at the theme's tempo.

pub mod seat;
mod set;

use seat::BSeat;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BVenue {
    Tent,
    Lantern,
    Sails,
    Diamond,
    Containers,
    Cap,
    Dunes,
    Arches,
}

/// The outer form the stands are wrapped in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BShell {
    /// A pitched tent: the rim rising to peaks at the corners, striped.
    Tent,
    /// A bowl that swells and narrows like a lantern, latticed in triangles.
    Lantern,
    /// Overlapping curved shells, each a sail.
    Sails,
    /// Faceted triangles that tilt in and out.
    Diamond,
    /// Boxes stacked in colour blocks.
    Containers,
    /// A drum wrapped in a woven lattice, open at the top in a ring.
    Cap,
    /// Ribbons that rise and fall round the rim like dunes.
    Dunes,
    /// A plain bowl spanned by two crossing arches.
    Arches,
}

/// What covers the stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BRoof {
    /// Open to the sky.
    Open,
    /// A ring over the stands, open over the field.
    Ring,
    /// A roof over everything (an indoor arena).
    Closed,
}

/// How the façade's lights move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BLightShow {
    /// A band of light running round the rim.
    Chase,
    /// Everything brightening and dimming on the beat.
    Pulse,
    /// A wave rising and falling round the bowl.
    Wave,
    /// Panels flicking on at random and fading.
    Sparkle,
    /// A beam sweeping round, panels lighting as it passes.
    Sweep,
    /// A slow breath of the whole façade.
    Breathe,
    /// Two chases crossing in opposite directions.
    Weave,
    /// Rings of light orbiting up and down.
    Orbit,
}

#[derive(Clone, Copy, Debug)]
pub struct BVenueSpec {
    pub label: &'static str,
    /// What it is after, one line.
    pub after: &'static str,
    pub shell: BShell,
    pub roof: BRoof,
    pub show: BLightShow,
    /// Terraces of seats.
    pub tiers: u8,
    /// The façade's light panels round the rim.
    pub panels: u16,
    /// The shell's height over the top terrace, metres.
    pub rise: f32,
    pub seat: BSeat,
}

impl BVenue {
    pub const ALL: [BVenue; 8] =
        [BVenue::Tent, BVenue::Lantern, BVenue::Sails, BVenue::Diamond, BVenue::Containers, BVenue::Cap, BVenue::Dunes, BVenue::Arches];
}

impl BLightShow {
    /// The light a façade panel at `at` round the rim (0–1) and `up` its
    /// height (0–1) shows `beat` beats in, 0–1. Every show is a pure
    /// function of these, so any frame can be drawn without the ones before.
    pub fn level(self, at: f32, up: f32, beat: f32) -> f32 {
        use std::f32::consts::TAU;
        let wave = |x: f32| 0.5 + 0.5 * (TAU * x).cos();
        // A band of light centred where `x` is 0.5 (mod 1): 1 at its centre,
        // falling to 0 `width` (a share of the round) either side.
        let near = |x: f32, width: f32| (1.0 - (x.rem_euclid(1.0) - 0.5).abs() / width).max(0.0).powi(2);
        match self {
            BLightShow::Chase => near(at - beat / 8.0 + 0.5, 0.15),
            BLightShow::Pulse => 0.2 + 0.8 * wave(beat).powi(3),
            BLightShow::Wave => wave(at * 3.0 - beat / 4.0) * (0.4 + 0.6 * up),
            BLightShow::Sparkle => {
                let seed = ((at * 997.0).floor() * 13.0 + (up * 31.0).floor() * 7.0).sin() * 43758.545;
                let phase = (seed - seed.floor() + beat / 2.0).fract();
                (1.0 - phase * 4.0).max(0.0)
            }
            BLightShow::Sweep => near(at - beat / 16.0 + 0.5, 0.08) * 0.8 + 0.2 * up,
            BLightShow::Breathe => 0.3 + 0.7 * wave(beat / 8.0),
            BLightShow::Weave => near(at - beat / 8.0 + 0.5, 0.1).max(near(at + beat / 8.0 + 0.5, 0.1)),
            BLightShow::Orbit => near(up - beat / 4.0 + 0.5, 0.2) * wave(at * 2.0 + beat / 8.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_venue_is_its_own_shape_and_its_show_stays_in_range() {
        let mut shells: Vec<_> = BVenue::ALL.iter().map(|v| v.spec().shell).collect();
        shells.dedup();
        assert_eq!(shells.len(), 8);
        for venue in BVenue::ALL {
            let s = venue.spec();
            assert!(s.tiers >= 4 && s.panels >= 24 && s.rise > 0.0, "{}", s.label);
            let mut lit = 0.0;
            for i in 0..64 {
                for beat in 0..16 {
                    let l = s.show.level(i as f32 / 64.0, 0.5, beat as f32 * 0.37);
                    assert!((0.0..=1.0).contains(&l), "{}: level {l}", s.label);
                    lit += l;
                }
            }
            assert!(lit > 0.0, "{}: a show that never lights", s.label);
        }
    }
}
