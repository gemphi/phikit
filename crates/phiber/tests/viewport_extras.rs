//! A loaded `.blend` gives the viewport what Blender's overlay and Solid
//! mode use: camera / light / empty data and wire shapes, per-face material
//! slots, and a working shading radio in the View3D header.

use phiber::phiphx::Phx;
use phiber::v2::host::BHostType;
use phiber::v2::scene::extra::{BExtraKind, scene_extras};
use phiber::v2::scene::{BShading, shading_key, view_shading};
use phiber::v2::state::{BKey as _, BState};
use phiber::v2::state::action::apply_action;
use phiber::v2::view::{BAction, BView, BViewItem};
use std::path::PathBuf;

fn fixture() -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../phiphx/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
    p.is_file().then_some(p)
}

#[test]
fn cameras_lights_and_empties_come_from_their_dna() {
    let Some(path) = fixture() else { return };
    let m = Phx::open(&path).expect("load");
    let front = m.cameras.iter().find(|c| c.name == "front_cam").expect("front_cam");
    assert_eq!(front.kind, "ORTHO");
    assert!((front.ortho_scale - 3.17).abs() < 1e-3, "{}", front.ortho_scale);
    assert!(m.cameras.iter().any(|c| c.kind == "PERSP"));
    assert!(m.lights.iter().any(|l| l.kind == "SPOT"), "{:?}", m.lights.iter().map(|l| &l.kind).collect::<Vec<_>>());
    assert!(m.empties.iter().any(|e| e.name == "cam_pivit"));
    // Non-mesh objects are no longer mesh rows.
    assert!(m.objects.iter().all(|o| o.name != "front_cam"));

    let lines = scene_extras(&m);
    for kind in [BExtraKind::Camera, BExtraKind::Light, BExtraKind::Empty] {
        assert!(lines.iter().any(|l| l.kind == kind), "no {kind:?} overlay lines");
    }
}

#[test]
fn multi_material_meshes_keep_their_face_slots() {
    let Some(path) = fixture() else { return };
    let m = Phx::open(&path).expect("load");
    let head = m.objects.iter().find(|o| o.name == "head_base_geo").expect("head");
    assert!(head.materials.len() > 1);
    assert_eq!(head.material_indices.len(), head.mesh.indices.len() / 3);
    assert!(head.material_indices.iter().any(|&s| s > 0));
}

fn tabs(v: &BView, out: &mut Vec<BAction>) {
    match v {
        BView::Tab(c) | BView::Button(c) => out.extend(c.action.clone()),
        BView::Row { list } | BView::Column { list } => list.iter().for_each(|k| tabs(k, out)),
        _ => {}
    }
}

#[test]
fn view3d_header_has_the_four_shading_modes() {
    let mut s = BState::new();
    assert_eq!(view_shading(&s), BShading::Solid);
    // The shading radio is a right-hand header control now, so it is built by
    // `part::filter` — the mode bar contributes only the mode picker.
    let mut actions = Vec::new();
    tabs(
        &phiber::v2::part::filter::BFilter::new(BHostType::View3D).build(&s),
        &mut actions,
    );
    // X-ray leads them (`view3d.toggle_xray`), then the four modes.
    assert!(matches!(actions.first(), Some(BAction::Set(k, v)) if k == &phiber::v2::state::OpKey::View3d.path() && v == "toggle_xray"), "{actions:?}");
    assert_eq!(actions.iter().filter(|a| matches!(a, BAction::Set(k, _) if k == &shading_key())).count(), 4);
    let wire = actions
        .iter()
        .find(|a| matches!(a, BAction::Set(k, v) if k == &shading_key() && v == "wireframe"))
        .expect("wireframe button");
    apply_action(&mut s, wire);
    assert_eq!(view_shading(&s), BShading::Wireframe);
}
