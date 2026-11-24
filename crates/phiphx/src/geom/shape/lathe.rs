//! Shapes turned about their up axis: a profile of rings swept round.
//! Sphere, cylinder, cone, capsule and torus are each a profile.

use super::part::{Frame, ShapeMesh};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// One ring of a profile: its radius, its height, and the surface normal
/// there as (outward, up).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ring {
    pub radius: f32,
    pub y: f32,
    pub normal: [f32; 2],
}

/// A profile: strips of rings, each strip joined ring to ring; a hard edge
/// is where one strip ends and the next begins.
pub type Profile = Vec<Vec<Ring>>;

/// Sweep `profile` round `segment` steps into `mesh`, placed by `frame`.
/// A ring of radius 0 is a pole: its faces are triangles.
pub fn sweep(mesh: &mut ShapeMesh, profile: &Profile, segment: usize, frame: &Frame) {
    let segment = segment.max(3);
    let step = TAU / segment as f32;
    let at = |r: &Ring, angle: f32| {
        let (s, c) = angle.sin_cos();
        let p = frame.point([r.radius * c, r.y, -r.radius * s]);
        let n = frame.turn([r.normal[0] * c, r.normal[1], -r.normal[0] * s]);
        (p, n)
    };
    for strip in profile {
        for pair in strip.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            for j in 0..segment {
                let (t0, t1) = (j as f32 * step, ((j + 1) % segment) as f32 * step);
                let mid = (j as f32 + 0.5) * step;
                match (a.radius == 0.0, b.radius == 0.0) {
                    (true, true) => {}
                    (true, false) => mesh.face(&[at(a, mid), at(b, t1), at(b, t0)]),
                    (false, true) => mesh.face(&[at(a, t0), at(a, t1), at(b, mid)]),
                    (false, false) => mesh.face(&[at(a, t0), at(a, t1), at(b, t1), at(b, t0)]),
                }
            }
        }
    }
}

/// A ring on a sphere of `radius` at latitude `lat` (radians), lifted by `lift`.
fn latitude(radius: f32, lat: f32, lift: f32) -> Ring {
    let (s, c) = lat.sin_cos();
    let r = match c.abs() < 1e-6 {
        true => 0.0,
        false => radius * c,
    };
    Ring { radius: r, y: radius * s + lift, normal: [c, s] }
}

/// A sphere of `radius`, `ring` steps pole to pole.
pub fn sphere(radius: f32, ring: usize) -> Profile {
    let ring = ring.max(2);
    vec![(0..=ring).map(|k| latitude(radius, -FRAC_PI_2 + PI * k as f32 / ring as f32, 0.0)).collect()]
}

/// A capsule: hemispheres of `radius` on a straight middle, `height` end to end.
pub fn capsule(radius: f32, height: f32, ring: usize) -> Profile {
    let half = (ring.max(2) / 2).max(1);
    let lift = (height / 2.0 - radius).max(0.0);
    let lat = |k: usize| -FRAC_PI_2 + FRAC_PI_2 * k as f32 / half as f32;
    let lower = (0..=half).map(|k| latitude(radius, lat(k), -lift));
    let upper = (half..=2 * half).map(|k| latitude(radius, lat(k), lift));
    vec![lower.chain(upper).collect()]
}

/// A closed cylinder of `radius` and `height`, flat ends.
pub fn cylinder(radius: f32, height: f32) -> Profile {
    let h = height / 2.0;
    let (down, out, up) = ([0.0, -1.0], [1.0, 0.0], [0.0, 1.0]);
    vec![
        vec![Ring { radius: 0.0, y: -h, normal: down }, Ring { radius, y: -h, normal: down }],
        vec![Ring { radius, y: -h, normal: out }, Ring { radius, y: h, normal: out }],
        vec![Ring { radius, y: h, normal: up }, Ring { radius: 0.0, y: h, normal: up }],
    ]
}

/// A cone of base `radius` and `height`, flat base.
pub fn cone(radius: f32, height: f32) -> Profile {
    let h = height / 2.0;
    let l = (height * height + radius * radius).sqrt();
    let side = [height / l, radius / l];
    vec![
        vec![Ring { radius: 0.0, y: -h, normal: [0.0, -1.0] }, Ring { radius, y: -h, normal: [0.0, -1.0] }],
        vec![Ring { radius, y: -h, normal: side }, Ring { radius: 0.0, y: h, normal: side }],
    ]
}

/// A torus `outer` across, its tube `tube` thick, `ring` steps round the tube.
pub fn torus(outer: f32, tube: f32, ring: usize) -> Profile {
    let ring = ring.max(3);
    let (r, centre) = (tube / 2.0, (outer - tube) / 2.0);
    let rings = (0..=ring).map(|k| {
        let (s, c) = (TAU * (k % ring) as f32 / ring as f32).sin_cos();
        Ring { radius: centre + r * c, y: r * s, normal: [c, s] }
    });
    vec![rings.collect()]
}
