//! A player's uniform, cut from her body: a top and bottoms in the sport's
//! cut (`BCut`), and for soccer long socks, painted with the uniform's
//! pattern and her number.
//!
//! The landmarks are her skeleton's — the neck, the waist, the hips, the
//! knees, the ankles (Rigify's `DEF-spine.004`, `.001`, `DEF-thigh`,
//! `DEF-shin`, `DEF-foot`) — so the same cut fits any figure on that rig.
//!
//! The top is painted in a wrap round her: its texture's columns go once
//! round the body (the front's middle in the middle), its rows from the
//! hem up; the pattern is read in the shirt's own frame (`BPattern`, u
//! across the front from her right side, v hem to shoulders), the back a
//! mirror of the front, and the number is set in metres on her chest and
//! larger on her back.

use super::body::{BBody, BPiece};
use super::font;
use super::image::BImage;
use super::part::BMat;
use crate::colour::Rgb;
use crate::kit::BCut;
use crate::kit::uniform::BUniformSpec;
use crate::team::BTeamSpec;
use glam::Vec3;
use std::f32::consts::{PI, TAU};
use std::sync::Arc;

/// A garment: the piece and the materials its faces use.
#[derive(Clone, Debug)]
pub struct BGarment {
    pub name: String,
    pub piece: BPiece,
    pub materials: Vec<BMat>,
}

/// Where her kit starts and ends, from her bones.
#[derive(Clone, Copy, Debug)]
pub struct BLandmarks {
    pub neck: f32,
    pub waist: f32,
    pub hip: f32,
    pub knee: f32,
    pub ankle: f32,
}

impl BLandmarks {
    pub fn of(body: &BBody) -> Option<BLandmarks> {
        let z = |bone: &str| body.head(bone).map(|p| p.z);
        Some(BLandmarks { neck: z("DEF-spine.004")?, waist: z("DEF-spine.001")?, hip: z("DEF-thigh.L")?, knee: z("DEF-shin.L")?, ankle: z("DEF-foot.L")? })
    }
}

/// The arm's bones: a face mostly on them is a sleeve's, not the torso's.
const ARM: [&str; 6] = ["DEF-upper_arm", "DEF-forearm", "DEF-hand", "DEF-palm", "DEF-f_", "DEF-thumb"];
/// The side panels' and the straps' x, as make_demo cut them.
const PANEL_X: f32 = 0.105;
const STRAP: (f32, f32) = (0.06, 0.115);
/// The chest's radius, for setting the number in metres round it.
const CHEST: f32 = 0.15;

/// The top, the bottoms and (for a jersey) the socks.
pub fn kit(body: &BBody, cut: BCut, uniform: &BUniformSpec, shirt: Rgb, trim: Rgb, number: u8) -> Vec<BGarment> {
    let Some(at) = BLandmarks::of(body) else { return Vec::new() };
    let (hem, neck) = match cut {
        BCut::Vest => (at.waist - 0.04, at.neck - 0.01),
        BCut::Fitted => (at.waist - 0.06, at.neck),
        BCut::Crop => (at.neck - 0.24, at.neck - 0.01),
        BCut::Jersey => (at.waist - 0.10, at.neck + 0.01),
    };
    let (shorts_hem, waistband) = match cut {
        BCut::Vest => (at.knee + 0.45 * (at.hip - at.knee), at.waist + 0.03),
        BCut::Fitted => (at.hip - 0.10, at.waist + 0.02),
        BCut::Crop => (at.hip - 0.06, at.waist - 0.02),
        BCut::Jersey => (at.knee + 0.55 * (at.hip - at.knee), at.waist + 0.03),
    };
    let mut cuts = body.bisect(2, hem).bisect(2, neck).bisect(2, shorts_hem).bisect(2, waistband);
    for x in [-PANEL_X, PANEL_X, -STRAP.0, STRAP.0, -STRAP.1, STRAP.1] {
        cuts = cuts.bisect(0, x);
    }
    // A short sleeve: the upper arm's first half, measured along the bone
    // from the shoulder — whichever way the arm is posed.
    let upper_arm = |side: &str| Some((body.head(&format!("DEF-upper_arm.{side}"))?, body.tail(&format!("DEF-upper_arm.{side}"))?));
    let arms = [upper_arm("L"), upper_arm("R")];
    let sleeve = |face: &[u32], c: Vec3| match cut {
        BCut::Fitted | BCut::Jersey => {
            let Some((head, tail)) = arms[usize::from(c.x < 0.0)] else { return false };
            let along = (c - head).dot(tail - head) / (tail - head).length_squared();
            cuts.share(face, &["DEF-upper_arm"]) > 0.5 && along < 0.55
        }
        _ => false,
    };
    let top_keep = |face: &[u32], c: Vec3| {
        let arm = cuts.share(face, &ARM);
        let trunk = c.z > hem && c.z < neck && c.x.abs() < 0.28 && arm < 0.35;
        let strap = c.z >= neck && c.z < neck + 0.08 && c.x.abs() > STRAP.0 && c.x.abs() < STRAP.1;
        let shoulders = matches!(cut, BCut::Fitted | BCut::Jersey) && c.z >= neck && c.z < neck + 0.1 && c.x.abs() > STRAP.0 && arm < 0.6;
        trunk || strap || shoulders || sleeve(face, c)
    };
    let top_image = Arc::new(top_texture(uniform, shirt, trim, number, hem, neck));
    let top = cuts.piece(top_keep, |_| 0.016, |p| wrap_uv(p, hem, neck + 0.12), |_| 0);
    let mut out = vec![BGarment { name: "Top".into(), piece: top, materials: vec![BMat::plain("Top", shirt, 0.55).with_image(top_image)] }];
    let flare = move |p: Vec3| {
        let t = ((waistband - 0.08 - p.z) / (waistband - 0.08 - shorts_hem).max(1e-6)).clamp(0.0, 1.0);
        0.011 + if cut == BCut::Crop { 0.0 } else { 0.04 * t.powf(1.5) }
    };
    let bottoms = cuts.piece(|face, c| c.z > shorts_hem && c.z < waistband && c.x.abs() < 0.28 && cuts.share(face, &ARM) < 0.35, flare, |p| wrap_uv(p, shorts_hem, waistband), |_| 0);
    let bottoms_image = Arc::new(bottoms_texture(uniform, shirt, trim, shorts_hem, waistband));
    let name = if cut == BCut::Crop { "Briefs" } else { "Shorts" };
    out.push(BGarment { name: name.into(), piece: bottoms, materials: vec![BMat::plain(name, shirt, 0.6).with_image(bottoms_image)] });
    if cut == BCut::Jersey {
        let legs = cuts.bisect(2, at.knee - 0.04);
        let socks = legs.piece(|face, c| c.z > at.ankle - 0.04 && c.z < at.knee - 0.04 && legs.share(face, &ARM) < 0.1, |_| 0.006, |p| wrap_uv(p, at.ankle, at.knee), |_| 0);
        out.push(BGarment { name: "Socks".into(), piece: socks, materials: vec![BMat::plain("Socks", trim, 0.7)] });
    }
    out
}

/// The wrap UV: u once round the body's vertical axis (her front's middle,
/// −y, at 0.5), v from `lo` to `hi`.
fn wrap_uv(p: Vec3, lo: f32, hi: f32) -> [f32; 2] {
    let angle = p.x.atan2(-p.y);
    [0.5 + angle / TAU, ((p.z - lo) / (hi - lo)).clamp(0.0, 1.0)]
}

/// The pattern's frame at a wrap angle: u across the front (or the back,
/// mirrored) from her right side, and whether it is the front.
fn pattern_u(angle: f32) -> (f32, bool) {
    if angle.abs() <= PI / 2.0 {
        (0.5 + angle / PI, true)
    } else {
        (0.5 + angle.signum() * (PI - angle.abs()) / PI, false)
    }
}

fn top_texture(uniform: &BUniformSpec, shirt: Rgb, trim: Rgb, number: u8, hem: f32, neck: f32) -> BImage {
    let ink = BTeamSpec::number_ink(shirt);
    let text = number.to_string();
    let top = neck + 0.12;
    BImage::painted(1024, 512, |u, v| {
        let angle = (u - 0.5) * TAU;
        let z = top - v * (top - hem);
        let (pu, front) = pattern_u(angle);
        let pv = (z - hem) / (neck - hem);
        let mut c = shirt.mix(trim, uniform.pattern.trim_weight(pu, pv));
        // Piping at the neck and the arm holes' top.
        if pv > 0.97 {
            c = trim;
        }
        // The number: 14 cm on the chest, 24 cm on the back, in metres
        // round her.
        let (height, centre) = if front { (0.14, hem + 0.62 * (neck - hem)) } else { (0.24, hem + 0.55 * (neck - hem)) };
        // Metres round her from the number's middle, read left to right by
        // whoever faces it: toward her left (+x) on the front, toward her
        // right on the back.
        let s = if front { angle * CHEST } else { -angle.signum() * (PI - angle.abs()) * CHEST };
        let glyph = height / font::TALL;
        if font::covers(&text, [s / glyph, (z - centre) / glyph], 0.22, 0.12) {
            c = ink;
        }
        c
    })
}

fn bottoms_texture(uniform: &BUniformSpec, shirt: Rgb, trim: Rgb, hem: f32, waist: f32) -> BImage {
    BImage::painted(512, 256, |u, v| {
        let angle = (u - 0.5) * TAU;
        let z = waist - v * (waist - hem);
        let side = (angle.abs() - PI / 2.0).abs() * CHEST < 0.02;
        let band = z - hem < 0.03 || waist - z < 0.025;
        if (uniform.side_stripe && side) || band { trim } else { shirt }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wrap_puts_her_front_in_the_middle_and_mirrors_the_back() {
        assert_eq!(wrap_uv(Vec3::new(0.0, -0.1, 1.0), 0.0, 2.0), [0.5, 0.5]);
        let (right, front) = pattern_u(-PI / 2.0);
        assert!(front && right.abs() < 1e-6, "her right side is u = 0");
        let (back, front) = pattern_u(PI);
        assert!(!front && (back - 0.5).abs() < 1e-6, "the back's middle is the pattern's middle");
        let (a, _) = pattern_u(-PI / 2.0 - 0.01);
        assert!(a < 0.01, "continuous round the side: {a}");
    }
}
