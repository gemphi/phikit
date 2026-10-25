//! The sixteen hairstyles.

use super::{BHair, BHairSpec, BStrands, BVolume};

impl BHair {
    pub fn spec(self) -> &'static BHairSpec {
        match self {
            BHair::Buzz => &BUZZ,
            BHair::Crop => &CROP,
            BHair::Pixie => &PIXIE,
            BHair::Bob => &BOB,
            BHair::Waves => &WAVES,
            BHair::Afro => &AFRO,
            BHair::Puff => &PUFF,
            BHair::DoublePuff => &DOUBLE_PUFF,
            BHair::Bun => &BUN,
            BHair::TopKnot => &TOP_KNOT,
            BHair::Ponytail => &PONYTAIL,
            BHair::Cornrows => &CORNROWS,
            BHair::BoxBraids => &BOX_BRAIDS,
            BHair::Locs => &LOCS,
            BHair::Twists => &TWISTS,
            BHair::Fade => &FADE,
        }
    }
}

const fn style(label: &'static str, shell: f32, reach: f32, curl: f32, volume: BVolume, strands: BStrands, faded_sides: bool) -> BHairSpec {
    BHairSpec { label, shell, reach, curl, volume, strands, faded_sides }
}

use BStrands as S;
use BVolume as V;

static BUZZ: BHairSpec = style("Buzz", 0.003, 0.62, 0.2, V::None, S::None, false);
static CROP: BHairSpec = style("Crop", 0.012, 0.6, 0.3, V::None, S::None, false);
static PIXIE: BHairSpec = style("Pixie", 0.018, 0.68, 0.1, V::None, S::None, false);
static BOB: BHairSpec = style("Bob", 0.022, 0.92, 0.05, V::None, S::None, false);
static WAVES: BHairSpec = style("Waves", 0.008, 0.6, 0.55, V::None, S::None, false);
static AFRO: BHairSpec = style("Afro", 0.075, 0.78, 0.95, V::None, S::None, false);
static PUFF: BHairSpec = style("Puff", 0.008, 0.66, 0.8, V::Puff { radius: 0.085, at: 0.85, pair: false }, S::None, false);
static DOUBLE_PUFF: BHairSpec = style("Double puff", 0.008, 0.66, 0.8, V::Puff { radius: 0.06, at: 0.75, pair: true }, S::None, false);
static BUN: BHairSpec = style("Bun", 0.01, 0.7, 0.15, V::Puff { radius: 0.05, at: 0.55, pair: false }, S::None, false);
static TOP_KNOT: BHairSpec = style("Top knot", 0.01, 0.68, 0.2, V::Puff { radius: 0.045, at: 1.0, pair: false }, S::None, false);
static PONYTAIL: BHairSpec = style("Ponytail", 0.012, 0.72, 0.1, V::None, S::Tail { length: 0.3, width: 0.05 }, false);
static CORNROWS: BHairSpec = style("Cornrows", 0.006, 0.7, 0.3, V::None, S::Rows { count: 8 }, false);
static BOX_BRAIDS: BHairSpec = style("Box braids", 0.01, 0.72, 0.4, V::None, S::Hanging { count: 48, length: 0.42, width: 0.011, braid: true }, false);
static LOCS: BHairSpec = style("Locs", 0.014, 0.74, 0.6, V::None, S::Hanging { count: 36, length: 0.32, width: 0.015, braid: false }, false);
static TWISTS: BHairSpec = style("Twists", 0.016, 0.72, 0.7, V::None, S::Hanging { count: 40, length: 0.16, width: 0.012, braid: true }, false);
static FADE: BHairSpec = style("Fade", 0.02, 0.4, 0.75, V::None, S::None, true);
