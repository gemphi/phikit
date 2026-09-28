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
        let opened = std::time::Instant::now();
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
        let palette = super::BMeshPalette {
            shading: BShading::Rendered, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey,
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
        let look = world::BRayLook { sky: black(), grid: white().opacity(GRID_ALPHA), world: Some(model_world), lights: model_lights.clone(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
        let cam = scene::camera(view_yaw(&s), view_pitch(&s), view_pan(&s), view_zoom(&s), setup, w as f32 / h as f32, &look);
        let t = std::time::Instant::now();
        let mut px = tracer.render(&gpu, cam, w, h).0;
        for _ in 1..phiray::Tracer::SAMPLES {
            px = tracer.render(&gpu, cam, w, h).0;
        }
        eprintln!("{name}: {count} tris, {vert} verts, frame {:?}", t.elapsed());
        let sky = [px[0], px[1], px[2]];
        let covered = px.chunks(4).filter(|p| p[..3].iter().zip(&sky).map(|(a, b)| a.abs_diff(*b) as u32).sum::<u32>() > 40).count();
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
    let palette = super::BMeshPalette {
        shading: BShading::Rendered, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey,
    };
    let cube = BObject3D::Primitive { name: "Cube".into(), shape: BShape::Cube, x: 0.0, y: 0.0 };
    worker.send(BRayJob::Scene(1, vec![cube], palette));
    let s = BState::default();
    let look = world::BRayLook { sky: black(), grid: white().opacity(GRID_ALPHA), world: None, lights: Vec::new(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
    let setup = crate::v2::scene::camera::setup_from_state(&s);
    let cam = |yaw: f32| scene::camera(yaw, 30.0, (0.0, 0.0), 1.0, setup, 1.0, &look);
    // An outdated view first, then the one that must win.
    worker.send(BRayJob::View(1, BRayShot { cam: cam(0.0), w: 32, h: 32, mode: phiray::Mode::Rendered, samples: 32, denoise: false }));
    worker.send(BRayJob::View(2, BRayShot { cam: cam(45.0), w: 48, h: 48, mode: phiray::Mode::Rendered, samples: 32, denoise: false }));
    let (mut seq, mut last) = (0, None);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while std::time::Instant::now() < deadline {
        if let Some(f) = worker.take(seq) {
            seq = f.seq;
            let done = f.done && f.view == 2;
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
}

/// Properties → Render → Light Paths → Max Bounces → Diffuse has to reach the
/// `Camera` the tracer runs, or the panel is decoration.
#[test]
fn the_panels_bounce_count_reaches_the_camera() {
    use crate::v2::scene::BRenderSet;
    let look = world::BRayLook {
        sky: black(),
        grid: white().opacity(GRID_ALPHA),
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
