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
    /// Approximate heap memory of the loaded scene, in bytes (Blender's
    /// `MEM_get_memory_in_use` equivalent for the read data).
    pub memory_bytes: u64,
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
        let stats = format!(
            "Verts:{} | Faces:{} | Tris:{} | Objects:{}",
            group(self.verts),
            group(self.faces),
            group(self.tris),
            group(self.objects),
        );
        let mem = self.mem_line();
        if mem.is_empty() { stats } else { format!("{stats} | {mem}") }
    }
    /// Blender's status-bar `Mem:` segment (`BLI_str_format_byte_unit`,
    /// binary base). Returns an empty string when no memory figure was set.
    pub fn mem_line(&self) -> String {
        if self.memory_bytes == 0 {
            return String::new();
        }
        let b = self.memory_bytes as f64;
        if b < 1024.0 * 1024.0 {
            format!("Mem: {:.1} KiB", b / 1024.0)
        } else if b < 1024.0 * 1024.0 * 1024.0 {
            format!("Mem: {:.1} MiB", b / (1024.0 * 1024.0))
        } else {
            format!("Mem: {:.2} GiB", b / (1024.0 * 1024.0 * 1024.0))
        }
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
        // With memory set, stats line includes it.
        let r2 = SceneReadReport { memory_bytes: 52_428_800, ..r.clone() };
        assert_eq!(r2.mem_line(), "Mem: 50.0 MiB");
        assert!(r2.stats_line().ends_with("| Mem: 50.0 MiB"));
    }
}
