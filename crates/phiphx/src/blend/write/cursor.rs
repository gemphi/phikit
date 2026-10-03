//! Writing the 3D cursor back (`scene/cursor.rs` reads it): its place, and
//! its turn in the rotation mode the file already keeps it in — a
//! quaternion, an axis and angle, or Euler angles in any of the six orders
//! (`View3DCursor::set_matrix`), so the N panel shows the cursor as the user
//! left it set up.

use crate::blend::file::types::{BlendFile, DnaType};
use glam::{EulerRot, Mat4};

impl BlendFile {
    /// Write `matrix` as scene `scene`'s cursor: how many fields changed.
    pub(crate) fn write_cursor(&mut self, scene: usize, dna: &DnaType, matrix: [[f32; 4]; 4]) -> usize {
        let (_, rotation, location) = Mat4::from_cols_array_2d(&matrix).to_scale_rotation_translation();
        let Some(base) = self.off("Scene", "cursor", dna) else { return 0 };
        let mode = self.off("View3DCursor", "rotation_mode", dna).map_or(1, |o| self.i16_at(&self.blocks[scene].data, Some(base + o), 1));
        let mut floats: Vec<(String, f32)> = (0..3).map(|i| (format!("cursor.location[{i}]"), location[i])).collect();
        match mode {
            0 => {
                let q = [rotation.w, rotation.x, rotation.y, rotation.z];
                floats.extend((0..4).map(|i| (format!("cursor.rotation_quaternion[{i}]"), q[i])));
            }
            -1 => {
                let (axis, angle) = rotation.to_axis_angle();
                floats.extend((0..3).map(|i| (format!("cursor.rotation_axis[{i}]"), axis[i])));
                floats.push(("cursor.rotation_angle".to_string(), angle));
            }
            _ => {
                // `ROT_MODE_XYZ` … `ZYX`: the first axis turns first, so the
                // matrix is the reverse product — glam's order reversed.
                let (order, axes) = match mode {
                    2 => (EulerRot::YZX, [1, 2, 0]),
                    3 => (EulerRot::ZXY, [2, 0, 1]),
                    4 => (EulerRot::XZY, [0, 2, 1]),
                    5 => (EulerRot::YXZ, [1, 0, 2]),
                    6 => (EulerRot::XYZ, [0, 1, 2]),
                    _ => (EulerRot::ZYX, [2, 1, 0]),
                };
                let (a, b, c) = rotation.to_euler(order);
                let mut euler = [0.0; 3];
                for (axis, angle) in axes.iter().zip([a, b, c]) {
                    euler[*axis] = angle;
                }
                floats.extend((0..3).map(|i| (format!("cursor.rotation_euler[{i}]"), euler[i])));
            }
        }
        floats.iter().map(|(field, value)| usize::from(self.set_f32(scene, "Scene", field, *value))).sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::blend::file::types::BlendFile;
    use glam::{Mat4, Quat, Vec3};

    const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");

    #[test]
    fn a_cursor_written_reads_back_where_it_was_put_in_the_file_s_rotation_mode() {
        // `fixtures/overlays.blend` keeps its cursor in ZXY Euler.
        let bytes = std::fs::read(format!("{DIR}/fixtures/overlays.blend")).unwrap();
        let mut file = BlendFile::parse_bytes(&bytes).unwrap();
        let want = Mat4::from_rotation_translation(Quat::from_euler(glam::EulerRot::XYZ, 0.4, -0.7, 1.9), Vec3::new(-3.0, 2.5, 0.75));
        let edits = crate::blend::write::BlendEdits { cursor: Some(want.to_cols_array_2d()), ..Default::default() };
        let report = edits.apply(&mut file);
        assert!(report.patched >= 6, "{report:?}");
        let out = std::env::temp_dir().join("phiphx_cursor_write.blend");
        file.write_to(&out, crate::blend::write::BlendCompression::None).unwrap();
        let scene = BlendFile::parse_bytes(&std::fs::read(&out).unwrap()).unwrap().extract_scene().unwrap();
        let got: Vec<f32> = scene.props.cursor.matrix.iter().flatten().copied().collect();
        let want: Vec<f32> = want.to_cols_array().to_vec();
        assert!(got.iter().zip(&want).all(|(a, b)| (a - b).abs() < 1e-5), "{got:?} against {want:?}");
    }
}

