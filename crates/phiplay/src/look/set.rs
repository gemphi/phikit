//! The eight looks.

use super::{BLook, BLookSpec};
use crate::colour::Rgb;

impl BLook {
    pub fn spec(self) -> &'static BLookSpec {
        match self {
            BLook::Porcelain => &PORCELAIN,
            BLook::Ivory => &IVORY,
            BLook::Sand => &SAND,
            BLook::Olive => &OLIVE,
            BLook::Amber => &AMBER,
            BLook::Bronze => &BRONZE,
            BLook::Umber => &UMBER,
            BLook::Ebony => &EBONY,
        }
    }
}

const fn look(label: &'static str, skin: u32, hair: [u32; 3], height: f32) -> BLookSpec {
    BLookSpec { label, skin: Rgb::hex(skin), hair: [Rgb::hex(hair[0]), Rgb::hex(hair[1]), Rgb::hex(hair[2])], height }
}

static PORCELAIN: BLookSpec = look("Porcelain", 0xF3D9C7, [0xD9B46A, 0x8C4A2F, 0x2B1D16], 0.98);
static IVORY: BLookSpec = look("Ivory", 0xE9C3A6, [0x6B4226, 0xB5652B, 0x1E1612], 1.02);
static SAND: BLookSpec = look("Sand", 0xD9A982, [0x3A2618, 0x5A3A22, 0x14100E], 0.96);
static OLIVE: BLookSpec = look("Olive", 0xC18E62, [0x241912, 0x3B2A1E, 0x0F0C0A], 1.0);
static AMBER: BLookSpec = look("Amber", 0xA7704A, [0x1A1310, 0x3A2416, 0x6B3B1F], 1.04);
static BRONZE: BLookSpec = look("Bronze", 0x8C5A3A, [0x120E0C, 0x2A1B12, 0x7A4A2A], 0.97);
static UMBER: BLookSpec = look("Umber", 0x6B4029, [0x0E0B0A, 0x24170F, 0xB0703A], 1.06);
static EBONY: BLookSpec = look("Ebony", 0x4A2C1D, [0x0B0908, 0x1C130E, 0xD4A04A], 1.03);
