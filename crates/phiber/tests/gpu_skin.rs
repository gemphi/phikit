//! phiray skinning gives the same opened scene as phiphx's CPU Armature path.

use phiber::phiphx::phx::{Phx, SceneModel};
use phiber::v2::scene::RaySkin;
use std::path::PathBuf;
use std::sync::Arc;

fn verts(m: &SceneModel) -> Vec<[f32; 3]> {
    m.objects.iter().flat_map(|o| o.mesh.vertices.iter().copied()).collect()
}

#[test]
fn gpu_skin_matches_cpu() {
    let skin = Arc::new(RaySkin::eager());
    if !skin.ready() {
        eprintln!("no GPU adapter; skipped");
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures");
    for name in ["FLORIS_BOLDRIOD_RIG.blend", "THYMUS-BOLDROID-WAVE-003.blend"] {
        let path = root.join(name);
        Phx::use_skinner(None);
        let cpu = Phx::open(&path).expect("open");
        Phx::use_skinner(Some(skin.clone()));
        let t = std::time::Instant::now();
        let gpu = Phx::open(&path).expect("open");
        eprintln!("{name}: GPU-skinned open {:?}", t.elapsed());
        Phx::use_skinner(None);
        let (a, b) = (verts(&cpu), verts(&gpu));
        assert_eq!(a.len(), b.len());
        let worst = a.iter().zip(&b).map(|(p, q)| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0, f32::max)).fold(0.0, f32::max);
        assert!(worst < 1e-3, "{name}: max diff {worst}");
    }
}
