//! The goals: what each sport scores into, built at both ends of its field
//! (or across the middle, for a net) from the sport's own measurements.

use super::mesh::BMesh;
use super::part::{BMat, BPart};
use crate::colour::{INK, Rgb, WHITE};
use crate::sport::{BGoal, BSportSpec};
use crate::theme::BThemeSpec;
use glam::{Mat4, Vec3};

/// Every goal of the sport, in the theme's accent where a goal has a
/// colour of its own (a rim, a post's padding).
pub fn goals(sport: &BSportSpec, theme: &BThemeSpec) -> Vec<BPart> {
    let [hx, hy] = sport.field.half;
    let steel = BMat::metal("Goal Steel", Rgb::hex(0xC8CCD2), 0.3);
    let accent = BMat::plain("Goal Accent", theme.accent, 0.4);
    let net = BMat::plain("Net", WHITE, 0.8);
    let mut parts = Vec::new();
    match sport.goal {
        BGoal::Hoop { height, out, rim } => {
            for side in [-1.0f32, 1.0] {
                let name = if side > 0.0 { "East" } else { "West" };
                let x = side * (hx - out);
                // The board stands 1.2 m in from the end line, its face
                // 0.15 m behind the rim's back; the support stands off court.
                let board_x = side * (hx - 1.2);
                let base_x = side * (hx + 1.6);
                parts.push(BPart::new(format!("Backboard {name}"), BMesh::cuboid(Vec3::new(0.04, 1.8, 1.05)), BMat::plain("Backboard", Rgb::hex(0xEEF2F5), 0.1), Mat4::from_translation(Vec3::new(board_x, 0.0, height + 0.3))));
                parts.push(BPart::new(format!("Rim {name}"), BMesh::torus(rim + 0.009, 0.009, 32, 8), accent.clone(), Mat4::from_translation(Vec3::new(x, 0.0, height))));
                parts.push(BPart::new(format!("Net {name}"), BMesh::cylinder(rim, rim * 0.65, 0.42, 16), net.clone(), Mat4::from_translation(Vec3::new(x, 0.0, height - 0.42))));
                parts.push(BPart::new(format!("Hoop Arm {name}"), BMesh::rod(Vec3::new(base_x, 0.0, height + 0.05), Vec3::new(board_x, 0.0, height + 0.05), 0.06), steel.clone(), Mat4::IDENTITY));
                parts.push(BPart::new(format!("Hoop Pole {name}"), BMesh::rod(Vec3::new(base_x, 0.0, 0.9), Vec3::new(base_x, 0.0, height + 0.05), 0.09), steel.clone(), Mat4::IDENTITY));
                parts.push(BPart::new(format!("Hoop Base {name}"), BMesh::cuboid(Vec3::new(1.2, 1.4, 0.9)), accent.clone(), Mat4::from_translation(Vec3::new(base_x, 0.0, 0.45))));
            }
        }
        BGoal::Ring { height, rim, reach } => {
            for side in [-1.0f32, 1.0] {
                let name = if side > 0.0 { "East" } else { "West" };
                let post = side * hx;
                let ring = side * (hx - reach - rim);
                parts.push(BPart::new(format!("Goal Post {name}"), BMesh::rod(Vec3::new(post, 0.0, 0.0), Vec3::new(post, 0.0, height), 0.04), steel.clone(), Mat4::IDENTITY));
                parts.push(BPart::new(format!("Post Pad {name}"), BMesh::cylinder(0.09, 0.09, 1.8, 16), accent.clone(), Mat4::from_translation(Vec3::new(post, 0.0, 0.0))));
                parts.push(BPart::new(format!("Ring Arm {name}"), BMesh::rod(Vec3::new(post, 0.0, height), Vec3::new(side * (hx - reach), 0.0, height), 0.012), steel.clone(), Mat4::IDENTITY));
                parts.push(BPart::new(format!("Ring {name}"), BMesh::torus(rim + 0.008, 0.008, 32, 8), steel.clone(), Mat4::from_translation(Vec3::new(ring, 0.0, height))));
                parts.push(BPart::new(format!("Net {name}"), BMesh::cylinder(rim, rim * 0.7, 0.38, 16), net.clone(), Mat4::from_translation(Vec3::new(ring, 0.0, height - 0.38))));
            }
        }
        BGoal::Net { height, post } => {
            let y = hy + post;
            for side in [-1.0f32, 1.0] {
                let name = if side > 0.0 { "North" } else { "South" };
                parts.push(BPart::new(format!("Net Post {name}"), BMesh::rod(Vec3::new(0.0, side * y, 0.0), Vec3::new(0.0, side * y, height + 0.2), 0.05), accent.clone(), Mat4::IDENTITY));
                // The antennae: 1.8 m rods in red and white bands over the
                // sidelines.
                parts.push(BPart::new(format!("Antenna {name}"), BMesh::rod(Vec3::new(0.0, side * hy, height - 0.9), Vec3::new(0.0, side * hy, height + 0.8), 0.005), BMat::plain("Antenna", Rgb::hex(0xE53935), 0.5), Mat4::IDENTITY));
            }
            // The net: a dark mesh a metre deep under a white top band.
            parts.push(BPart::new("Net", BMesh::cuboid(Vec3::new(0.01, 2.0 * y, 1.0)), BMat::plain("Net Mesh", INK, 0.9), Mat4::from_translation(Vec3::new(0.0, 0.0, height - 0.5))));
            parts.push(BPart::new("Net Band", BMesh::cuboid(Vec3::new(0.012, 2.0 * hy, 0.07)), net.clone(), Mat4::from_translation(Vec3::new(0.0, 0.0, height - 0.035))));
        }
        BGoal::Goal { width, height, depth } => {
            for side in [-1.0f32, 1.0] {
                let name = if side > 0.0 { "East" } else { "West" };
                let x = side * hx;
                let back = side * (hx + depth);
                let (w, r) = (width / 2.0 + 0.06, 0.06);
                let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
                for (a, b) in [
                    (p(x, -w, 0.0), p(x, -w, height + r)),
                    (p(x, w, 0.0), p(x, w, height + r)),
                    (p(x, -w, height + r), p(x, w, height + r)),
                ] {
                    parts.push(BPart::new(format!("Goal Frame {name}"), BMesh::rod(a, b, r), BMat::plain("Goal Frame", WHITE, 0.3), Mat4::IDENTITY));
                }
                // The net: its back and roof as thin panels.
                parts.push(BPart::new(format!("Goal Net {name}"), BMesh::cuboid(Vec3::new(0.01, 2.0 * w, height)), net.clone(), Mat4::from_translation(Vec3::new(back, 0.0, height / 2.0))));
                parts.push(BPart::new(format!("Goal Roof {name}"), BMesh::cuboid(Vec3::new(depth, 2.0 * w, 0.01)), net.clone(), Mat4::from_translation(Vec3::new((x + back) / 2.0, 0.0, height))));
                for s in [-1.0f32, 1.0] {
                    parts.push(BPart::new(format!("Goal Side {name}"), BMesh::cuboid(Vec3::new(depth, 0.01, height)), net.clone(), Mat4::from_translation(Vec3::new((x + back) / 2.0, s * w, height / 2.0))));
                }
            }
        }
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sport::BSport;
    use crate::theme::BTheme;

    #[test]
    fn every_sport_gets_its_goals_at_its_own_height() {
        for sport in BSport::ALL {
            let spec = sport.spec();
            let parts = goals(spec, BTheme::Bloom.spec());
            assert!(!parts.is_empty(), "{}", spec.label);
            let top = parts.iter().flat_map(|p| p.mesh.positions.iter().map(|v| p.at.transform_point3(Vec3::from(*v)).z)).fold(0.0, f32::max);
            let want = match spec.goal {
                BGoal::Hoop { height, .. } => height + 0.3 + 0.525,
                BGoal::Ring { height, .. } => height + 0.03,
                // The antennae rise 0.8 m over the net (FIVB).
                BGoal::Net { height, .. } => height + 0.8,
                BGoal::Goal { height, .. } => height + 0.12,
            };
            assert!((top - want).abs() < 0.06, "{}: the goals reach {top}, not {want}", spec.label);
        }
    }
}
