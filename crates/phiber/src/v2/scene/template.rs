//! The startup file — what `wm.read_homefile` reads: Blender opens its
//! `startup.blend` (a cube, a light, a camera) as an untitled document, so
//! New File gives a scene with a file behind it to save through.
//!
//! A `.blend`'s layout goes with its version — the header, how a mesh keeps
//! its attributes — so the file New File saves through should be the layout
//! of the Blender it will be opened in. phiber carries one factory startup
//! per major version, each written by that Blender itself
//! (`wm.read_factory_settings` then `wm.save_as_mainfile`; the script is
//! `scratchpad/startup/make_startup.py`): **3.6 LTS**, **4.2 LTS** and
//! **5.0**. Which one New File reads is matched to the open document's
//! version — a 3.6 pipeline stays 3.6 — or named outright
//! (`ctx.op.new_file = general:4`); with neither, the newest.

use std::path::PathBuf;

/// One factory startup, by the Blender major version whose layout it has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BTemplate {
    /// Blender 3.6 LTS (`BLENDER-v306`): also what anything older gets.
    V3,
    /// Blender 4.2 LTS.
    V4,
    /// Blender 5.0, and anything newer.
    V5,
}

impl BTemplate {
    pub const ALL: [Self; 3] = [Self::V3, Self::V4, Self::V5];
    /// The newest: what a document with no version behind it gets.
    pub const NEWEST: Self = Self::V5;

    /// The template for a Blender major version: 3 and below, 4, 5 and up.
    pub fn for_major(major: u32) -> Self {
        match major {
            0..=3 => Self::V3,
            4 => Self::V4,
            _ => Self::V5,
        }
    }

    /// The template for a version label as the reader reports one — "3.6",
    /// "4.2.23", "5.0", or a `BLENDER-v306` header's "306" — by its major.
    pub fn for_version(label: &str) -> Option<Self> {
        let digits: String = label.trim_start_matches(|c: char| !c.is_ascii_digit()).chars().take_while(|c| c.is_ascii_digit()).collect();
        let number: u32 = digits.parse().ok()?;
        // A header's `306` / `402` / `500` is `major × 100 + minor`.
        Some(Self::for_major(if digits.len() >= 3 { number / 100 } else { number }))
    }

    /// The Blender release the template was written by.
    pub fn blender(self) -> &'static str {
        match self {
            Self::V3 => "3.6",
            Self::V4 => "4.2",
            Self::V5 => "5.0",
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Self::V3 => "startup-3.blend",
            Self::V4 => "startup-4.blend",
            Self::V5 => "startup-5.blend",
        }
    }

    /// The file's bytes, zstd-wrapped as Blender writes a compressed save.
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::V3 => include_bytes!("../../../assets/blend/startup-3.blend"),
            Self::V4 => include_bytes!("../../../assets/blend/startup-4.blend"),
            Self::V5 => include_bytes!("../../../assets/blend/startup-5.blend"),
        }
    }

    /// The template `spec` asks for — `general` matches the open document's
    /// version (`open`), `general:4` names a major — and the app template's
    /// own name. With no document and no version named, the newest.
    pub fn pick<'a>(spec: &'a str, open: Option<&str>) -> (&'a str, Self) {
        let (name, version) = spec.split_once(':').map_or((spec, None), |(n, v)| (n, Some(v)));
        let template = version
            .and_then(Self::for_version)
            .or_else(|| open.and_then(Self::for_version))
            .unwrap_or(Self::NEWEST);
        (name, template)
    }

    /// The startup file on disk, written fresh so it is always this build's.
    pub fn startup_path(self) -> std::io::Result<PathBuf> {
        let dir = std::env::temp_dir().join("phiber");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(self.file_name());
        if std::fs::read(&path).map(|have| have != self.bytes()).unwrap_or(true) {
            std::fs::write(&path, self.bytes())?;
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_startup_file_is_blenders_own_factory_scene_in_its_versions_layout() {
        for template in BTemplate::ALL {
            let path = template.startup_path().expect("written");
            let file = crate::phiphx::blend::BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap();
            assert_eq!(BTemplate::for_version(&file.header.version_label()), Some(template), "{}: {}", template.file_name(), file.header.version_label());
            let scene = file.extract_scene().unwrap();
            let mut names: Vec<&str> = scene.objects.iter().map(|o| o.name.as_str()).collect();
            names.sort();
            assert_eq!(names, ["Camera", "Cube", "Light"], "{}", template.file_name());
            assert_eq!((scene.props.frame_current, scene.props.frame_start, scene.props.frame_end), (1, 1, 250));
        }
    }

    #[test]
    fn the_template_is_matched_to_the_open_files_major_or_named_else_the_newest() {
        assert_eq!(BTemplate::for_version("3.6"), Some(BTemplate::V3));
        assert_eq!(BTemplate::for_version("2.93"), Some(BTemplate::V3), "older than 3 gets the oldest carried");
        assert_eq!(BTemplate::for_version("4.5.1"), Some(BTemplate::V4));
        assert_eq!(BTemplate::for_version("v402"), Some(BTemplate::V4), "a header's major×100+minor");
        assert_eq!(BTemplate::for_version("5.2"), Some(BTemplate::V5));
        assert_eq!(BTemplate::for_version("6.0"), Some(BTemplate::V5), "newer than we carry gets the newest");
        assert_eq!(BTemplate::for_version(""), None);
        assert_eq!(BTemplate::pick("general", Some("3.6")), ("general", BTemplate::V3), "matches the open file");
        assert_eq!(BTemplate::pick("general:4", Some("3.6")), ("general", BTemplate::V4), "a named major wins");
        assert_eq!(BTemplate::pick("general", None), ("general", BTemplate::V5), "nothing open: the newest");
        assert_eq!(BTemplate::pick("2d_animation:5", None), ("2d_animation", BTemplate::V5));
    }
}
