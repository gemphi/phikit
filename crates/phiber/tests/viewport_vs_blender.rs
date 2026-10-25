//! The viewport must show what Blender shows. For each file, opening it
//! (reader → modifier stack → Load UI → camera) and filling every triangle
//! must cover the same pixels as Blender's own render of that saved view
//! (`phiphx/tests/truth/views/*.png`, made by `blender_view_mask.py`).
//! Files missing on this machine are skipped.

use phiber::phiphx::Phx;
use phiber::v2::app::load_ui::restore_ui;
use phiber::v2::scene::camera::{note_aspect, setup_from_state, with_view};
use phiber::v2::scene::{MESH_SCALE, project3_full, view_pan, view_pitch, view_yaw, view_zoom};
use phiber::v2::state::BState;
use std::path::PathBuf;

const W: u32 = 480;
const H: u32 = 270;

fn our_mask(path: &PathBuf) -> Vec<bool> {
    let model = Phx::open(path).expect("loads");
    let mut s = BState::default();
    restore_ui(&mut s, &model);
    note_aspect(W as f32 / H as f32);
    let (yaw, pitch, pan, zoom) = (view_yaw(&s), view_pitch(&s), view_pan(&s), view_zoom(&s));
    let mut mask = vec![false; (W * H) as usize];
    with_view(setup_from_state(&s), || {
        for o in &model.objects {
            let m = glam::Mat4::from_cols_array_2d(&o.matrix_world);
            let p: Vec<(f32, f32)> = o.mesh.vertices.iter().map(|v| {
                let q = m.transform_point3(glam::Vec3::from(*v)) * MESH_SCALE;
                let (x, y, _) = project3_full(q.x, q.z, q.y, yaw, pitch, pan, zoom);
                ((0.5 + x) * W as f32, (0.5 + y) * H as f32)
            }).collect();
            for t in o.mesh.indices.chunks_exact(3) {
                fill(&mut mask, [p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]]);
            }
        }
    });
    mask
}

fn fill(mask: &mut [bool], [a, b, c]: [(f32, f32); 3]) {
    let area = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
    if area.abs() < 1e-9 {
        return;
    }
    let (x0, x1) = (a.0.min(b.0).min(c.0).max(0.0) as u32, a.0.max(b.0).max(c.0).min(W as f32 - 1.0).max(0.0) as u32);
    let (y0, y1) = (a.1.min(b.1).min(c.1).max(0.0) as u32, a.1.max(b.1).max(c.1).min(H as f32 - 1.0).max(0.0) as u32);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = ((b.0 - px) * (c.1 - py) - (b.1 - py) * (c.0 - px)) / area;
            let w1 = ((c.0 - px) * (a.1 - py) - (c.1 - py) * (a.0 - px)) / area;
            if w0 >= 0.0 && w1 >= 0.0 && w0 + w1 <= 1.0 {
                mask[(y * W + x) as usize] = true;
            }
        }
    }
}

#[test]
fn saved_views_match_blender_renders() {
    let phiphx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx");
    let cases = [
        ("tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend", "THYMUS-BOLDROID-WAVE-003.png"),
        ("tests/fixtures/FLORIS_BOLDRIOD_RIG.blend", "FLORIS_BOLDRIOD_RIG.png"),
        ("../../../ext/REFS/blender/blender/tests/files/modeling/operators.blend", "operators.png"),
    ];
    let mut checked = 0;
    for (file, truth) in cases {
        let path = phiphx.join(file);
        if !path.exists() {
            continue;
        }
        let want = image::open(phiphx.join("tests/truth/views").join(truth)).unwrap().to_luma8();
        let got = our_mask(&path);
        let (mut both, mut any) = (0usize, 0usize);
        for (i, px) in want.pixels().enumerate() {
            let (b, o) = (px.0[0] > 0, got[i]);
            both += (b && o) as usize;
            any += (b || o) as usize;
        }
        let iou = both as f32 / any.max(1) as f32;
        assert!(iou >= 0.99, "{file}: silhouette IoU {iou:.3} vs Blender");
        checked += 1;
    }
    assert!(checked > 0, "no files found");
}
