//! What Blender prints after a file read (`file_read_reports_finalize`,
//! wm_files.cc) and shows in the status bar (`info_stats.cc`): timings,
//! scene name, file version and scene statistics.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneReadReport {
    /// Active scene name (`FileGlobal.curscene`).
    pub scene: String,
    /// File version as saved, e.g. "3.6", "5.2".
    pub file_version: String,
    /// Whole read duration in milliseconds (0 when not timed).
    pub duration_ms: u64,
    pub objects: usize,
    pub verts: usize,
    /// Polygons as stored in the file (Blender "Faces").
    pub faces: usize,
    pub tris: usize,
    pub cameras: usize,
    pub lights: usize,
    pub empties: usize,
    /// Non-fatal notes (e.g. unsupported data skipped).
    pub warnings: Vec<String>,
}

impl SceneReadReport {
    /// Blender's CLOG line: "Blender file read in 0m0.41s".
    pub fn duration_line(&self) -> String {
        let secs = self.duration_ms as f64 / 1000.0;
        format!("Blender file read in {}m{:.2}s", (secs / 60.0).floor(), secs % 60.0)
    }

    /// Status-bar statistics in `info_stats.cc` format (Object Mode).
    pub fn stats_line(&self) -> String {
        format!(
            "Verts:{} | Faces:{} | Tris:{} | Objects:{}",
            group(self.verts),
            group(self.faces),
            group(self.tris),
            group(self.objects)
        )
    }
}

/// `BLI_str_format_int_grouped`: 1234567 → "1,234,567".
fn group(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_blender() {
        let r = SceneReadReport { duration_ms: 1410, verts: 754928, faces: 12, tris: 20, objects: 179, ..Default::default() };
        assert_eq!(r.duration_line(), "Blender file read in 0m1.41s");
        assert_eq!(r.stats_line(), "Verts:754,928 | Faces:12 | Tris:20 | Objects:179");
    }
}
