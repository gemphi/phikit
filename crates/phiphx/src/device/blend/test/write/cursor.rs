use crate::device::blend::file::kind::BlendFile;
use glam::{Mat4, Quat, Vec3};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");

#[test]
fn a_cursor_written_reads_back_where_it_was_put_in_the_file_s_rotation_mode() {
    // `fixtures/overlays.blend` keeps its cursor in ZXY Euler.
    let bytes = std::fs::read(format!("{DIR}/fixtures/overlays.blend")).unwrap();
    let mut file = BlendFile::parse_bytes(&bytes).unwrap();
    let want = Mat4::from_rotation_translation(Quat::from_euler(glam::EulerRot::XYZ, 0.4, -0.7, 1.9), Vec3::new(-3.0, 2.5, 0.75));
    let edits = crate::device::blend::write::BlendEditList { cursor: Some(want.to_cols_array_2d()), ..Default::default() };
    let report = edits.apply(&mut file);
    assert!(report.patched >= 6, "{report:?}");
    let out = std::env::temp_dir().join("phiphx_cursor_write.blend");
    file.write_to(&out, crate::device::blend::write::BlendCompression::None).unwrap();
    let scene = BlendFile::parse_bytes(&std::fs::read(&out).unwrap()).unwrap().extract_scene().unwrap();
    let got: Vec<f32> = scene.props.cursor.matrix.iter().flatten().copied().collect();
    let want: Vec<f32> = want.to_cols_array().to_vec();
    assert!(got.iter().zip(&want).all(|(a, b)| (a - b).abs() < 1e-5), "{got:?} against {want:?}");
}
