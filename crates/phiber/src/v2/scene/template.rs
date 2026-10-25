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

    /// Where [`crate::v2::app::BPhiberApp::start_import`] finds this
    /// template: the startup file on disk, or — in a browser, which has no
    /// disk — the `embedded:` scheme the importer reads straight from the
    /// binary ([`embedded_bytes`]).
    pub fn source(self) -> std::io::Result<String> {
        #[cfg(target_family = "wasm")]
        return Ok(format!("embedded:{}", self.file_name()));
        #[cfg(not(target_family = "wasm"))]
        self.startup_path().map(|p| p.to_string_lossy().to_string())
    }

    /// The startup file on disk, written fresh so it is always this build's.
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
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

/// The demo the browser build boots into (`app::boot`): two teams of
/// three rigged players playing a possession each way on an LA28 court —
/// moves as NLA strips, pose and object keys, armature deform and a
/// scoreboard on constant keys in one small file
/// (`assets/source/make_demo.py`).
pub const DEMO: &str = "demo-court.blend";

/// A `.blend` carried inside the binary, by the name an `embedded:` path
/// gives (`app/load.rs`): the three startup templates and the demo scene.
pub fn embedded_bytes(name: &str) -> Option<&'static [u8]> {
    if name == DEMO {
        return Some(include_bytes!("../../../assets/blend/demo-court.blend"));
    }
    BTemplate::ALL.iter().find(|t| t.file_name() == name).map(|t| t.bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The demo the browser boots into opens, is animated, and plays the
    /// kinds of motion it exists to show: the game's moves as NLA strips on
    /// each player's rig (pose keys through strips, armature deform), the
    /// ball's flights as strips of object keys, its glow and the
    /// scoreboard's digits switching on constant keys.
    #[test]
    fn the_embedded_demo_opens_with_its_animation() {
        let bytes = embedded_bytes(DEMO).expect("embedded");
        let model = crate::phiphx::Phx::open_bytes(DEMO, bytes).expect("opens");
        assert_eq!((model.timeline.frame_start, model.timeline.frame_end), (1, 360));
        assert!(model.objects.iter().any(|o| o.name == "Basketball"));
        assert_eq!(model.objects.iter().filter(|o| o.name.ends_with("Body")).count(), 6, "two teams of three");
        // The packed images decode: the court, the ball and every jersey
        // carry a texture, with no file beside the blend to find.
        let textured = |name: &str| {
            model.objects.iter().find(|o| o.name == name).is_some_and(|o| o.materials.iter().any(|m| m.texture.is_some()))
        };
        assert!(textured("Court") && textured("Basketball") && textured("LED Board.N"), "packed textures decode");
        // Every player wears a kit and carries a number.
        for who in ["Ayanda", "Buhle", "Chuma", "Dumi", "Lerato", "Sipho"] {
            for part in ["Jersey", "Shorts", "Numbers"] {
                assert!(model.objects.iter().any(|o| o.name == format!("{who} {part}")), "{who} {part}");
            }
        }
        assert!(model.motion.rigs.animated(), "the players' rigs are animated");
        // Every player's moves are strips on one track, their fingers on
        // another; ten kinds of move between them, and the ball's ten clips
        // on its own.
        let strips = |object: &str, track: &str| -> Vec<String> {
            let stack = model.nla.iter().find(|n| n.object == object).unwrap_or_else(|| panic!("{object} has an NLA"));
            stack.tracks.iter().filter(|t| t.name == track).flat_map(|t| &t.strips).filter_map(|s| s.action.clone()).collect()
        };
        let mut kinds = std::collections::BTreeSet::new();
        for who in ["Ayanda", "Buhle", "Chuma", "Dumi", "Lerato", "Sipho"] {
            let moves = strips(&format!("{who} Rig"), "Moves");
            assert!(!moves.is_empty(), "{who} moves");
            assert_eq!(strips(&format!("{who} Rig"), "Hands"), [format!("{who} Hands")], "the fingers on a track above");
            kinds.extend(moves.iter().map(|m| m.trim_start_matches(who).trim().trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ').to_string()));
        }
        assert_eq!(kinds.len(), 10, "{kinds:?}");
        assert_eq!(strips("Basketball", "Play").len(), 10, "the ball's clips");
        for team in ["Home", "Away"] {
            for points in [0, 2] {
                assert!(model.objects.iter().any(|o| o.name == format!("{team} Points {points}")), "{team} {points}");
            }
        }
        // The ball glows in flight: its lit child steps out of it from the
        // release (Ayanda's pass, frame 57) to the catch, and back inside.
        let glow = model.objects.iter().position(|o| o.name == "Basketball Glow").expect("the ball's glow");
        let size = |f: f32| {
            let m = model.motion.at(f).object(glow).expect("placed");
            glam::Vec3::from_slice(&m[0][..3]).length()
        };
        assert!(size(50.0) < 1.0 && size(60.0) > 1.0 && size(70.0) < 1.0, "{} {} {}", size(50.0), size(60.0), size(70.0));
        // A player's mesh deforms between two frames.
        let (a, _) = model.motion.posed_at(4.0);
        let (b, _) = model.motion.posed_at(10.0);
        assert_eq!(a.len(), b.len());
        assert!(!a.is_empty(), "the rigs deform meshes");
        let moved = a.iter().zip(&b).any(|((_, ma), (_, mb))| {
            ma.vertices.iter().zip(&mb.vertices).any(|(x, y)| (x[2] - y[2]).abs() > 1e-3)
        });
        assert!(moved, "a player's vertices move between frames");
    }

    /// Every embedded name resolves, and an unknown one does not.
    #[test]
    fn embedded_bytes_knows_the_templates_and_the_demo() {
        for t in BTemplate::ALL {
            assert!(embedded_bytes(t.file_name()).is_some(), "{}", t.file_name());
        }
        assert!(embedded_bytes(DEMO).is_some());
        assert!(embedded_bytes("nope.blend").is_none());
    }

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
