//! THYMUS waves in the viewport: the object list at the shown frame carries
//! each rigged mesh posed by its rig there (phiphx `scene/rig.rs`) — the
//! same triangles every frame, other vertices — while the meshes no rig
//! moves stay the ones read from the file.

use phiber::phiphx::phx::Phx;
use phiber::v2::scene::{BObject3D, current_scene};
use phiber::v2::state::{BState, CtxKey};
use std::path::PathBuf;

fn at_frame(frame: &str) -> BState {
    let mut state = BState::new();
    state.set(CtxKey::Frame, frame);
    state
}

fn mesh_of<'a>(objects: &'a [BObject3D], name: &str) -> &'a std::sync::Arc<phiber::phiphx::scene::SceneMesh> {
    objects
        .iter()
        .find_map(|o| match o {
            BObject3D::Loaded { name: n, mesh, .. } if n == name => Some(mesh),
            _ => None,
        })
        .expect("the object")
}

#[test]
fn the_rig_bends_the_character_at_the_shown_frame() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
    let model = Phx::open(&path).expect("THYMUS opens");
    let (one, sixty_six) = (current_scene(&at_frame("1"), Some(&model)), current_scene(&at_frame("66"), Some(&model)));
    let (a, b) = (mesh_of(&one, "arm_lower_geo"), mesh_of(&sixty_six, "arm_lower_geo"));
    assert_eq!((a.indices.len(), a.vertices.len()), (b.indices.len(), b.vertices.len()), "the same mesh");
    assert!(a.indices == b.indices, "the same triangles");
    let moved = a.vertices.iter().zip(&b.vertices).filter(|(p, q)| (0..3).any(|i| (p[i] - q[i]).abs() > 0.05)).count();
    assert!(moved > 1000, "the waving arm moves ({moved} vertices)");
    // At the frame the file was saved on, the pose is the saved one.
    let read = &model.objects.iter().find(|o| o.name == "arm_lower_geo").expect("read").mesh;
    let off = read.vertices.iter().zip(&a.vertices).map(|(p, q)| (0..3).map(|i| (p[i] - q[i]).abs()).fold(0.0, f32::max)).fold(0.0, f32::max);
    assert!(off < 1e-4, "frame 1 draws the saved pose ({off})");
    // A mesh no rig deforms is shared, not rebuilt.
    let skinned: Vec<usize> = model.motion.rigs.skins.iter().map(|s| s.object).collect();
    if let Some(i) = (0..model.objects.len()).find(|i| !skinned.contains(i) && !model.objects[*i].mesh.vertices.is_empty()) {
        let name = &model.objects[i].name;
        assert!(std::sync::Arc::ptr_eq(mesh_of(&one, name), &model.objects[i].mesh), "{name} is the mesh as read");
    }
}
