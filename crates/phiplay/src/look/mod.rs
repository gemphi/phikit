//! How a player looks: skin, hair colour and build (`BLook`, eight of them
//! across the range of human skin), and the hairstyle (`hair`, sixteen).
//! A roster mixes them (`roster`), so a team is sixteen different people,
//! not one person sixteen times.

pub mod hair;
pub mod roster;
mod set;

use crate::colour::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BLook {
    Porcelain,
    Ivory,
    Sand,
    Olive,
    Amber,
    Bronze,
    Umber,
    Ebony,
}

#[derive(Clone, Copy, Debug)]
pub struct BLookSpec {
    pub label: &'static str,
    pub skin: Rgb,
    /// The hair colours this look is drawn with, one picked per player.
    pub hair: [Rgb; 3],
    /// Height as a share of the base figure's (1.68 m).
    pub height: f32,
}

impl BLook {
    pub const ALL: [BLook; 8] = [BLook::Porcelain, BLook::Ivory, BLook::Sand, BLook::Olive, BLook::Amber, BLook::Bronze, BLook::Umber, BLook::Ebony];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_looks_run_light_to_dark() {
        let lum: Vec<f32> = BLook::ALL.iter().map(|l| l.spec().skin.luminance()).collect();
        assert!(lum.windows(2).all(|w| w[0] > w[1]), "{lum:?}");
        for look in BLook::ALL {
            assert!((0.94..=1.08).contains(&look.spec().height));
        }
    }
}
