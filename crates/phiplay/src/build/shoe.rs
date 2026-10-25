//! The shoes, cut from her feet up to the design's collar — every one
//! above the ankle bone — and stood off the skin more toward the floor so
//! the sole has depth (make_demo's sneaker). Three material slots: the
//! upper, the sole, and the trim, laid where the design says (`BShoeTrim`)
//! in each foot's own frame, heel to toe.

use super::body::BBody;
use super::kit::BGarment;
use super::part::BMat;
use crate::colour::{INK, Rgb, WHITE};
use crate::kit::BFootwear;
use crate::kit::sneaker::{BShoeTrim, BSneakerSpec};
use glam::Vec3;

/// A boot's collar: under the ankle, as football boots are cut.
const BOOT_COLLAR: f32 = 0.075;

/// The pair, or nothing barefoot.
pub fn shoes(body: &BBody, footwear: BFootwear, spec: &BSneakerSpec, shirt: Rgb, trim: Rgb) -> Option<BGarment> {
    let collar = match footwear {
        BFootwear::Barefoot => return None,
        BFootwear::Boots => BOOT_COLLAR,
        BFootwear::Shoes => spec.collar,
    };
    // Each foot's line from heel to toe tip.
    let foot = |side: &str| -> Option<(Vec3, Vec3)> {
        let ankle = body.head(&format!("DEF-foot.{side}"))?;
        let tip = body.tail(&format!("DEF-toe.{side}"))?;
        // The heel sits behind the ankle by about a fifth of the foot.
        let along = Vec3::new(tip.x - ankle.x, tip.y - ankle.y, 0.0);
        Some((Vec3::new(ankle.x, ankle.y, 0.0) - along * 0.25, Vec3::new(tip.x, tip.y, 0.0)))
    };
    let (left, right) = (foot("L")?, foot("R")?);
    let frame = |p: Vec3| {
        let (heel, tip) = if p.x >= 0.0 { left } else { right };
        let line = tip - heel;
        let along = ((Vec3::new(p.x, p.y, 0.0) - heel).dot(line) / line.length_squared()).clamp(0.0, 1.0);
        // Outward: away from her midline, on either foot.
        let centre = heel + line * along;
        let outside = (p.x - centre.x) * p.x.signum();
        (along, outside)
    };
    let sole_top = spec.sole;
    let slot = |c: Vec3| -> u16 {
        if c.z < sole_top {
            return 1;
        }
        let (along, outside) = frame(c);
        let up = (c.z / collar).clamp(0.0, 1.0);
        let trim_here = match spec.trim {
            BShoeTrim::Toe => along > 0.78,
            BShoeTrim::Heel => along < 0.2,
            BShoeTrim::Blade => ((along - 0.35) - up * 0.45).abs() < 0.09,
            BShoeTrim::Collar => c.z > collar - 0.028,
            BShoeTrim::Fade => up < 0.45,
            BShoeTrim::Split => outside > 0.005,
            BShoeTrim::Panels => (c.z > 0.06 && (0.45..0.78).contains(&along)) || along < 0.18,
            BShoeTrim::GlowSole => false,
        };
        if trim_here { 2 } else { 0 }
    };
    let cut = body.bisect(2, collar).bisect(2, sole_top);
    let piece = cut.piece(
        |face, c| c.z < collar && cut.share(face, &["DEF-upper_arm", "DEF-forearm", "DEF-hand"]) < 0.1,
        // Thicker toward the floor: the sole's depth under the foot.
        |p| 0.011 + 0.013 * ((0.03 - p.z) / 0.03).max(0.0),
        |p| [frame(p).0, p.z / collar],
        slot,
    );
    let upper = if spec.coloured { shirt } else { WHITE };
    let sole = if spec.trim == BShoeTrim::GlowSole { BMat::glowing("Sole", trim, 2.5) } else { BMat::plain("Sole", INK, 0.7) };
    let name = if footwear == BFootwear::Boots { "Boots" } else { "Sneakers" };
    Some(BGarment { name: name.into(), piece, materials: vec![BMat::plain("Upper", upper, 0.45), sole, BMat::plain("Shoe Trim", trim, 0.45)] })
}
