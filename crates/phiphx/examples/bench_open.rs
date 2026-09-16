//! Time a `.blend` open exactly the way Phiber opens it (no sidecar, real
//! progress callbacks). Usage: `cargo run -p phiphx --example bench_open -- file.blend`
use std::time::Instant;

fn main() {
    let path = std::env::args().nth(1).expect("path to .blend");
    let t0 = Instant::now();
    let model = phiphx::services::scene::load_blend_scene_with_progress(
        std::path::Path::new(&path),
        &|fraction, stage| {
            eprintln!("  {:5.1}% {stage}", fraction * 100.0);
            true
        },
    )
    .expect("load");
    let verts: usize = model.objects.iter().map(|o| o.mesh.vertices.len()).sum();
    println!(
        "{path}: {} objects, {verts} verts, {} cameras, {} lights in {:.2?}",
        model.objects.len(),
        model.cameras.len(),
        model.lights.len(),
        t0.elapsed()
    );
}
