//! A batch skinner must give the same scene as the built-in Armature path.

use phiphx::phx::{CpuSkin, Phx, SceneModel, SkinJob, Skinner};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Counting(AtomicUsize);

impl Skinner for Counting {
    fn run(&self, jobs: &mut [SkinJob]) -> bool {
        self.0.fetch_add(jobs.len(), Ordering::SeqCst);
        CpuSkin.run(jobs)
    }
}

fn verts(m: &SceneModel) -> Vec<[f32; 3]> {
    m.objects.iter().flat_map(|o| o.mesh.vertices.iter().copied()).collect()
}

#[test]
fn batched_skin_matches_builtin() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for name in ["FLORIS_BOLDRIOD_RIG.blend", "THYMUS-BOLDROID-WAVE-003.blend"] {
        let path = root.join(name);
        Phx::use_skinner(None);
        let base = Phx::open(&path).expect("open");
        let counter = Arc::new(Counting(AtomicUsize::new(0)));
        Phx::use_skinner(Some(counter.clone()));
        let batched = Phx::open(&path).expect("open");
        Phx::use_skinner(None);
        assert!(counter.0.load(Ordering::SeqCst) > 0, "{name}: no skin jobs");
        let (a, b) = (verts(&base), verts(&batched));
        assert_eq!(a.len(), b.len());
        let worst = a.iter().zip(&b).map(|(p, q)| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0, f32::max)).fold(0.0, f32::max);
        assert!(worst < 1e-3, "{name}: max diff {worst}");
    }
}
