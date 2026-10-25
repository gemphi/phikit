//! The sketch check — a "pix box" for each design of the Cut-Off Series: a
//! plane ruled in centimetres, its origin where the shoe meets the floor at
//! its back, the user's drawing laid on it faintly, and over it two clean
//! outlines — the drawing's silhouette (amber) and the model's drawn from the
//! same side (cyan, `phiplay`'s picture), scaled to the drawing's height and
//! stood on its floor. What the inspector shows beside "Confirmed against
//! the sketch", so whoever owns the design can say it matches.
//!
//! The drawings are `assets/footwear/sketch/*.png`, cut from the sheets by
//! `phiplay/tools/sketch/export.py`, which also measured each silhouette's
//! box in them (`CHECKS`).

use phiplay::build::footwear::picture::BShot;
use phiplay::kit::footwear::{BFootwearSet, BFootwearSpec};

mod plane;

pub use plane::{image, overlap};

pub const PREFIX: &str = "wear-check:";
const EDGE: u32 = 512;

/// Where the model is stood in the drawing's box: by its left edge, its
/// right edge, or its middle.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BStand {
    Left,
    Right,
    Middle,
}

/// One drawing: the view it shows, its picture, the silhouette's box in it,
/// whether the model is mirrored to face the way the drawing does, and how
/// it is stood in the box.
pub struct BCheck {
    pub label: &'static str,
    shot: BShot,
    png: &'static [u8],
    bounds: [u32; 4],
    flip: bool,
    stand: BStand,
}

macro_rules! sketch {
    ($name:literal) => {
        include_bytes!(concat!("../../../../../assets/footwear/sketch/", $name, ".png"))
    };
}

const TALL: [BCheck; 5] = [
    BCheck { label: "Side (outer)", shot: BShot::SideOuter, png: sketch!("tall_side"), bounds: [8, 9, 302, 558], flip: false, stand: BStand::Left },
    BCheck { label: "Side (inner)", shot: BShot::SideInner, png: sketch!("tall_inner"), bounds: [7, 10, 277, 518], flip: false, stand: BStand::Right },
    BCheck { label: "Front", shot: BShot::Front, png: sketch!("tall_front"), bounds: [10, 13, 149, 557], flip: false, stand: BStand::Middle },
    BCheck { label: "Back", shot: BShot::Back, png: sketch!("tall_back"), bounds: [5, 14, 155, 556], flip: false, stand: BStand::Middle },
    BCheck { label: "Sole", shot: BShot::Sole, png: sketch!("tall_sole"), bounds: [12, 10, 226, 553], flip: false, stand: BStand::Middle },
];
const THIGH: [BCheck; 1] = [BCheck { label: "Side", shot: BShot::SideOuter, png: sketch!("thigh_side"), bounds: [9, 10, 342, 549], flip: true, stand: BStand::Right }];
const MID: [BCheck; 1] = [BCheck { label: "Side", shot: BShot::SideOuter, png: sketch!("mid_side"), bounds: [11, 10, 403, 537], flip: true, stand: BStand::Right }];
const ANKLE: [BCheck; 1] = [BCheck { label: "Front", shot: BShot::Front, png: sketch!("ankle_front"), bounds: [14, 11, 245, 536], flip: false, stand: BStand::Middle }];
const SLIP: [BCheck; 1] = [BCheck { label: "Side", shot: BShot::SideOuter, png: sketch!("slip_side"), bounds: [13, 16, 519, 248], flip: true, stand: BStand::Right }];
const FLAT: [BCheck; 1] = [BCheck { label: "Side", shot: BShot::SideOuter, png: sketch!("flat_side"), bounds: [14, 12, 625, 442], flip: false, stand: BStand::Left }];

/// The drawings a design is checked against; none for the First Batch.
pub fn checks(d: &BFootwearSpec) -> &'static [BCheck] {
    match d.set {
        BFootwearSet::TallBoot => &TALL,
        BFootwearSet::ThighHigh => &THIGH,
        BFootwearSet::MidCalf => &MID,
        BFootwearSet::AnkleBoot => &ANKLE,
        BFootwearSet::SlipOn => &SLIP,
        BFootwearSet::Ballet => &FLAT,
        _ => &[],
    }
}

pub fn key(d: &BFootwearSpec, i: usize) -> String {
    format!("{PREFIX}{}:{i}", d.slug)
}

#[cfg(test)]
mod tests {
    use super::plane::{AXIS, MAJOR, MODEL, SKETCH};
    use super::*;

    #[test]
    fn every_cut_off_design_has_its_drawing_and_the_model_lands_on_it() {
        for d in phiplay::kit::footwear::DESIGNS.iter() {
            let n = checks(d).len();
            assert_eq!(n > 0, d.set.collection() == phiplay::kit::footwear::BCollection::CutOff, "{}", d.label);
        }
        let d = phiplay::kit::footwear::by_slug("tall-classique").unwrap();
        let (w, h, rgba) = image(&key(d, 0)).expect("the overlay");
        let count = |c: [u8; 3]| rgba.chunks(4).filter(|p| p[..3] == c).count();
        assert!(count(MODEL) > ((w + h) / 2) as usize, "the model's outline is drawn: {}", count(MODEL));
        assert!(count(SKETCH) > ((w + h) / 2) as usize, "the drawing's outline is drawn: {}", count(SKETCH));
        assert!(count(MAJOR) > 0 && count(AXIS) > 0, "the plane is ruled");
        let score = overlap(&key(d, 0)).unwrap();
        assert!(score > 0.75, "the tall boot's side lands on its drawing: {score}");
    }
}
