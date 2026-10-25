//! The crowd: a fan in every seat the choice fills, drawn as the Olympic
//! pictograms draw people — a ball for the head, smooth rods for the limbs,
//! a tapered trunk, no faces — so three thousand of them read as a crowd
//! and not as three thousand people to look at.
//!
//! A fan is a mesh and a palette. The meshes are few: each of the sixteen
//! hairstyles (`look::hair`, the same sets the players wear) on each of
//! three seated poses — hands on knees, clapping, arms up. The palettes are
//! few too: a look's skin and hair with the shirt of the section's team.
//! Every fan is one of those meshes in one of those palettes, seated,
//! scaled to her look's height and turned a little off square; Phiber
//! draws each (mesh, palette) pair as one instanced list.

use super::mesh::BMesh;
use super::part::BMat;
use super::venue::{BInstances, BPlace};
use super::{BImage, field::hash};
use crate::choice::BChoice;
use crate::colour::Rgb;
use crate::look::BLook;
use crate::look::hair::{BHair, BHairSpec, BStrands, BVolume};
use glam::{Mat4, Vec2, Vec3};
use std::collections::BTreeMap;
use std::f32::consts::{PI, TAU};
use std::sync::Arc;

/// How a fan sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BFanPose {
    /// Hands on her knees.
    Seated,
    /// Hands together in front of her chest.
    Clap,
    /// Both arms up in a V.
    Cheer,
}

impl BFanPose {
    pub const ALL: [BFanPose; 3] = [BFanPose::Seated, BFanPose::Clap, BFanPose::Cheer];
}

/// The palette's cells, left to right in its image.
#[derive(Clone, Copy)]
enum Cell {
    Skin,
    Shirt,
    Legs,
    Hair,
    Shoes,
}

const CELLS: u32 = 5;

/// A palette: one fan's colours, painted as a strip the mesh's UVs index.
fn palette(colours: [Rgb; CELLS as usize]) -> BImage {
    BImage::painted(CELLS, 1, |u, _| colours[((u * CELLS as f32) as usize).min(CELLS as usize - 1)])
}

/// `m` with every UV on `cell`'s centre.
fn painted(mut m: BMesh, cell: Cell) -> BMesh {
    let uv = [(cell as u32 as f32 + 0.5) / CELLS as f32, 0.5];
    m.uvs.iter_mut().for_each(|t| *t = uv);
    m
}

/// A limb: a rod, with a ball at its far end where that end is a joint
/// or a hand, so it reads round (an ankle's is inside the shoe).
fn limb(m: &mut BMesh, a: Vec3, b: Vec3, radius: f32, cell: Cell, round: bool) {
    m.merge(&painted(BMesh::rod(a, b, radius), cell), Mat4::IDENTITY);
    if round {
        m.merge(&painted(BMesh::sphere(radius, 6, 4), cell), Mat4::from_translation(b));
    }
}

/// The head's centre and radius, in the fan's space.
const HEAD: Vec3 = Vec3::new(0.0, 0.03, 1.13);
const HEAD_R: f32 = 0.095;

/// A fan's body in `pose`, seated on a pan at z = `seat::PAN` above her
/// origin, facing −y.
fn body(pose: BFanPose) -> BMesh {
    let mut m = BMesh::default();
    let hip = 0.47;
    // The trunk: a tapered tube from the hips to the shoulders, a little
    // flatter front to back than across, leaning back into the seat.
    let trunk = BMesh::cylinder(0.135, 0.165, 0.47, 12);
    let lean = Mat4::from_translation(Vec3::new(0.0, 0.06, hip)) * Mat4::from_rotation_x(-0.08) * Mat4::from_scale(Vec3::new(1.0, 0.68, 1.0));
    m.merge(&painted(trunk, Cell::Shirt), lean);
    // Shoulders and hips rounded off.
    for x in [-1.0, 1.0] {
        m.merge(&painted(BMesh::sphere(0.06, 6, 4), Cell::Shirt), Mat4::from_translation(Vec3::new(x * 0.17, 0.08, 0.92)));
    }
    // Neck and head.
    m.merge(&painted(BMesh::rod(Vec3::new(0.0, 0.07, 0.93), HEAD, 0.042), Cell::Skin), Mat4::IDENTITY);
    m.merge(&painted(BMesh::sphere(HEAD_R, 10, 6), Cell::Skin), Mat4::from_translation(HEAD));
    for x in [-1.0f32, 1.0] {
        // Legs: thighs level forward, shins down to the floor, shoes.
        let hip_at = Vec3::new(x * 0.09, 0.05, hip);
        let knee = Vec3::new(x * 0.11, -0.36, hip + 0.03);
        let ankle = Vec3::new(x * 0.11, -0.42, 0.09);
        limb(&mut m, hip_at, knee, 0.072, Cell::Legs, true);
        limb(&mut m, knee, ankle, 0.05, Cell::Legs, false);
        let shoe = BMesh::cuboid(Vec3::new(0.09, 0.24, 0.08));
        m.merge(&painted(shoe, Cell::Shoes), Mat4::from_translation(Vec3::new(x * 0.11, -0.47, 0.04)));
        // Arms, by the pose.
        let shoulder = Vec3::new(x * 0.19, 0.08, 0.9);
        let (elbow, hand) = match pose {
            BFanPose::Seated => (Vec3::new(x * 0.22, 0.0, 0.65), Vec3::new(x * 0.14, -0.3, hip + 0.1)),
            BFanPose::Clap => (Vec3::new(x * 0.22, -0.1, 0.7), Vec3::new(x * 0.035, -0.33, 0.92)),
            BFanPose::Cheer => (Vec3::new(x * 0.31, 0.02, 1.16), Vec3::new(x * 0.42, -0.04, 1.42)),
        };
        limb(&mut m, shoulder, elbow, 0.045, Cell::Skin, true);
        limb(&mut m, elbow, hand, 0.038, Cell::Skin, true);
    }
    m
}

/// A cap of a sphere about `centre`: from the crown down to `front` of the
/// way to the bottom over the brow and `back` at the nape, smooth,
/// standing `shell` off the head, roughened by `curl`.
fn cap(centre: Vec3, radius: f32, front: f32, back: f32, curl: f32) -> BMesh {
    let (segments, rings) = (10u32, 5u32);
    let mut m = BMesh::default();
    for r in 0..=rings {
        for s in 0..=segments {
            let phi = TAU * s as f32 / segments as f32;
            // Facing −y is the front: blend the cap's reach round the head.
            let toward_back = 0.5 + 0.5 * phi.sin();
            let reach = front + (back - front) * toward_back;
            let theta = PI * reach * r as f32 / rings as f32;
            let n = Vec3::new(theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos());
            let bump = 1.0 + curl * 0.06 * (hash(s as i32 * 3, r as i32 * 5) - 0.5);
            m.vertex(centre + n * radius * bump, n, Vec2::ZERO);
        }
    }
    let row = segments + 1;
    for r in 0..rings {
        for s in 0..segments {
            let (a, b) = (r * row + s, (r + 1) * row + s);
            if r > 0 {
                m.triangle(a, b, a + 1);
            }
            m.triangle(a + 1, b, b + 1);
        }
    }
    m
}

/// The hairstyle on a fan's head, from the same numbers the players' is
/// grown from.
fn hair(spec: &BHairSpec) -> BMesh {
    let mut m = BMesh::default();
    let r = HEAD_R + spec.shell.max(0.004);
    // The hairline: across the brow, down the back to `reach`.
    let back = 0.38 + 0.3 * spec.reach;
    let front = if spec.faded_sides { 0.26 } else { 0.3 };
    m.merge(&painted(cap(HEAD, r, front, back, spec.curl), Cell::Hair), Mat4::IDENTITY);
    match spec.volume {
        BVolume::None => {}
        BVolume::Puff { radius, at, pair } => {
            // `at` up the back from the nape (0) to the top (1).
            let a = PI * (1.0 - at) * 0.5;
            let up = Vec3::new(0.0, a.sin(), a.cos()) * (r * 0.95);
            let spots: &[Vec3] = if pair { &[Vec3::new(-0.75, 0.25, 0.65), Vec3::new(0.75, 0.25, 0.65)] } else { &[Vec3::ZERO] };
            for &side in spots {
                let at = if pair { HEAD + side.normalize() * r } else { HEAD + up };
                m.merge(&painted(BMesh::sphere(radius, 8, 5), Cell::Hair), Mat4::from_translation(at));
            }
        }
    }
    match spec.strands {
        BStrands::None | BStrands::Rows { .. } => {}
        BStrands::Tail { length, width } => {
            let from = HEAD + Vec3::new(0.0, 0.85, 0.2).normalize() * r;
            let to = from + Vec3::new(0.0, 0.05, -length.min(0.3));
            limb(&mut m, from, to, width * 0.5, Cell::Hair, true);
        }
        BStrands::Hanging { count, length, width, .. } => {
            // Round the back half of the head, under the hairline.
            let n = usize::from(count).clamp(4, 6);
            for k in 0..n {
                let phi = PI * (0.08 + 0.84 * k as f32 / (n - 1) as f32);
                let out = Vec3::new(phi.cos(), phi.sin(), -0.1).normalize();
                let from = HEAD + out * r;
                let to = from + Vec3::new(out.x * 0.02, out.y * 0.03, -length.min(0.32));
                m.merge(&painted(BMesh::rod(from, to, width.max(0.012) * 0.5), Cell::Hair), Mat4::IDENTITY);
            }
        }
    }
    m
}

/// Which way a section leans: the home end, the away end, or neither.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Section {
    Home,
    Away,
    Neutral,
}

/// The fans the choice seats in `places`: one instance list for every
/// (pose, hairstyle, look, section) a fan was drawn with.
pub fn crowd(choice: &BChoice, places: &[BPlace]) -> Vec<BInstances> {
    let sport = choice.sport.spec();
    let theme = choice.theme.spec();
    let (home, away) = (choice.home.spec(), choice.away.spec());
    let [fx, _] = sport.field.floor_half();
    // Which seats are filled: the first `crowd` by a hash of the seat, so a
    // smaller crowd thins out evenly rather than emptying a stand.
    let mut order: Vec<usize> = (0..places.len()).collect();
    order.sort_by(|&a, &b| hash(a as i32, 41).total_cmp(&hash(b as i32, 41)));
    order.truncate(choice.crowd as usize);
    order.sort_unstable();
    let mut bodies: BTreeMap<BFanPose, Arc<BMesh>> = BTreeMap::new();
    let mut styles: BTreeMap<(BFanPose, usize), Arc<BMesh>> = BTreeMap::new();
    let mut lists: BTreeMap<(BFanPose, usize, usize, Section), Vec<Mat4>> = BTreeMap::new();
    for i in order {
        let p = places[i];
        let roll = |salt: i32| hash(i as i32, salt);
        // Behind the home end most fans wear the home colour, behind the
        // away end the away colour; along the sides they are mixed.
        let side = p.at.x / fx;
        let section = match roll(3) {
            r if side < -0.3 && r < 0.8 => Section::Home,
            r if side > 0.3 && r < 0.8 => Section::Away,
            r if r < 0.35 => Section::Home,
            r if r < 0.7 => Section::Away,
            _ => Section::Neutral,
        };
        let pose = match roll(5) {
            r if r < 0.5 => BFanPose::Seated,
            r if r < 0.85 => BFanPose::Clap,
            _ => BFanPose::Cheer,
        };
        let style = ((roll(7) * 16.0) as usize).min(15);
        let look = ((roll(11) * 8.0) as usize).min(7);
        let height = BLook::ALL[look].spec().height;
        let turn = (roll(13) - 0.5) * 0.3;
        let at = Mat4::from_translation(p.at - Vec3::Z * super::seat::PAN) * Mat4::from_rotation_z(p.heading + turn) * Mat4::from_scale(Vec3::splat(height));
        lists.entry((pose, style, look, section)).or_default().push(at);
        if !styles.contains_key(&(pose, style)) {
            let base = bodies.entry(pose).or_insert_with(|| Arc::new(body(pose))).clone();
            let mut m = (*base).clone();
            m.merge(&hair(&BHair::ALL[style].spec()), Mat4::IDENTITY);
            styles.insert((pose, style), Arc::new(m));
        }
    }
    // A palette per (look, section): her skin, a hair colour of her look,
    // the section's shirt, and legs and shoes from a short list.
    let legs = [Rgb::hex(0x2b3a55), Rgb::hex(0x3a3a3c), Rgb::hex(0xb9a888), Rgb::hex(0x1d1d1f)];
    let shoes = [Rgb::hex(0xf5f5f7), Rgb::hex(0x1d1d1f), theme.accent];
    let mut materials: BTreeMap<(usize, Section), Arc<BMat>> = BTreeMap::new();
    let mut out = Vec::new();
    for ((pose, style, look, section), at) in lists {
        let material = materials
            .entry((look, section))
            .or_insert_with(|| {
                let spec = BLook::ALL[look].spec();
                let k = look + section as usize;
                let shirt = match section {
                    Section::Home => home.home,
                    Section::Away => away.shirt(&home),
                    Section::Neutral => [theme.primary, theme.secondary][look % 2],
                };
                let name = format!("Fan {} {:?}", spec.label, section);
                let image = palette([spec.skin, shirt, legs[k % legs.len()], spec.hair[k % 3], shoes[k % shoes.len()]]);
                Arc::new(BMat { roughness: 0.6, ..BMat::plain(name, shirt, 0.6) }.with_image(Arc::new(image)))
            })
            .clone();
        out.push(BInstances { name: format!("Fans {pose:?} {} {}", BHair::ALL[style].spec().label, material.name), mesh: styles[&(pose, style)].clone(), material, at });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::venue::venue;

    #[test]
    fn a_full_house_is_three_thousand_fans_from_a_few_meshes() {
        let choice = BChoice::default();
        let built = venue(&choice);
        let fans = crowd(&choice, &built.places);
        let seated: usize = fans.iter().map(|f| f.at.len()).sum();
        assert_eq!(seated, (choice.crowd as usize).min(built.places.len()));
        // Few meshes and few palettes, however many fans.
        let meshes: std::collections::HashSet<*const BMesh> = fans.iter().map(|f| Arc::as_ptr(&f.mesh)).collect();
        let palettes: std::collections::HashSet<*const BMat> = fans.iter().map(|f| Arc::as_ptr(&f.material)).collect();
        assert!(meshes.len() <= 48, "{} meshes", meshes.len());
        assert!(palettes.len() <= 24, "{} palettes", palettes.len());
        // Every style and pose drawn, each fan light enough to draw by the
        // thousand.
        assert!(meshes.len() >= 40);
        for f in &fans {
            assert!(f.mesh.triangle_count() < 1100, "{}: {} triangles", f.name, f.mesh.triangle_count());
        }
        // Each sits in a seat: her pan under her hips.
        for f in &fans {
            for at in &f.at {
                let hip = at.transform_point3(Vec3::new(0.0, 0.05, 0.47));
                assert!(built.places.iter().any(|p| p.at.distance(hip) < 0.15), "{}: hips at {hip:?}", f.name);
            }
        }
    }

    #[test]
    fn a_fan_stands_on_the_floor_and_cheers_above_her_head() {
        for pose in BFanPose::ALL {
            let m = body(pose);
            let low = m.positions.iter().map(|p| p[2]).fold(f32::MAX, f32::min);
            let high = m.positions.iter().map(|p| p[2]).fold(f32::MIN, f32::max);
            assert!(low.abs() < 0.01, "{pose:?}: feet at {low}");
            let top = HEAD.z + HEAD_R;
            if pose == BFanPose::Cheer {
                assert!(high > top + 0.2, "{pose:?}: hands at {high}");
            } else {
                assert!((high - top).abs() < 0.01, "{pose:?}: top at {high}");
            }
        }
    }
}
