//! The eight seats.

use super::{BBack, BSeat, BSeatSpec};

impl BSeat {
    pub fn spec(self) -> &'static BSeatSpec {
        match self {
            BSeat::SemiHex => &SEMI_HEX,
            BSeat::Phiegg => &PHIEGG,
            BSeat::Shell => &SHELL,
            BSeat::Wing => &WING,
            BSeat::Pod => &POD,
            BSeat::Saddle => &SADDLE,
            BSeat::Bench => &BENCH,
            BSeat::Blade => &BLADE,
        }
    }
}

static SEMI_HEX: BSeatSpec = BSeatSpec { label: "Semi-hex", back: BBack::SemiHex { top: 0.5 }, width: 0.4, height: 0.38, lean: 10.0, facets: 6 };
static PHIEGG: BSeatSpec = BSeatSpec { label: "Phiegg", back: BBack::Egg { n: 2.6, top: 1.0 / 3.0 }, width: 0.38, height: 0.42, lean: 10.0, facets: 10 };
static SHELL: BSeatSpec = BSeatSpec { label: "Shell", back: BBack::Shell { wrap: 140.0 }, width: 0.42, height: 0.34, lean: 8.0, facets: 12 };
static WING: BSeatSpec = BSeatSpec { label: "Wing", back: BBack::Wing { sweep: 25.0 }, width: 0.4, height: 0.46, lean: 18.0, facets: 8 };
static POD: BSeatSpec = BSeatSpec { label: "Pod", back: BBack::Pod { round: 0.35 }, width: 0.4, height: 0.4, lean: 9.0, facets: 16 };
static SADDLE: BSeatSpec = BSeatSpec { label: "Saddle", back: BBack::Saddle, width: 0.42, height: 0.16, lean: 6.0, facets: 10 };
static BENCH: BSeatSpec = BSeatSpec { label: "Bench", back: BBack::None, width: 0.5, height: 0.0, lean: 0.0, facets: 4 };
static BLADE: BSeatSpec = BSeatSpec { label: "Blade", back: BBack::Blade, width: 0.3, height: 0.5, lean: 12.0, facets: 4 };
