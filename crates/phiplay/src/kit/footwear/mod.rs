//! Footwear off the court — the wardrobe the players arrive in before the
//! game, in two collections:
//!
//! - **The Cut-Off Series** (`BCollection::CutOff`), the user's sketch sheets
//!   made solid: one tall boot — its side, front and sole read off the
//!   drawing (`build::footwear::sketch`) — cut off at six heights, from the
//!   3/4 tall boot down to the slip-on and the ballet slipper, each in the
//!   house's four colourways. The drawing's marks are fields: the scalloped
//!   edging repeated along the rim and the straps (`BBandKind::Scallop`),
//!   the plain and wavy lines, the foxing panel round the foot, the back zip,
//!   the sole that cups the heel, and the softened hexagon quilting the pink
//!   sheets suggest (`quilt`).
//! - **The First Batch** (`BCollection::FirstBatch`), the earlier
//!   twenty-four — boots, ballet, sandals, heels — now on the same form.
//!
//! Each design is an entity of named fields (`BFootwearSpec`); the builder
//! (`build::footwear`) turns it into a pair. Heights are in foot lengths
//! (a women's 39, 25 cm), as the sketches are drawn.

mod set;

pub use set::DESIGNS;

use crate::colour::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BCollection {
    CutOff,
    FirstBatch,
}

impl BCollection {
    pub const ALL: [BCollection; 2] = [Self::CutOff, Self::FirstBatch];

    pub fn label(self) -> &'static str {
        match self {
            Self::CutOff => "The Cut-Off Series",
            Self::FirstBatch => "First Batch",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::CutOff => "cut-off",
            Self::FirstBatch => "first-batch",
        }
    }

    pub fn sets(self) -> impl Iterator<Item = BFootwearSet> {
        BFootwearSet::ALL.into_iter().filter(move |s| s.collection() == self)
    }
}

/// The sets the Project Browser shows as folders, six a collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BFootwearSet {
    TallBoot,
    ThighHigh,
    MidCalf,
    AnkleBoot,
    SlipOn,
    Ballet,
    AboveKnee,
    MediumHigh,
    AboveAnkle,
    BalletRibbon,
    Sandal,
    Heel,
}

pub struct BSetSpec {
    pub label: &'static str,
    pub slug: &'static str,
    /// One line on what the set is.
    pub caption: &'static str,
}

impl BFootwearSet {
    pub const ALL: [BFootwearSet; 12] = [
        Self::TallBoot,
        Self::ThighHigh,
        Self::MidCalf,
        Self::AnkleBoot,
        Self::SlipOn,
        Self::Ballet,
        Self::AboveKnee,
        Self::MediumHigh,
        Self::AboveAnkle,
        Self::BalletRibbon,
        Self::Sandal,
        Self::Heel,
    ];

    pub fn collection(self) -> BCollection {
        match self {
            Self::TallBoot | Self::ThighHigh | Self::MidCalf | Self::AnkleBoot | Self::SlipOn | Self::Ballet => BCollection::CutOff,
            _ => BCollection::FirstBatch,
        }
    }

    pub fn spec(self) -> &'static BSetSpec {
        match self {
            Self::TallBoot => &BSetSpec { label: "3/4 Tall Boot", slug: "tall-boot", caption: "Over the knee, the rim rising to the front, scalloped bands" },
            Self::ThighHigh => &BSetSpec { label: "Thigh-High", slug: "thigh-high", caption: "The tall boot cut a little lower" },
            Self::MidCalf => &BSetSpec { label: "Mid-Calf Boot", slug: "mid-calf", caption: "Cut at the calf, the foxing round the foot" },
            Self::AnkleBoot => &BSetSpec { label: "Ankle Boot", slug: "ankle-boot", caption: "Cut just over the ankle" },
            Self::SlipOn => &BSetSpec { label: "Slip-On Shoe", slug: "slip-on", caption: "A high vamp, the heel open" },
            Self::Ballet => &BSetSpec { label: "Ballet", slug: "ballet", caption: "The flat and the pointe shoe, ribbons crossed up the ankle" },
            Self::AboveKnee => &BSetSpec { label: "Above the Knee", slug: "above-knee", caption: "A shaft over the knee, three scalloped straps" },
            Self::MediumHigh => &BSetSpec { label: "Medium High", slug: "medium-high", caption: "To mid-calf, the rim cut to shape" },
            Self::AboveAnkle => &BSetSpec { label: "Above Ankle", slug: "above-ankle", caption: "A bootie just over the ankle bone" },
            Self::BalletRibbon => &BSetSpec { label: "Ballet Slipper", slug: "ballet-slipper", caption: "Satin, ribbons wound to the shin and tied in a bow" },
            Self::Sandal => &BSetSpec { label: "Sandal", slug: "sandal", caption: "A sole and a thong between the toes" },
            Self::Heel => &BSetSpec { label: "Heel", slug: "heel", caption: "A pump on a heel, most with an ankle strap" },
        }
    }

    pub fn by_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.spec().slug == slug)
    }

    /// The set's designs, in the order the browser shows them.
    pub fn designs(self) -> impl Iterator<Item = &'static BFootwearSpec> {
        DESIGNS.iter().filter(move |d| d.set == self)
    }
}

/// Where the shoe's opening sits, in foot lengths off the floor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BReach {
    /// A shaft: the rim at the back and at the front.
    Rim { back: f32, front: f32 },
    /// A low shoe: its rim line down the foot — the throat over the instep
    /// (`front`), the collar at the side of the foot, the heel counter.
    Low { front: f32, side: f32, back: f32 },
}

/// The rim's shape on a shaft.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BTopline {
    Straight,
    /// Rising this much more at the front (foot lengths).
    Peak(f32),
    /// Cut down this much in a V at the front and the back, leaving ears at
    /// the sides — the "Option 08" boot.
    Notch(f32),
    /// A wave this deep, six crests round.
    Wave(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BToe {
    Round,
    Almond,
    Point,
}

impl BToe {
    /// How much longer the toe runs than the form's own.
    pub fn stretch(self) -> f32 {
        match self {
            Self::Round => 0.0,
            Self::Almond => 0.1,
            Self::Point => 0.22,
        }
    }
}

/// What the heel stands on, and how high it raises it (metres).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BHeel {
    /// The sketches' sole: its own heel and nothing under it.
    Flat(f32),
    /// A block, narrowing a little to the floor, in tiers.
    Block(f32),
    Stiletto(f32),
    /// The sole itself raised under the heel, tapering to the ball.
    Wedge(f32),
}

impl BHeel {
    pub fn height(self) -> f32 {
        match self {
            Self::Flat(h) | Self::Block(h) | Self::Stiletto(h) | Self::Wedge(h) => h,
        }
    }

    pub fn label(self) -> String {
        let cm = |h: f32| format!("{:.1} cm", h * 100.0);
        match self {
            Self::Flat(h) => format!("Flat, {}", cm(h)),
            Self::Block(h) => format!("Block, {}", cm(h)),
            Self::Stiletto(h) => format!("Stiletto, {}", cm(h)),
            Self::Wedge(h) => format!("Wedge, {}", cm(h)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BBandKind {
    /// A strap edged with a row of scallops below it, repeated along it.
    Scallop,
    /// A stitched line.
    Line,
    /// A stitched line that waves.
    Wave,
}

/// A band round the shaft, its height at the back and at the front (foot
/// lengths) — the sketches' lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBand {
    pub back: f32,
    pub front: f32,
    pub kind: BBandKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BExtra {
    None,
    /// Two satin ribbons wound up the shin, crossing, tied at the front.
    Ribbons,
    /// A thong from between the toes to either side of the sole.
    Thong,
    /// A strap round the ankle, held up from the heel.
    AnkleStrap,
}

/// The sole's lines: the arcs its wall carries round the heel and the toe
/// (the drawings' heel lines) and the pattern it meets the ground with —
/// one of the four per design.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BTread {
    /// Waves across the sole, rings under the heel — the sole sheet's.
    Waves,
    /// Six-sided pads under the heel and the ball.
    Hexagon,
    /// Rings round the heel and round the ball.
    Rings,
    /// Arcs across the heel's back, one inside the other — the back view's.
    Ripple,
}

impl BTread {
    pub const ALL: [BTread; 4] = [Self::Waves, Self::Hexagon, Self::Rings, Self::Ripple];
}

/// The colours of a design.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BColours {
    pub upper: Rgb,
    /// The piping, the scallops, the lines.
    pub trim: Rgb,
    /// The foxing and the toe cap.
    pub panel: Rgb,
    pub sole: Rgb,
    pub lining: Rgb,
}

/// One design — the entity's fields.
#[derive(Clone, Debug, PartialEq)]
pub struct BFootwearSpec {
    pub label: &'static str,
    pub slug: &'static str,
    pub set: BFootwearSet,
    pub reach: BReach,
    pub topline: BTopline,
    pub toe: BToe,
    pub heel: BHeel,
    pub bands: &'static [BBand],
    /// A toe cap in the panel's colour.
    pub cap: bool,
    /// The panel round the foot under the sketch's vamp line.
    pub foxing: bool,
    /// Scalloped edging round the rim.
    pub piping: bool,
    /// A zip up the back.
    pub zip: bool,
    /// The upper quilted in softened hexagons.
    pub quilt: bool,
    pub extra: BExtra,
    pub tread: BTread,
    /// No upper at all: a sole and its straps.
    pub open: bool,
    /// Satin rather than leather: smoother, with more sheen.
    pub satin: bool,
    pub colours: BColours,
}

impl BFootwearSpec {
    /// The fields as the inspector lists them.
    pub fn fields(&self) -> Vec<(&'static str, String)> {
        let l = |x: f32| format!("{:.0} cm", x * 25.0);
        let reach = match self.reach {
            BReach::Rim { back, front } => format!("{} at the back, {} at the front", l(back), l(front)),
            BReach::Low { front, back, .. } => format!("Low, {} at the heel, {} at the vamp", l(back), l(front)),
        };
        let topline = match self.topline {
            BTopline::Straight => "Straight".to_string(),
            BTopline::Peak(m) => format!("Peak, {}", l(m)),
            BTopline::Notch(m) => format!("Notch, {}", l(m)),
            BTopline::Wave(m) => format!("Wave, {}", l(m)),
        };
        let hex = |c: Rgb| {
            let [r, g, b] = c.srgb();
            format!("#{:02X}{:02X}{:02X}", (r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
        };
        let yes = |b: bool| if b { "Yes" } else { "No" }.to_string();
        vec![
            ("Collection", self.set.collection().label().to_string()),
            ("Set", self.set.spec().label.to_string()),
            ("Reach", reach),
            ("Rim", topline),
            ("Toe", format!("{:?}", self.toe)),
            ("Heel", self.heel.label()),
            ("Bands", self.bands.len().to_string()),
            ("Toe cap", yes(self.cap)),
            ("Foxing", yes(self.foxing)),
            ("Scalloped rim", yes(self.piping)),
            ("Back zip", yes(self.zip)),
            ("Quilted", yes(self.quilt)),
            ("Extra", format!("{:?}", self.extra)),
            ("Tread", format!("{:?}", self.tread)),
            ("Material", if self.satin { "Satin" } else { "Leather" }.to_string()),
            ("Upper", hex(self.colours.upper)),
            ("Trim", hex(self.colours.trim)),
            ("Panel", hex(self.colours.panel)),
            ("Sole", hex(self.colours.sole)),
        ]
    }
}

pub fn by_slug(slug: &str) -> Option<&'static BFootwearSpec> {
    DESIGNS.iter().find(|d| d.slug == slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_collections_of_six_sets_of_four_every_slug_its_own() {
        for c in BCollection::ALL {
            assert_eq!(c.sets().count(), 6, "{}", c.label());
        }
        for set in BFootwearSet::ALL {
            assert_eq!(set.designs().count(), 4, "{}", set.spec().label);
            // The heel lines tell a set's four apart: each its own tread.
            let mut treads: Vec<BTread> = set.designs().map(|d| d.tread).collect();
            treads.sort_by_key(|t| BTread::ALL.iter().position(|a| a == t));
            assert_eq!(treads, BTread::ALL, "{}", set.spec().label);
            assert_eq!(BFootwearSet::by_slug(set.spec().slug), Some(set));
        }
        let mut slugs: Vec<&str> = DESIGNS.iter().map(|d| d.slug).collect();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), DESIGNS.len());
        for d in DESIGNS.iter() {
            assert_eq!(by_slug(d.slug).map(|x| x.label), Some(d.label));
            assert!(d.heel.height() >= 0.0 && d.heel.height() < 0.12, "{}", d.label);
        }
    }

    #[test]
    fn the_cut_off_series_is_one_boot_cut_lower_and_lower() {
        let rim = |set: BFootwearSet| {
            set.designs()
                .map(|d| match d.reach {
                    BReach::Rim { back, front } => back.max(front),
                    BReach::Low { front, side, back } => front.max(side).max(back),
                })
                .fold(0.0f32, f32::max)
        };
        let series = [BFootwearSet::TallBoot, BFootwearSet::ThighHigh, BFootwearSet::MidCalf, BFootwearSet::AnkleBoot, BFootwearSet::SlipOn];
        for w in series.windows(2) {
            assert!(rim(w[0]) > rim(w[1]), "{} is cut over {}", w[0].spec().label, w[1].spec().label);
        }
        for d in DESIGNS.iter().filter(|d| d.set.collection() == BCollection::CutOff) {
            assert!(d.toe != BToe::Point, "the sketches' toe is round: {}", d.label);
        }
    }
}
