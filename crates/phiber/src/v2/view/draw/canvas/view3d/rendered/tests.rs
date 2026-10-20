//! A real file through the Rendered path: saved view → phiray image.

use super::scene;
use super::world;
use crate::phiphx::phx::Phx;
use crate::v2::scene::{BShading, scene_source, set_loaded_scene, view_pan, view_pitch, view_yaw, view_zoom};
use crate::v2::state::BState;
use gpui::{black, white};

/// Test stand-in for the theme's grid alpha.
const GRID_ALPHA: f32 = 0.35;

#[test]
fn fixture_renders_through_phiray() {
    let Some(gpu) = phiray::Gpu::new() else { return };
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures");
    let assets = root.join("../../../phiber/assets/production");
    let names = ["FLORIS_BOLDRIOD_RIG.blend", "THYMUS-BOLDROID-WAVE-003.blend", "TEXTURED-PACKED.blend", "MATERIALS-BSDF.blend", "NORMAL-MAP.blend"];
    // Production files (kept outside the repo) join in when present.
    // Any other files: PHIBER_TEST_FILES="a.blend;b.blend".
    let extra: Vec<std::path::PathBuf> = std::env::var("PHIBER_TEST_FILES")
        .unwrap_or_default()
        .split(';')
        .filter(|p| !p.trim().is_empty())
        .map(|p| std::path::PathBuf::from(p.trim()))
        .collect();
    let files = names
        .map(|n| root.join(n))
        .into_iter()
        .chain(["LAND-TYPE-15-01.blend", "Mannequins-Female--Pose-05.blend"].map(|n| assets.join(n)))
        .chain(extra);
    for path in files.filter(|p| p.exists()) {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let name = name.as_str();
        let opened = web_time::Instant::now();
        let model = Phx::open(&path).expect("open");
        eprintln!("{name}: opened in {:?}, {} objects", opened.elapsed(), model.objects.len());
        let mut s = BState::default();
        if let Some(v) = model.ui.view.as_ref() {
            crate::v2::app::load_ui::apply_view(&mut s, v);
        }
        let (model_world, model_lights) = (model.world.horizon_color, model.lights.clone());
        set_loaded_scene(Some(model));
        let objects = scene_source(&s);
        let grey = white();
        let palette = super::super::mesh::BMeshPalette {
            shading: BShading::Rendered, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false,
        };
        let built = scene::build(&objects, &palette);
        if name.starts_with("TEXTURED") {
            assert!(!built.atlas.is_empty(), "packed texture reaches phiray");
        }
        let (count, vert, biggest) = (built.len(), built.pos.len(), built.biggest());
        eprintln!("{name}: scene built, peak {} MB", peak_mb());
        let Some(mut tracer) = phiray::Tracer::from_scene(&gpu, built) else {
            // Larger than this device allows in one binding: the painter falls back.
            assert!(count > 1_000_000, "{name}: a small scene was refused");
            eprintln!("{name}: {count} tris ({vert} verts, {} MB buffer) exceed this GPU's {} MB binding limit; mesh painter fallback", biggest >> 20, gpu.max_binding() >> 20);
            set_loaded_scene(None);
            continue;
        };
        let setup = crate::v2::scene::camera::setup_from_state(&s);
        let (w, h) = (320u32, 180u32);
        let look = world::BRayLook { sky: black(), grid: white().opacity(GRID_ALPHA), grid_hide: 0, world: Some(model_world), lights: model_lights.clone(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
        let mut cam = scene::camera(view_yaw(&s), view_pitch(&s), view_pan(&s), view_zoom(&s), setup, w as f32 / h as f32, &look);
        // What met an object, by the film's alpha: with the file's world as
        // the background (`world::lit`), a dim object on a dim world is the
        // background's colour give or take, and that is Blender's picture.
        cam.film[0] = 1;
        let t = web_time::Instant::now();
        let mut px = tracer.render(&gpu, cam, w, h).0;
        for _ in 1..phiray::Tracer::SAMPLES {
            px = tracer.render(&gpu, cam, w, h).0;
        }
        eprintln!("{name}: {count} tris, {vert} verts, frame {:?}", t.elapsed());
        let covered = px.chunks(4).filter(|p| p[3] > 0).count();
        assert!(covered > (w * h / 50) as usize, "{name}: only {covered} object pixels");
        if let Ok(dir) = std::env::var("PHIRAY_DUMP") {
            let mut ppm = format!("P6 {w} {h} 255\n").into_bytes();
            ppm.extend(px.chunks(4).flat_map(|c| [c[0], c[1], c[2]]));
            std::fs::write(format!("{dir}/{name}.ppm"), ppm).expect("dump");
        }
        set_loaded_scene(None);
    }
}

/// Peak resident memory in MB (Linux; 0 elsewhere).
fn peak_mb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| s.lines().find(|l| l.starts_with("VmHWM")).and_then(|l| l.split_whitespace().nth(1)?.parse().ok()))
        .map_or(0, |kb: u64| kb / 1024)
}

#[test]
fn render_thread_refines_latest_view() {
    use super::worker::{BRayJob, BRayWorker, BRayShot};
    use crate::v2::scene::{BObject3D, BShape};
    let Some(worker) = BRayWorker::start() else { return };
    let grey = white();
    let palette = super::super::mesh::BMeshPalette {
        shading: BShading::Rendered, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false,
    };
    let cube = BObject3D::Primitive { name: "Cube".into(), shape: BShape::Cube, x: 0.0, y: 0.0 };
    worker.send(BRayJob::Scene(1, 1, vec![cube], palette));
    let s = BState::default();
    let look = world::BRayLook { sky: black(), grid: white().opacity(GRID_ALPHA), grid_hide: 0, world: None, lights: Vec::new(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
    let setup = crate::v2::scene::camera::setup_from_state(&s);
    let cam = |yaw: f32| scene::camera(yaw, 30.0, (0.0, 0.0), 1.0, setup, 1.0, &look);
    // An outdated view first, then the one that must win.
    worker.send(BRayJob::View(1, BRayShot { cam: cam(0.0), w: 32, h: 32, mode: phiray::Mode::Rendered, samples: 32, aa: 8, denoise_from: None, outline: std::sync::Arc::from([]) }));
    worker.send(BRayJob::View(2, BRayShot { cam: cam(45.0), w: 48, h: 48, mode: phiray::Mode::Rendered, samples: 32, aa: 8, denoise_from: None, outline: std::sync::Arc::from([]) }));
    let (mut seq, mut last, mut sizes) = (0, None, Vec::new());
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(60);
    while web_time::Instant::now() < deadline {
        if let Some(f) = worker.take(seq) {
            seq = f.seq;
            let done = f.done && f.view == 2;
            if f.view == 2 && sizes.last() != Some(&(f.w, f.h)) {
                sizes.push((f.w, f.h));
            }
            last = Some(f);
            if done {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let f = last.expect("frames arrive");
    assert!(f.done && f.view == 2 && (f.w, f.h) == (48, 48), "settles on the newest view");
    assert_eq!(f.bgra.len(), 48 * 48 * 4);
    // A navigated view's first image is small and grows to full size
    // (`BCoarse`: an eighth, a quarter, a half, then 48 × 48 — the exact
    // ladder is `a_navigated_view_starts_at_an_eighth_and_doubles_to_full`).
    // The worker keeps only its newest frame, so a poll may miss the small
    // ones — all of them, on a loaded machine; what it does see never
    // shrinks.
    assert!(sizes.windows(2).all(|w| w[0].0 <= w[1].0), "coarse first: {sizes:?}");
}

#[test]
fn a_navigated_view_starts_at_an_eighth_and_doubles_to_full() {
    use super::coarse::BCoarse;
    let mut c = BCoarse::start(phiray::Mode::Rendered);
    let mut sizes = Vec::new();
    while !c.full() {
        sizes.push(c.pixels(1600, 900));
        c.next();
    }
    assert_eq!(sizes, [(200, 112), (400, 225), (800, 450)]);
    assert_eq!(c.pixels(1600, 900), (1600, 900));
    let mut solid = BCoarse::start(phiray::Mode::Solid);
    assert_eq!(solid.pixels(1600, 900), (400, 225), "Workbench is one dispatch: a quarter is enough");
    solid.next();
    solid.next();
    assert!(solid.full());
    c.next();
    assert!(c.full(), "full stays full");
}

/// Properties → Render → Light Paths → Max Bounces → Diffuse has to reach the
/// `Camera` the tracer runs, or the panel is decoration.
#[test]
fn the_panels_bounce_count_reaches_the_camera() {
    use crate::v2::scene::BRenderSet;
    let look = world::BRayLook {
        sky: black(),
        grid: white().opacity(GRID_ALPHA),
        grid_hide: 0,
        world: None,
        lights: Vec::new(),
        wire: None,
        view: [0.0, 0.0, 1.0, 0.0],
        environment: None,
        set: BRenderSet { diffuse_bounces: 9, ..Default::default() },
        studio: false,
    };
    let cam = scene::camera(0.0, 0.0, (0.0, 0.0), 1.0, Default::default(), 1.0, &look);
    assert_eq!(cam.bounces, 9);
}

/// A moved object reaches the render thread as a new pose of the scene it
/// holds (the same shape: `Tracer::repose`, no build) and draws what a
/// thread given the moved scene from the start draws.
#[test]
fn a_moved_object_draws_as_a_scene_built_there() {
    use super::key::BRayKey;
    use super::worker::{BRayFrame, BRayJob, BRayShot, BRayWorker};
    use crate::v2::scene::{BObject3D, BShape};
    let grey = white();
    let palette = super::super::mesh::BMeshPalette { shading: BShading::Solid, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false };
    let cubes = |x: f32| vec![BObject3D::Primitive { name: "Cube".into(), shape: BShape::Cube, x, y: 0.0 }, BObject3D::Primitive { name: "Ball".into(), shape: BShape::Sphere, x: -2.0, y: 0.0 }];
    let scene_of = |x: f32| BRayJob::Scene(BRayKey::of(&cubes(x), &palette), BRayKey::shape(&cubes(x), &palette), cubes(x), palette);
    assert_eq!(BRayKey::shape(&cubes(0.0), &palette), BRayKey::shape(&cubes(1.5), &palette), "one shape, two poses");
    let look = world::BRayLook { sky: black(), grid: white().opacity(GRID_ALPHA), grid_hide: 0, world: None, lights: Vec::new(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
    let setup = crate::v2::scene::camera::setup_from_state(&BState::default());
    let shot = BRayShot { cam: scene::camera(20.0, 30.0, (0.0, 0.0), 1.0, setup, 1.0, &look), w: 40, h: 40, mode: phiray::Mode::Solid, samples: 1, aa: 8, denoise_from: None, outline: std::sync::Arc::from([]) };
    let finished = |worker: &BRayWorker, after: u64| -> BRayFrame {
        let deadline = web_time::Instant::now() + std::time::Duration::from_secs(60);
        while web_time::Instant::now() < deadline {
            if let Some(f) = worker.take(after).filter(|f| f.done) {
                return f;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("no finished frame");
    };
    let (Some(moved), Some(built)) = (BRayWorker::start(), BRayWorker::start()) else { return };
    moved.send(scene_of(0.0));
    moved.send(BRayJob::View(1, shot.clone()));
    let before = finished(&moved, 0);
    moved.send(scene_of(1.5));
    let after = finished(&moved, before.seq);
    built.send(scene_of(1.5));
    built.send(BRayJob::View(1, shot));
    assert_ne!(before.bgra, after.bgra, "the cube moved");
    assert_eq!(after.bgra, finished(&built, 0).bgra, "as a scene built there draws it");
}
