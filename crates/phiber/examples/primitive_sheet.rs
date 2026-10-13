//! Every base primitive, rendered side by side — a rainbow row in a rough
//! dielectric and a second row in polished silver — so the meshes
//! themselves can be *looked* at rather than counted.
//!
//!   cargo run --release -p phiber --example primitive_sheet -- OUT.png [samples]

use phiphx::spatial::{PhiQuiMeshType, SpatialObject3D};
use phiray::{Camera, LIGHTS, Light, Mat, Mode, Ray, Scene, Tracer, Tri};

const W: u32 = 1200;
const H: u32 = 580;

/// The primitives, left to right.
const SHOWN: [(PhiQuiMeshType, &str); 12] = [
    (PhiQuiMeshType::Plane, "Plane"),
    (PhiQuiMeshType::Cube, "Cube"),
    (PhiQuiMeshType::Circle, "Circle"),
    (PhiQuiMeshType::Sphere, "UV Sphere"),
    (PhiQuiMeshType::IcoSphere, "Ico Sphere"),
    (PhiQuiMeshType::Cylinder, "Cylinder"),
    (PhiQuiMeshType::Cone, "Cone"),
    (PhiQuiMeshType::Torus, "Torus"),
    (PhiQuiMeshType::Icosahedron, "Gem"),
    (PhiQuiMeshType::Metaball, "Metaball"),
    (PhiQuiMeshType::Grid, "Grid"),
    (PhiQuiMeshType::Text, "Text"),
];

/// A rainbow across the row: hue from `t`, full saturation, scene-linear.
fn rainbow(t: f32) -> [f32; 4] {
    let h = t * 5.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    // sRGB-ish to linear, so the render is lit rather than painted.
    [r.powf(2.2).max(0.02), g.powf(2.2).max(0.02), b.powf(2.2).max(0.02), 1.0]
}

/// One primitive placed at `(x, z)`, scaled to `s`.
fn place(scene: &mut Scene, kind: PhiQuiMeshType, at: [f32; 3], s: f32, color: [f32; 4], rough: f32, metal: f32) {
    let (v, idx, n) = SpatialObject3D::buffers(kind);
    let mut world = phiray::trace::instance::IDENTITY;
    for k in 0..3 {
        world[k][k] = s;
    }
    world[3] = [at[0], at[1], at[2], 1.0];
    scene.instance(world);
    let p = |i: u32| v.get(i as usize).map(|v| [v.0, v.1, v.2]).unwrap_or_default();
    let nn = |i: u32| n.get(i as usize).map(|n| [n.0, n.1, n.2]).unwrap_or_default();
    // ONE material for the whole primitive. Asking for one per triangle
    // filled the material table — the metaball alone is 13,668 of them —
    // and every face past the table read a material that is not there,
    // which draws as a black card floating where the mesh should be. That
    // is what the first sheet showed for the gem and the metaball.
    let shade = Mat::of(&Tri::new([[0.0; 3]; 3], [[0.0, 1.0, 0.0]; 3], color).with_pbr(rough, metal));
    let m = scene.material(shade);
    // Vertices are shared by index, as the buffers give them, so the
    // smooth normals stay smooth.
    let base: Vec<u32> = (0..v.len() as u32).map(|k| scene.vertex(p(k), nn(k), [0.0; 2])).collect();
    for t in idx.chunks_exact(3) {
        scene.face([base[t[0] as usize], base[t[1] as usize], base[t[2] as usize]], m);
    }
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "primitives.png".into());
    let samples: u32 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(64);
    let Some(gpu) = Ray::gpu() else { return eprintln!("no GPU adapter") };

    let mut scene = Scene::default();
    let n = SHOWN.len() as f32;
    let spread = 2.9f32;
    for (i, (kind, _)) in SHOWN.iter().enumerate() {
        let x = (i as f32 - (n - 1.0) / 2.0) * spread;
        let t = i as f32 / (n - 1.0);
        // Top row: the rainbow, a rough dielectric — the shape reads.
        // Text is one em tall where the others are two units across.
        let s = if *kind == PhiQuiMeshType::Text { 1.9 } else { 1.0 };
        place(&mut scene, *kind, [x, 0.0, 2.4], s, rainbow(t), 0.42, 0.0);
        // Behind and above: polished silver, which is where a bad normal
        // shows at once. Lifted clear of the front row rather than tucked
        // behind it — half a sphere says nothing about its shading.
        place(&mut scene, *kind, [x, 2.45, -2.2], s, [0.93, 0.93, 0.95, 1.0], 0.08, 1.0);
    }
    // A floor, so the silver has something to reflect.
    let f = 200.0f32;
    scene.instance(phiray::trace::instance::IDENTITY);
    let floor = Tri::new([[-f, -1.2, -f], [f, -1.2, -f], [f, -1.2, f]], [[0.0, 1.0, 0.0]; 3], [0.22, 0.23, 0.26, 1.0]).with_pbr(0.6, 0.0);
    let m = scene.material(Mat::of(&floor));
    let c = [[-f, -1.2, -f], [f, -1.2, -f], [f, -1.2, f], [-f, -1.2, f]].map(|q| scene.vertex(q, [0.0, 1.0, 0.0], [0.0; 2]));
    scene.face([c[0], c[1], c[2]], m);
    scene.face([c[0], c[2], c[3]], m);

    let mut tracer = Tracer::from_scene(&gpu, scene).expect("scene fits");
    tracer.set_mode(Mode::Rendered);
    tracer.set_samples(samples);

    // Looking slightly down the −Z axis, orthographic, so every primitive
    // is drawn at the same size and nothing is foreshortened.
    let pitch = 10f32.to_radians();
    let (sp, cp) = pitch.sin_cos();
    let mut lights = [Light::default(); LIGHTS];
    lights[0] = Light { pos: [6.0, 9.0, 8.0], kind: 1, color: [900.0, 880.0, 840.0], size: 2.0, ..Default::default() };
    lights[1] = Light { pos: [-9.0, 5.0, 4.0], kind: 1, color: [300.0, 330.0, 420.0], size: 3.0, ..Default::default() };
    let cam = Camera {
        right: [1.0, 0.0, 0.0],
        up: [0.0, cp, sp],
        back: [0.0, sp, cp],
        units: 1.0,
        scale: 1.0 / (spread * n + 2.2),
        aspect: W as f32 / H as f32,
        persp: 0,
        dist: 40.0,
        pivot: [0.0, 1.0, 0.0],
        sky: [0.30, 0.32, 0.37, 0.0],
        env: [0.16, 0.18, 0.24, 1.0],
        count: 2,
        lights,
        bounces: 3,
        ..Default::default()
    };
    let at = std::time::Instant::now();
    while !tracer.sample(&gpu, cam, W, H).1 {}
    tracer.denoise(&gpu);
    let px = tracer.pixels(&gpu);
    image::RgbaImage::from_raw(W, H, px).expect("size").save(&out).expect("write");
    println!("{out}: {}x{} at {samples} samples in {:.1}s", W, H, at.elapsed().as_secs_f64());
}
