//! The forty-eight designs, four a set. Names are the house's own.

use super::{BBand, BBandKind as K, BColours, BExtra, BFootwearSet as S, BFootwearSpec, BHeel, BReach, BToe, BTopline, BTread};
use crate::colour::Rgb;

const IVORY: Rgb = Rgb::hex(0xEDE6DA);
const BEIGE: Rgb = Rgb::hex(0xE4D3BA);
const CLAY: Rgb = Rgb::hex(0xD8CDBD);
const GOLD: Rgb = Rgb::hex(0xC9A24A);
const NAVY: Rgb = Rgb::hex(0x16224C);
const MIDNIGHT: Rgb = Rgb::hex(0x0D1433);
const MAGENTA: Rgb = Rgb::hex(0xE03BD6);
const ORANGE: Rgb = Rgb::hex(0xEE8A1E);
const TAN: Rgb = Rgb::hex(0xA8693A);
const CREAM: Rgb = Rgb::hex(0xF2E6CC);
const BLACK: Rgb = Rgb::hex(0x17171B);
const SILVER: Rgb = Rgb::hex(0xB8BCC4);
const BURGUNDY: Rgb = Rgb::hex(0x5E1420);
const PLUM: Rgb = Rgb::hex(0x2E1824);
const OLIVE: Rgb = Rgb::hex(0x5B6234);
const BROWN: Rgb = Rgb::hex(0x5A3A22);
const BLUSH: Rgb = Rgb::hex(0xEBC2C7);
const CORAL: Rgb = Rgb::hex(0xE3725C);
const RUBBER: Rgb = Rgb::hex(0x24201E);
const CORK: Rgb = Rgb::hex(0xC49A6C);
const LINING: Rgb = Rgb::hex(0xB89A7A);

const fn c(upper: Rgb, trim: Rgb, panel: Rgb, sole: Rgb) -> BColours {
    BColours { upper, trim, panel, sole, lining: LINING }
}

// ── The Cut-Off Series ────────────────────────────────────────────────────

/// The tall boot's lines as the sheet draws them, back to front: a stitched
/// line under the rim, a scalloped band falling to the front, a wave, a
/// second scalloped band, the ankle's band rising to the front. A shorter
/// cut keeps the ones under its rim.
const SHEET: &[BBand] = &[
    BBand { back: 1.62, front: 1.80, kind: K::Line },
    BBand { back: 1.57, front: 1.45, kind: K::Scallop },
    BBand { back: 1.02, front: 1.12, kind: K::Wave },
    BBand { back: 0.89, front: 0.79, kind: K::Scallop },
    BBand { back: 0.35, front: 0.45, kind: K::Scallop },
];

/// The house's four colourways: the classic two-tone, the sheet's own pink
/// on navy, quilted black, and blush with plum.
const CLASSIQUE: BColours = c(BEIGE, GOLD, BLACK, BLACK);
const NUIT: BColours = c(NAVY, MAGENTA, MIDNIGHT, MIDNIGHT);
const MATELASSE: BColours = c(BLACK, GOLD, BLACK, BLACK);
const ROSE: BColours = c(BLUSH, IVORY, PLUM, PLUM);

const fn cut(label: &'static str, slug: &'static str, set: S, reach: BReach, zip: bool, colours: BColours, quilt: bool, tread: BTread) -> BFootwearSpec {
    BFootwearSpec {
        label,
        slug,
        set,
        reach,
        topline: BTopline::Straight,
        toe: BToe::Round,
        // The sketches' sole carries its own heel: nothing under it.
        heel: BHeel::Flat(0.0),
        bands: SHEET,
        cap: false,
        foxing: true,
        piping: true,
        zip,
        quilt,
        extra: BExtra::None,
        tread,
        open: false,
        satin: false,
        colours,
    }
}

const TALL: BReach = BReach::Rim { back: 1.8, front: 1.98 };
const THIGH: BReach = BReach::Rim { back: 1.58, front: 1.74 };
const MID: BReach = BReach::Rim { back: 1.26, front: 1.41 };
const ANKLE: BReach = BReach::Rim { back: 0.92, front: 1.01 };
// The loafer on its own last (`sketch::SLIP`): the throat at the drawing's
// top, the collar falling to a heel counter two thirds its height.
const SLIP: BReach = BReach::Low { front: 0.47, side: 0.37, back: 0.3 };
// The flat on its own last (`sketch::FLAT`): the heel counter at the
// drawing's top, the line falling straight to a short throat by the toe.
const FLAT: BReach = BReach::Low { front: 0.24, side: 0.3, back: 0.35 };

/// A flat, or a pointe shoe with its satin box over the toe.
const fn ballet(label: &'static str, slug: &'static str, set: S, pointe: bool, colours: BColours, tread: BTread) -> BFootwearSpec {
    BFootwearSpec {
        label,
        slug,
        set,
        reach: FLAT,
        topline: BTopline::Straight,
        toe: BToe::Round,
        heel: BHeel::Flat(0.004),
        bands: &[],
        cap: pointe,
        foxing: false,
        piping: true,
        zip: false,
        quilt: false,
        extra: BExtra::Ribbons,
        tread,
        open: false,
        satin: true,
        colours,
    }
}

// ── The First Batch ───────────────────────────────────────────────────────

const THREE: &[BBand] = &[BBand { back: 0.5, front: 0.36, kind: K::Scallop }, BBand { back: 1.0, front: 1.12, kind: K::Scallop }, BBand { back: 1.5, front: 1.4, kind: K::Scallop }];
const TWO: &[BBand] = &[BBand { back: 0.45, front: 0.3, kind: K::Scallop }, BBand { back: 1.1, front: 1.1, kind: K::Scallop }];
/// One strap over the ankle running down to the vamp — the Option 08 boot's.
const OVER: &[BBand] = &[BBand { back: 0.42, front: 0.22, kind: K::Scallop }];

const fn boot(label: &'static str, slug: &'static str, set: S, reach: BReach, topline: BTopline, toe: BToe, heel: BHeel, bands: &'static [BBand], (upper, trim, sole): (Rgb, Rgb, Rgb), tread: BTread) -> BFootwearSpec {
    BFootwearSpec { label, slug, set, reach, topline, toe, heel, bands, cap: true, foxing: false, piping: true, zip: false, quilt: false, extra: BExtra::None, tread, open: false, satin: false, colours: c(upper, trim, trim, sole) }
}

const SLIPPER: BReach = BReach::Low { front: 0.25, side: 0.25, back: 0.36 };
const fn slipper(label: &'static str, slug: &'static str, upper: Rgb, ribbon: Rgb, tread: BTread) -> BFootwearSpec {
    BFootwearSpec { label, slug, set: S::BalletRibbon, reach: SLIPPER, topline: BTopline::Straight, toe: BToe::Round, heel: BHeel::Flat(0.004), bands: &[], cap: false, foxing: false, piping: false, zip: false, quilt: false, extra: BExtra::Ribbons, tread, open: false, satin: true, colours: BColours { upper, trim: ribbon, panel: upper, sole: BROWN, lining: upper } }
}

const fn sandal(label: &'static str, slug: &'static str, heel: BHeel, sole: Rgb, thong: Rgb, tread: BTread) -> BFootwearSpec {
    BFootwearSpec { label, slug, set: S::Sandal, reach: BReach::Low { front: 0.0, side: 0.0, back: 0.0 }, topline: BTopline::Straight, toe: BToe::Round, heel, bands: &[], cap: false, foxing: false, piping: false, zip: false, quilt: false, extra: BExtra::Thong, tread, open: true, satin: false, colours: BColours { upper: thong, trim: thong, panel: thong, sole, lining: sole } }
}

const PUMP: BReach = BReach::Low { front: 0.3, side: 0.25, back: 0.4 };
const fn heel(label: &'static str, slug: &'static str, toe: BToe, heel: BHeel, strap: bool, (upper, trim): (Rgb, Rgb), tread: BTread) -> BFootwearSpec {
    BFootwearSpec { label, slug, set: S::Heel, reach: PUMP, topline: BTopline::Straight, toe, heel, bands: &[], cap: true, foxing: false, piping: false, zip: false, quilt: false, extra: if strap { BExtra::AnkleStrap } else { BExtra::None }, tread, open: false, satin: false, colours: BColours { upper, trim, panel: BLACK, sole: RUBBER, lining: LINING } }
}

pub static DESIGNS: [BFootwearSpec; 48] = [
    cut("Tall Classique", "tall-classique", S::TallBoot, TALL, true, CLASSIQUE, false, BTread::Waves),
    cut("Tall Nuit", "tall-nuit", S::TallBoot, TALL, true, NUIT, false, BTread::Rings),
    cut("Tall Matelassé", "tall-matelasse", S::TallBoot, TALL, true, MATELASSE, true, BTread::Hexagon),
    cut("Tall Rose", "tall-rose", S::TallBoot, TALL, true, ROSE, false, BTread::Ripple),
    cut("Thigh Classique", "thigh-classique", S::ThighHigh, THIGH, true, CLASSIQUE, false, BTread::Waves),
    cut("Thigh Nuit", "thigh-nuit", S::ThighHigh, THIGH, true, NUIT, false, BTread::Rings),
    cut("Thigh Matelassé", "thigh-matelasse", S::ThighHigh, THIGH, true, MATELASSE, true, BTread::Hexagon),
    cut("Thigh Rose", "thigh-rose", S::ThighHigh, THIGH, true, ROSE, false, BTread::Ripple),
    cut("Mid Classique", "mid-classique", S::MidCalf, MID, true, CLASSIQUE, false, BTread::Waves),
    cut("Mid Nuit", "mid-nuit", S::MidCalf, MID, true, NUIT, false, BTread::Rings),
    cut("Mid Matelassé", "mid-matelasse", S::MidCalf, MID, true, MATELASSE, true, BTread::Hexagon),
    cut("Mid Rose", "mid-rose", S::MidCalf, MID, true, ROSE, false, BTread::Ripple),
    cut("Ankle Classique", "ankle-classique", S::AnkleBoot, ANKLE, false, CLASSIQUE, false, BTread::Waves),
    cut("Ankle Nuit", "ankle-nuit", S::AnkleBoot, ANKLE, false, NUIT, false, BTread::Rings),
    cut("Ankle Matelassé", "ankle-matelasse", S::AnkleBoot, ANKLE, false, MATELASSE, true, BTread::Hexagon),
    cut("Ankle Rose", "ankle-rose", S::AnkleBoot, ANKLE, false, ROSE, false, BTread::Ripple),
    cut("Slip-On Classique", "slip-classique", S::SlipOn, SLIP, false, CLASSIQUE, false, BTread::Waves),
    cut("Slip-On Nuit", "slip-nuit", S::SlipOn, SLIP, false, NUIT, false, BTread::Rings),
    cut("Slip-On Matelassé", "slip-matelasse", S::SlipOn, SLIP, false, MATELASSE, true, BTread::Hexagon),
    cut("Slip-On Rose", "slip-rose", S::SlipOn, SLIP, false, ROSE, false, BTread::Ripple),
    ballet("Flat Classique", "flat-classique", S::Ballet, false, c(BEIGE, BLACK, BLACK, BLACK), BTread::Waves),
    ballet("Flat Rose", "flat-rose", S::Ballet, false, c(BLUSH, BLUSH, PLUM, PLUM), BTread::Ripple),
    ballet("Pointe Nuit", "pointe-nuit", S::Ballet, true, c(NAVY, MAGENTA, NAVY, MIDNIGHT), BTread::Rings),
    ballet("Pointe Blush", "pointe-blush", S::Ballet, true, c(BLUSH, BLUSH, BLUSH, BROWN), BTread::Hexagon),
    boot("Tallulah", "tallulah", S::AboveKnee, BReach::Rim { back: 1.75, front: 1.8 }, BTopline::Peak(0.2), BToe::Round, BHeel::Block(0.035), THREE, (IVORY, GOLD, RUBBER), BTread::Waves),
    boot("Nightfall", "nightfall", S::AboveKnee, BReach::Rim { back: 1.7, front: 1.75 }, BTopline::Peak(0.24), BToe::Almond, BHeel::Flat(0.02), THREE, (NAVY, ORANGE, RUBBER), BTread::Hexagon),
    boot("Saddle", "saddle", S::AboveKnee, BReach::Rim { back: 1.6, front: 1.6 }, BTopline::Straight, BToe::Round, BHeel::Block(0.04), TWO, (TAN, CREAM, BROWN), BTread::Rings),
    boot("Onyx", "onyx", S::AboveKnee, BReach::Rim { back: 1.85, front: 1.85 }, BTopline::Notch(0.14), BToe::Point, BHeel::Stiletto(0.085), &[], (BLACK, SILVER, BLACK), BTread::Ripple),
    boot("Option 08", "option-08", S::MediumHigh, BReach::Rim { back: 1.25, front: 1.25 }, BTopline::Notch(0.2), BToe::Point, BHeel::Block(0.045), OVER, (CLAY, CLAY, CLAY), BTread::Waves),
    boot("Ember", "ember", S::MediumHigh, BReach::Rim { back: 1.15, front: 1.15 }, BTopline::Wave(0.05), BToe::Almond, BHeel::Block(0.03), TWO, (BURGUNDY, GOLD, RUBBER), BTread::Hexagon),
    boot("Tide", "tide", S::MediumHigh, BReach::Rim { back: 1.1, front: 1.1 }, BTopline::Peak(0.16), BToe::Round, BHeel::Flat(0.02), OVER, (NAVY, ORANGE, RUBBER), BTread::Rings),
    boot("Fawn", "fawn", S::MediumHigh, BReach::Rim { back: 1.05, front: 1.05 }, BTopline::Straight, BToe::Round, BHeel::Wedge(0.05), &[], (TAN, BROWN, CORK), BTread::Ripple),
    boot("Marigold", "marigold", S::AboveAnkle, BReach::Rim { back: 0.62, front: 0.62 }, BTopline::Wave(0.04), BToe::Round, BHeel::Flat(0.02), OVER, (ORANGE, NAVY, RUBBER), BTread::Waves),
    boot("Clay", "clay", S::AboveAnkle, BReach::Rim { back: 0.66, front: 0.66 }, BTopline::Notch(0.12), BToe::Almond, BHeel::Block(0.035), OVER, (IVORY, TAN, BROWN), BTread::Hexagon),
    boot("Coal", "coal", S::AboveAnkle, BReach::Rim { back: 0.58, front: 0.58 }, BTopline::Straight, BToe::Point, BHeel::Stiletto(0.08), &[], (BLACK, GOLD, BLACK), BTread::Rings),
    boot("Moss", "moss", S::AboveAnkle, BReach::Rim { back: 0.7, front: 0.7 }, BTopline::Peak(0.1), BToe::Round, BHeel::Block(0.025), OVER, (OLIVE, CREAM, RUBBER), BTread::Ripple),
    slipper("Blush", "blush", BLUSH, BLUSH, BTread::Waves),
    slipper("Swan", "swan", IVORY, IVORY, BTread::Hexagon),
    slipper("Noir", "noir", BLACK, BLACK, BTread::Rings),
    slipper("Sunset", "sunset", ORANGE, NAVY, BTread::Ripple),
    sandal("Shore", "shore", BHeel::Flat(0.004), CORK, NAVY, BTread::Waves),
    sandal("Coral", "coral", BHeel::Flat(0.004), CREAM, CORAL, BTread::Hexagon),
    sandal("Midnight", "midnight", BHeel::Flat(0.004), BLACK, GOLD, BTread::Rings),
    sandal("Dune", "dune", BHeel::Wedge(0.035), CORK, TAN, BTread::Ripple),
    heel("Gala", "gala", BToe::Point, BHeel::Stiletto(0.09), true, (BLACK, GOLD), BTread::Waves),
    heel("Rose", "rose", BToe::Almond, BHeel::Stiletto(0.075), true, (BLUSH, GOLD), BTread::Hexagon),
    heel("Amber", "amber", BToe::Point, BHeel::Block(0.065), true, (ORANGE, NAVY), BTread::Rings),
    heel("Pearl", "pearl", BToe::Point, BHeel::Stiletto(0.1), false, (IVORY, GOLD), BTread::Ripple),
];
