//! The bowl: terraces stepping up and back from the floor's edge, each a
//! band round the ring with a riser in front, the top one closed by a back
//! wall — and a seat every pitch along every terrace, facing the field.

use super::super::mesh::BMesh;
use super::super::part::{BMat, BPart};
use super::super::ring::BRing;
use super::super::seat::{self, PITCH};
use super::{BInstances, BPlace, BVenueBuild};
use crate::choice::BChoice;
use glam::{Mat4, Vec2, Vec3};
use std::sync::Arc;

/// A terrace's depth and its rise over the one below.
pub const BACK: f32 = 0.85;
pub const RISE: f32 = 0.45;
/// From the floor's edge to the first terrace's front.
pub const GAP: f32 = 0.6;
/// Segments in each quarter corner.
const CORNER: usize = 12;

/// The bowl's extent, for what is built round it.
#[derive(Clone, Copy, Debug)]
pub struct BBowl {
    /// The floor's half extents.
    pub half: [f32; 2],
    /// From the floor's edge to the back of the top terrace.
    pub depth: f32,
    /// The top terrace's height.
    pub top: f32,
    pub tiers: u8,
}

/// Lays the terraces and the seats into `out`.
pub fn bowl(choice: &BChoice, out: &mut BVenueBuild) -> BBowl {
    let sport = choice.sport.spec();
    let venue = choice.venue.spec();
    let theme = choice.theme.spec();
    let half = sport.field.floor_half();
    let tiers = venue.tiers;
    let ring = |d: f32| BRing::rounded(half, d, CORNER);
    let concrete = |i: usize| theme.seats[i % 4].mix(theme.apron, 0.6);
    for i in 0..tiers as usize {
        let (d0, d1) = (GAP + i as f32 * BACK, GAP + (i + 1) as f32 * BACK);
        let (z0, z1) = (i as f32 * RISE, (i + 1) as f32 * RISE);
        let (front, back) = (ring(d0), ring(d1));
        let mut m = BMesh::default();
        let n = front.points().len();
        let p = |r: &BRing, k: usize, z: f32| {
            let q = r.points()[k % n];
            Vec3::new(q.x, q.y, z)
        };
        for k in 0..n {
            let j = k + 1;
            // The tread, facing up, and the riser in front of it, facing
            // the field.
            m.quad(p(&front, k, z1), p(&back, k, z1), p(&back, j, z1), p(&front, j, z1));
            m.quad(p(&front, k, z0), p(&front, k, z1), p(&front, j, z1), p(&front, j, z0));
            if i + 1 == tiers as usize {
                m.quad(p(&back, j, 0.0), p(&back, j, z1), p(&back, k, z1), p(&back, k, 0.0));
            }
        }
        out.parts.push(BPart::new(format!("Terrace {i}"), m, BMat::plain(format!("Terrace {}", i % 4), concrete(i), 0.85), Mat4::IDENTITY));
        // The seats: set a little behind the tread's middle.
        let row = ring(d0 + BACK * 0.55);
        let count = (row.length() / PITCH).floor() as usize;
        for (q, n) in row.spaced(count) {
            out.places.push(BPlace { at: Vec3::new(q.x, q.y, z1 + seat::PAN), heading: facing(n), tier: i as u8 });
        }
    }
    // One instance list per seat colour, cycled tier by tier.
    let spec = choice.seat().spec();
    let mesh = Arc::new(seat::seat(spec));
    for (c, colour) in theme.seats.iter().enumerate() {
        let at: Vec<Mat4> = out
            .places
            .iter()
            .filter(|p| p.tier as usize % 4 == c)
            .map(|p| Mat4::from_translation(p.at - Vec3::Z * seat::PAN) * Mat4::from_rotation_z(p.heading))
            .collect();
        if !at.is_empty() {
            let material = Arc::new(BMat::plain(format!("Seat {c}"), *colour, 0.45));
            out.seats.push(BInstances { name: format!("Seats {c}"), mesh: mesh.clone(), material, at });
        }
    }
    BBowl { half, depth: GAP + tiers as f32 * BACK, top: tiers as f32 * RISE, tiers }
}

/// The turn about z that brings a seat's −y round to face along −`n`
/// (inward, to the field).
pub fn facing(n: Vec2) -> f32 {
    (-n.x).atan2(n.y)
}
