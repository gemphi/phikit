//! Compare evaluated vertices against Blender's, object by object.
//!
//! `phiphx/tests/truth/blender_vertex_dump.py` writes the reference; this
//! reads it and prints, per object, the mean and worst world-space distance.
//! Sub-millimetre deviations are what make one mesh poke through another.
//!
//!   cargo run --release -p phiber --example vertex_delta -- FILE TRUTH.bin NAME...
use phiber::phiphx::Phx;
use phiber::v2::app::seed::seed_view_layer;
use phiber::v2::scene::{BObject3D, current_scene, source::set_loaded_scene};
use phiber::v2::state::BState;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("file");
    let truth = args.next().expect("truth.bin");
    let want: Vec<String> = args.collect();
    let bytes = std::fs::read(&truth).expect("truth");
    let model = Phx::open(std::path::Path::new(&path)).expect("open");
    let mut s = BState::new();
    seed_view_layer(&mut s, &model);
    set_loaded_scene(Some(model.clone()));
    let scene = current_scene(&s, Some(&model));

    let mut at = 0usize;
    for name in &want {
        let count = u32::from_le_bytes(bytes[at..at + 4].try_into().expect("count")) as usize;
        at += 4;
        let truth: Vec<[f32; 3]> = (0..count)
            .map(|i| {
                let o = at + i * 12;
                [0, 1, 2].map(|k| f32::from_le_bytes(bytes[o + k * 4..o + k * 4 + 4].try_into().expect("f32")))
            })
            .collect();
        at += count * 12;
        let Some(BObject3D::Loaded { mesh, matrix_world, .. }) =
            scene.iter().find(|o| matches!(o, BObject3D::Loaded { name: n, .. } if n == name))
        else {
            println!("{name:40} missing");
            continue;
        };
        if mesh.vertices.len() != count {
            println!("{name:40} count {} vs Blender {count}", mesh.vertices.len());
            continue;
        }
        let m = glam::Mat4::from_cols_array_2d(matrix_world);
        let mut sum = 0.0f64;
        let mut worst = 0.0f32;
        let mut over = 0usize;
        for (v, t) in mesh.vertices.iter().zip(&truth) {
            let d = (m.transform_point3(glam::Vec3::from(*v)) - glam::Vec3::from(*t)).length();
            sum += d as f64;
            worst = worst.max(d);
            if d > 0.0005 {
                over += 1;
            }
        }
        println!(
            "{name:40} v={count:7} mean={:.4} mm worst={:.4} mm over-0.5mm={over}",
            sum / count as f64 * 1000.0,
            worst * 1000.0
        );
        // PHIBER_DUMP=<dir> also writes our own vertices, so a point-set
        // comparison can be made when the two orders do not line up (Blender's
        // Solidify emits shell and rim vertices in its own order).
        if let Ok(dir) = std::env::var("PHIBER_DUMP") {
            let mut out = Vec::with_capacity(count * 12);
            for v in &mesh.vertices {
                for c in m.transform_point3(glam::Vec3::from(*v)).to_array() {
                    out.extend(c.to_le_bytes());
                }
            }
            let file = std::path::Path::new(&dir).join(format!("{}.bin", name.replace(['/', ' '], "_")));
            std::fs::write(&file, out).expect("dump");
        }
    }
}
