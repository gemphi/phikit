//! The seeds — one mid-tone per theme, and the light-mode seed each is
//! lifted to — and the eleven ladders in their two modes.

use super::BShadeLadder;

/// The sphere's mid-tone: the default theme's seed.
pub const SPHERE_SEED: [u8; 3] = [0x3B, 0x46, 0x57];
/// Blender's panel grey `#3D3D3D`, for the Graphite theme.
pub const GRAPHITE_SEED: [u8; 3] = [0x3D, 0x3D, 0x3D];
/// Tokyo Night's highlight blue, for the Tokyo theme.
pub const TOKYO_SEED: [u8; 3] = [0x3B, 0x42, 0x61];
/// Black and grey: a near-black and a mid grey; Grey's light mode is the
/// silver the Tesla dashboard reads in (`BShadeLadder::light`).
pub const ONYX_SEED: [u8; 3] = [0x2A, 0x2A, 0x2A];
pub const GREY_SEED: [u8; 3] = [0x7A, 0x7A, 0x7A];
pub const GREY_LIGHT_SEED: [u8; 3] = [0xC4, 0xC6, 0xC9];
/// Apple's dark and light window greys — one theme, two modes.
pub const APPLE_SEED: [u8; 3] = [0x38, 0x38, 0x3A];
pub const APPLE_LIGHT_SEED: [u8; 3] = [0xCB, 0xCB, 0xCD];
/// The `phi` appkit variants' surfaces (`appkit/src/theme/variants`), each
/// brought up to a canvas seed: Swift's warm dark, Barbie's pink-tinted
/// dark, Fenty's earth, Cyberpunk's purple-black, Chanel's mushroom forest.
pub const SWIFT_SEED: [u8; 3] = [0x3F, 0x35, 0x32];
pub const BARBIE_SEED: [u8; 3] = [0x4D, 0x2A, 0x3E];
pub const FENTY_SEED: [u8; 3] = [0x4E, 0x3F, 0x33];
pub const CYBERPUNK_SEED: [u8; 3] = [0x2C, 0x1C, 0x42];
pub const CHANEL_SEED: [u8; 3] = [0x3E, 0x3A, 0x30];

/// How far a dark seed is lifted toward white to seed the same theme's
/// light mode: three quarters of the way, so the `#3B4657` blue-grey seeds
/// a `#CED1D5` one — the silver Grey's light mode reads in — with its hue kept.
pub const LIFT_TO_LIGHT: u32 = 75;

/// The light-mode seed of a dark seed: lifted `LIFT_TO_LIGHT` percent
/// toward white.
pub const fn lifted(seed: [u8; 3]) -> [u8; 3] {
    const fn lift(channel: u8) -> u8 {
        ((channel as u32 * (100 - LIFT_TO_LIGHT) + 255 * LIFT_TO_LIGHT + 50) / 100) as u8
    }
    [lift(seed[0]), lift(seed[1]), lift(seed[2])]
}

impl BShadeLadder {
    pub const fn dark(seed: [u8; 3]) -> Self {
        Self { seed, light: false }
    }
    pub const fn light(seed: [u8; 3]) -> Self {
        Self { seed, light: true }
    }
}

pub const SPHERE: BShadeLadder = BShadeLadder::dark(SPHERE_SEED);
pub const GRAPHITE: BShadeLadder = BShadeLadder::dark(GRAPHITE_SEED);
pub const TOKYO: BShadeLadder = BShadeLadder::dark(TOKYO_SEED);
pub const ONYX: BShadeLadder = BShadeLadder::dark(ONYX_SEED);
pub const GREY: BShadeLadder = BShadeLadder::dark(GREY_SEED);
pub const APPLE: BShadeLadder = BShadeLadder::dark(APPLE_SEED);
pub const SWIFT: BShadeLadder = BShadeLadder::dark(SWIFT_SEED);
pub const BARBIE: BShadeLadder = BShadeLadder::dark(BARBIE_SEED);
pub const FENTY: BShadeLadder = BShadeLadder::dark(FENTY_SEED);
pub const CYBERPUNK: BShadeLadder = BShadeLadder::dark(CYBERPUNK_SEED);
pub const CHANEL: BShadeLadder = BShadeLadder::dark(CHANEL_SEED);
// Every theme's light mode: the same eighteen steps the other way up, on a
// seed lifted toward white — or, for Grey and Apple, their own light seed.
pub const SPHERE_LIGHT: BShadeLadder = BShadeLadder::light(lifted(SPHERE_SEED));
pub const GRAPHITE_LIGHT: BShadeLadder = BShadeLadder::light(lifted(GRAPHITE_SEED));
pub const TOKYO_LIGHT: BShadeLadder = BShadeLadder::light(lifted(TOKYO_SEED));
pub const ONYX_LIGHT: BShadeLadder = BShadeLadder::light(lifted(ONYX_SEED));
pub const GREY_LIGHT: BShadeLadder = BShadeLadder::light(GREY_LIGHT_SEED);
pub const APPLE_LIGHT: BShadeLadder = BShadeLadder::light(APPLE_LIGHT_SEED);
pub const SWIFT_LIGHT: BShadeLadder = BShadeLadder::light(lifted(SWIFT_SEED));
pub const BARBIE_LIGHT: BShadeLadder = BShadeLadder::light(lifted(BARBIE_SEED));
pub const FENTY_LIGHT: BShadeLadder = BShadeLadder::light(lifted(FENTY_SEED));
pub const CYBERPUNK_LIGHT: BShadeLadder = BShadeLadder::light(lifted(CYBERPUNK_SEED));
pub const CHANEL_LIGHT: BShadeLadder = BShadeLadder::light(lifted(CHANEL_SEED));
