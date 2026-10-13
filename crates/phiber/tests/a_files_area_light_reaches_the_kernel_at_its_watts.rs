//! A file's area light reaches the kernel at the watts its UI shows —
//! measured against Cycles rendering the same file
//! (`truth/blender_area_light.py`, `rendering.md` §71).
//!
//! `Lamp.energy` in the DNA is 4/pi of the watts the UI shows for an area
//! light (batch 173: exactly 4/pi, independent of shape and size), while
//! the kernel wants watts — `phiray/tests/lamp_power.rs` fed it 100 W by
//! hand and matched Cycles. [`BLamp::record`] is where the two
//! vocabularies meet, so the conversion belongs there: an area light's
//! stored energy comes down by pi/4 on its way to `Light::color`, and
//! every path inherits it, because the viewport's Rendered mode and an F12
//! both build their lights through `rendered::world::lit`.
//!
//! Before the conversion, a file's area light reached the kernel 4/pi
//! (27%) too bright: this fixture measured 2.430 against Cycles' 1.862
//! (31% out — the constant plus the lobe difference the tolerance carries).
//!
//! [`BLamp::record`]: phiber::v2::view::draw::canvas::view3d::rendered::BLamp::record

use phiber::phiphx::Phx;
use phiber::v2::view::draw::canvas::view3d::rendered::BLamp;
use phiray::{Atlas, Camera, Light, LIGHTS, Mode, Ray, Scene, Tracer, Tri};

const W: u32 = 32;
/// Cycles' own radiance at the centre pixel, rendering **this fixture**
/// (`truth/blender_area_light.py`, Blender 5.2.2, 256 samples, one bounce).
/// The hand-fed lamp of the same watts — same rig, scene built in the
/// script — gave 1.861637 under 5.0.1 (`lamp_power.rs`
/// `AREA_100W_1M_AT_2M`), so the two numbers also say the file-read path
/// and the hand-fed one agree across Blender's own versions.
const AREA_FILE_100W_AT_2M: f32 = 1.861_612;
/// The tolerance `lamp_power.rs` gives the same lamp fed by hand: the disk
/// form factor is exact and the lobe is what is left, so a *constant*
/// being wrong — the 4/pi this test exists for — is 27%, not 6%.
const TOLERANCE: f32 = 0.06;

fn fixture() -> phiber::phiphx::scene::SceneModel {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures/AREALIGHT.blend");
    Phx::open(std::path::Path::new(path)).expect("fixture loads")
}

/// A 20 x 20 m plane at z = 0, albedo 0.8, as rough as a surface gets so
/// its lobe is Lambert and nothing else — the measuring surface
/// `truth/blender_lamp_power.py` sets up and the fixture's own plane is.
fn plane() -> Vec<Tri> {
    let n = [0.0, 0.0, 1.0];
    let q = [[-10.0, -10.0, 0.0], [10.0, -10.0, 0.0], [10.0, 10.0, 0.0], [-10.0, 10.0, 0.0]];
    let c = [0.8, 0.8, 0.8, 1.0];
    vec![
        Tri::new([q[0], q[1], q[2]], [n; 3], c).with_pbr(1.0, 0.0),
        Tri::new([q[0], q[2], q[3]], [n; 3], c).with_pbr(1.0, 0.0),
    ]
}

/// A black world: the lamp is the only light.
fn dark() -> Atlas {
    let (w, h) = (8u32, 4u32);
    let rgb = vec![0.0f32; (w * h * 3) as usize];
    let mut atlas = Atlas::default();
    atlas.add_hdr(w, h, &rgb);
    atlas.add_env_cdf(w, h, &rgb);
    atlas.add_env_lighting(w, h, &rgb);
    atlas
}

/// Straight down on the origin, orthographic, 2 m across — the frame the
/// fixture's own camera sets (`ortho_scale = 2.0`).
fn overhead(lamp: Light) -> Camera {
    let mut lights = [Light::default(); LIGHTS];
    lights[0] = lamp;
    Camera {
        right: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        back: [0.0, 0.0, 1.0],
        pivot: [0.0, 0.0, 0.0],
        dist: 10.0,
        units: 1.0,
        persp: 0,
        scale: 0.5,
        aspect: 1.0,
        sky: [0.0; 4],
        env: [0.0, 0.0, 0.0, 1.0],
        envmap: 0,
        env_cdf: 0,
        env_hidden: 0,
        preview: [0; 4],
        count: 1,
        lights,
        ..Default::default()
    }
}

/// The mean linear radiance over the middle of the frame, where the lamp
/// is straight overhead and the geometry has nothing to say.
fn centre_radiance(gpu: &phiray::Gpu, cam: Camera) -> f32 {
    let mut tr = Tracer::from_scene(gpu, Scene::from_tris(&plane(), &dark())).expect("scene");
    tr.set_mode(Mode::Rendered);
    tr.set_samples(256);
    while !tr.sample(gpu, cam, W, W).1 {}
    let lin = tr.radiance(gpu);
    let (lo, hi) = (W / 2 - 2, W / 2 + 2);
    let mut sum = 0.0;
    let mut n = 0.0;
    for y in lo..hi {
        for x in lo..hi {
            sum += lin[(y * W + x) as usize][0];
            n += 1.0;
        }
    }
    sum / n
}

/// The file stores 4/pi of the watts the UI shows, and the record carries
/// the watts. Both halves are needed: the first says the fixture really
/// holds Blender's pre-multiplied field (127.324 for the 100 W the panel
/// shows — batch 173's measurement, present in this file), the second says
/// the record converted it — which is the one line this test exists for.
#[test]
fn the_file_stores_four_over_pi_and_the_record_carries_the_watts() {
    let model = fixture();
    let l = model.lights.iter().find(|l| l.name == "Area").unwrap_or_else(|| panic!("the fixture's area light"));
    let watts = l.energy * std::f32::consts::FRAC_PI_4;
    assert!((watts - 100.0).abs() < 1e-2, "the DNA field is 4/pi of the UI's 100 W: {} -> {}", l.energy, watts);
    let r = l.record();
    assert_eq!((r.kind, r.shape), (3, phiray::AREA_SQUARE));
    assert_eq!((r.size, r.size_y), (1.0, 1.0), "a 1 m square, as the file says");
    for c in r.color {
        assert!((c - 100.0).abs() < 1e-2, "the record carries the watts, not the stored 4/pi of them: {c}");
    }
}

/// The whole path, against Cycles rendering the same file: the stored
/// energy, through the record, into the kernel, onto the measuring plane —
/// 1.86 in scene-linear radiance, not the 2.37 the stored number gives.
#[test]
fn a_files_area_light_lights_the_plane_as_cycles_lights_it() {
    let Some(gpu) = Ray::gpu() else { return eprintln!("no GPU adapter; skipped") };
    let model = fixture();
    let lamp = model.lights.iter().find(|l| l.name == "Area").unwrap_or_else(|| panic!("the fixture's area light")).record();
    let got = centre_radiance(&gpu, overhead(lamp));
    eprintln!("file-read area light {got:.4}, Cycles {AREA_FILE_100W_AT_2M:.4}");
    let off = (got / AREA_FILE_100W_AT_2M - 1.0).abs();
    assert!(off < TOLERANCE, "the file's 100 W area light at 2 m over albedo 0.8: {got:.3}, Cycles {AREA_FILE_100W_AT_2M:.3} ({:.0}% out)", off * 100.0);
}
