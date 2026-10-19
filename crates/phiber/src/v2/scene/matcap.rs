//! Blender's matcaps for Solid's Lighting ▸ MatCap
//! (`datafiles/studiolights/matcap/*.exr`, CC0 — `assets/matcap/LICENSE.txt`):
//! each a diffuse image and, for most, a specular one (the EXR's `diffuse`
//! and `specular` layers, `studiolight_load_matcap`). Blender's copies are
//! DWA-compressed, which the `exr` decoder does not read, so they ship here
//! decoded once to 256² sRGB PNGs at half scale (a value of 2 is 255) and
//! come back linear (`Matcap::decode`). A user's own matcaps — the files in
//! Blender's user datafiles, `studiolights/matcap/` (`BKE_appdir_folder_id
//! (BLENDER_USER_DATAFILES)`, `STUDIOLIGHT_USER_DATAFILES`) — are listed
//! after the shipped set and read when named, as Blender lists them.
//! Workbench samples them by the view normal (`matcap_uv_compute`,
//! `solid.wgsl`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

/// Name, diffuse PNG, specular PNG. `View3DShading.matcap` names one.
const FILE_LIST: &[(&str, &[u8], Option<&[u8]>)] = &[
    ("basic_bright", include_bytes!("../../../assets/matcap/basic_bright.png"), Some(include_bytes!("../../../assets/matcap/basic_bright.specular.png"))),
    ("basic_dark", include_bytes!("../../../assets/matcap/basic_dark.png"), Some(include_bytes!("../../../assets/matcap/basic_dark.specular.png"))),
    ("basic_grey", include_bytes!("../../../assets/matcap/basic_grey.png"), Some(include_bytes!("../../../assets/matcap/basic_grey.specular.png"))),
    ("basic_side", include_bytes!("../../../assets/matcap/basic_side.png"), Some(include_bytes!("../../../assets/matcap/basic_side.specular.png"))),
    ("ceramic_dark", include_bytes!("../../../assets/matcap/ceramic_dark.png"), Some(include_bytes!("../../../assets/matcap/ceramic_dark.specular.png"))),
    ("ceramic_lightbulb", include_bytes!("../../../assets/matcap/ceramic_lightbulb.png"), Some(include_bytes!("../../../assets/matcap/ceramic_lightbulb.specular.png"))),
    ("check_gradient", include_bytes!("../../../assets/matcap/check_gradient.png"), Some(include_bytes!("../../../assets/matcap/check_gradient.specular.png"))),
    ("check_normal+y", include_bytes!("../../../assets/matcap/check_normal+y.png"), None),
    ("check_reflection_horizontal", include_bytes!("../../../assets/matcap/check_reflection_horizontal.png"), Some(include_bytes!("../../../assets/matcap/check_reflection_horizontal.specular.png"))),
    ("check_reflection_vertical", include_bytes!("../../../assets/matcap/check_reflection_vertical.png"), Some(include_bytes!("../../../assets/matcap/check_reflection_vertical.specular.png"))),
    ("check_rim_dark", include_bytes!("../../../assets/matcap/check_rim_dark.png"), Some(include_bytes!("../../../assets/matcap/check_rim_dark.specular.png"))),
    ("check_rim_light", include_bytes!("../../../assets/matcap/check_rim_light.png"), Some(include_bytes!("../../../assets/matcap/check_rim_light.specular.png"))),
    ("clay_brown", include_bytes!("../../../assets/matcap/clay_brown.png"), Some(include_bytes!("../../../assets/matcap/clay_brown.specular.png"))),
    ("clay_green", include_bytes!("../../../assets/matcap/clay_green.png"), Some(include_bytes!("../../../assets/matcap/clay_green.specular.png"))),
    ("clay_studio", include_bytes!("../../../assets/matcap/clay_studio.png"), Some(include_bytes!("../../../assets/matcap/clay_studio.specular.png"))),
    ("clay_warm", include_bytes!("../../../assets/matcap/clay_warm.png"), Some(include_bytes!("../../../assets/matcap/clay_warm.specular.png"))),
    ("fullmetal", include_bytes!("../../../assets/matcap/fullmetal.png"), Some(include_bytes!("../../../assets/matcap/fullmetal.specular.png"))),
    ("hard_surface_grey", include_bytes!("../../../assets/matcap/hard_surface_grey.png"), Some(include_bytes!("../../../assets/matcap/hard_surface_grey.specular.png"))),
    ("hard_surface_red", include_bytes!("../../../assets/matcap/hard_surface_red.png"), Some(include_bytes!("../../../assets/matcap/hard_surface_red.specular.png"))),
    ("metal_bronze", include_bytes!("../../../assets/matcap/metal_bronze.png"), Some(include_bytes!("../../../assets/matcap/metal_bronze.specular.png"))),
    ("metal_carpaint", include_bytes!("../../../assets/matcap/metal_carpaint.png"), Some(include_bytes!("../../../assets/matcap/metal_carpaint.specular.png"))),
    ("pearl", include_bytes!("../../../assets/matcap/pearl.png"), Some(include_bytes!("../../../assets/matcap/pearl.specular.png"))),
    ("red_wax", include_bytes!("../../../assets/matcap/red_wax.png"), Some(include_bytes!("../../../assets/matcap/red_wax.specular.png"))),
    ("resin", include_bytes!("../../../assets/matcap/resin.png"), Some(include_bytes!("../../../assets/matcap/resin.specular.png"))),
    ("toon_dark", include_bytes!("../../../assets/matcap/toon_dark.png"), Some(include_bytes!("../../../assets/matcap/toon_dark.specular.png"))),
    ("toon_light", include_bytes!("../../../assets/matcap/toon_light.png"), Some(include_bytes!("../../../assets/matcap/toon_light.specular.png"))),
];

/// A decoded matcap: linear RGB, `size × size`, rows top-down.
pub struct Matcap {
    pub size: u32,
    pub diffuse: Vec<f32>,
    pub specular: Option<Vec<f32>>,
}

/// A matcap of the user's own: its file and its name — the file name with
/// its extension, as `View3DShading.matcap` stores it.
struct UserMatcap {
    path: PathBuf,
    name: &'static str,
}

impl Matcap {
    /// `BKE_studiolight_find_default`'s matcap: `basic_1.exr`, which this
    /// set no longer has, so the first by name — as Blender then takes it.
    pub const DEFAULT: &'static str = "basic_bright";

    /// The atlas slots the Solid build puts the chosen matcap's images in
    /// (`rendered/atlas.rs`): Solid uploads nothing else, so they lead.
    pub const DIFFUSE_SLOT: u32 = 0;
    pub const SPECULAR_SLOT: u32 = 1;

    /// The PNG's sRGB bytes back to linear, twice (the half scale).
    fn decode(png: &[u8]) -> Option<(u32, Vec<f32>)> {
        let img = image::load_from_memory(png).ok()?.to_rgb8();
        Some((img.width(), img.as_raw().iter().map(|&c| 2.0 * phiray::Srgb::byte_to_linear(c)).collect()))
    }

    /// A user's image file as a matcap: an EXR or HDR is linear already, an
    /// LDR image is sRGB (`IMB_float_from_byte`); fitted to 256², no
    /// specular pass (`studiolight_load_matcap` finds one only in a
    /// multilayer EXR, which the decoder here does not read).
    fn decode_file(path: &std::path::Path) -> Option<Self> {
        let img = image::load_from_memory(&std::fs::read(path).ok()?).ok()?.resize_exact(256, 256, image::imageops::FilterType::Triangle);
        let deep = matches!(path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref(), Some("exr" | "hdr"));
        let diffuse = if deep {
            img.to_rgb32f().into_raw()
        } else {
            img.to_rgb8().as_raw().iter().map(|&c| phiray::Srgb::byte_to_linear(c)).collect()
        };
        Some(Self { size: 256, diffuse, specular: None })
    }

    /// Blender's user datafiles folders, every version, newest first:
    /// `%APPDATA%\Blender Foundation\Blender\<v>` on Windows,
    /// `~/Library/Application Support/Blender/<v>` on macOS,
    /// `$XDG_CONFIG_HOME/blender/<v>` elsewhere — each with
    /// `datafiles/studiolights/matcap/`.
    /// A browser has no Blender install to borrow from, and no folders to
    /// look in — so the list is empty there and the shipped matcaps are all
    /// of them. This is also what keeps `dirs` out of the wasm crate graph:
    /// `dirs-sys` reaches for platform APIs that `wasm32-unknown-unknown`
    /// does not have.
    #[cfg(target_family = "wasm")]
    fn user_dir_list() -> Vec<PathBuf> {
        Vec::new()
    }

    #[cfg(not(target_family = "wasm"))]
    fn user_dir_list() -> Vec<PathBuf> {
        let root = if cfg!(target_os = "windows") {
            dirs::config_dir().map(|d| d.join("Blender Foundation").join("Blender"))
        } else if cfg!(target_os = "macos") {
            dirs::home_dir().map(|d| d.join("Library").join("Application Support").join("Blender"))
        } else {
            dirs::config_dir().map(|d| d.join("blender"))
        };
        let Some(root) = root else { return Vec::new() };
        let mut versions: Vec<PathBuf> = std::fs::read_dir(&root).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect()).unwrap_or_default();
        versions.sort();
        versions.reverse();
        versions.into_iter().map(|v| v.join("datafiles").join("studiolights").join("matcap")).collect()
    }

    /// The image files in `dir_list`, by name; a name seen in an earlier folder
    /// wins. Names live for the program, as the shipped ones do.
    fn scan(dir_list: &[PathBuf]) -> Vec<UserMatcap> {
        let mut out: Vec<UserMatcap> = Vec::new();
        for dir in dir_list {
            let Ok(rd) = std::fs::read_dir(dir) else { continue };
            let mut files: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
            files.sort();
            for path in files {
                let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else { continue };
                if !matches!(path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref(), Some("png" | "jpg" | "jpeg" | "exr" | "hdr" | "tga" | "bmp" | "tif" | "tiff")) {
                    continue;
                }
                if out.iter().any(|m| m.name == name) {
                    continue;
                }
                out.push(UserMatcap { path, name: Box::leak(name.into_boxed_str()) });
            }
        }
        out
    }

    /// The user's matcaps, found once.
    fn user_set() -> &'static [UserMatcap] {
        static USER: OnceLock<Vec<UserMatcap>> = OnceLock::new();
        USER.get_or_init(|| Self::scan(&Self::user_dir_list()))
    }

    /// Every matcap's name: the shipped set in Blender's order (by file
    /// name), then the user's own.
    pub fn name_list() -> impl Iterator<Item = &'static str> {
        FILE_LIST.iter().map(|(n, _, _)| *n).chain(Self::user_set().iter().map(|m| m.name))
    }

    /// The matcap `name` names — shipped or the user's — or `DEFAULT` when
    /// there is none by that name or its file no longer reads (Blender's
    /// fallback for a missing studio light); decoded once.
    pub fn named(name: &str) -> Arc<Self> {
        static DECODED: OnceLock<Mutex<HashMap<&'static str, Arc<Matcap>>>> = OnceLock::new();
        let mut cache = DECODED.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
        if let Some(user) = Self::user_set().iter().find(|m| m.name == name) {
            if let Some(m) = cache.get(user.name) {
                return m.clone();
            }
            if let Some(m) = Self::decode_file(&user.path) {
                let m = Arc::new(m);
                cache.insert(user.name, m.clone());
                return m;
            }
        }
        let (name, diffuse, specular) = FILE_LIST.iter().find(|(n, _, _)| *n == name).or_else(|| FILE_LIST.iter().find(|(n, _, _)| *n == Self::DEFAULT)).expect("the default matcap ships");
        cache
            .entry(name)
            .or_insert_with(|| {
                let (size, diffuse) = Self::decode(diffuse).expect("a shipped PNG decodes");
                let specular = specular.and_then(|s| Self::decode(s)).map(|(_, s)| s);
                Arc::new(Self { size, diffuse, specular })
            })
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_matcaps_decode_and_an_unknown_name_is_the_default() {
        assert_eq!(Matcap::name_list().next(), Some(Matcap::DEFAULT));
        assert!(Matcap::name_list().take(26).count() == 26 && Matcap::name_list().take(26).all(|n| !n.contains('.')), "the shipped set leads");
        let m = Matcap::named("clay_studio");
        assert_eq!((m.size, m.diffuse.len()), (256, 256 * 256 * 3));
        assert!(m.specular.as_ref().is_some_and(|s| s.len() == m.diffuse.len()), "clay_studio has a specular pass");
        // The lit side of a matcap is its top: brighter than the bottom.
        let row = |y: usize| m.diffuse[y * 256 * 3..(y + 1) * 256 * 3].iter().sum::<f32>() / (256.0 * 3.0);
        assert!(row(64) > row(192), "{} vs {}", row(64), row(192));
        assert!(Matcap::named("check_normal+y").specular.is_none(), "a single-layer EXR has no specular");
        assert!(Arc::ptr_eq(&Matcap::named("no such matcap"), &Matcap::named(Matcap::DEFAULT)));
        assert!(Arc::ptr_eq(&Matcap::named("clay_studio"), &m), "decoded once");
    }

    #[test]
    fn a_users_folder_is_scanned_and_its_images_read_as_matcaps() {
        let dir = std::env::temp_dir().join(format!("phiber-matcap-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        // A 2 × 2 green PNG saved under a `.jpg` name (the decoder reads the
        // bytes, not the name, as Blender's IMB does), and a stray text file.
        let img = image::RgbImage::from_pixel(2, 2, image::Rgb([0, 255, 0]));
        img.save_with_format(dir.join("mine.jpg"), image::ImageFormat::Png).unwrap();
        std::fs::write(dir.join("notes.txt"), "not an image").unwrap();
        let found = Matcap::scan(&[dir.clone(), dir.join("missing")]);
        assert_eq!(found.iter().map(|m| m.name).collect::<Vec<_>>(), ["mine.jpg"], "images alone, a missing folder skipped");
        let m = Matcap::decode_file(&found[0].path).expect("decodes");
        assert_eq!((m.size, m.diffuse.len()), (256, 256 * 256 * 3));
        assert!(m.specular.is_none());
        assert!((m.diffuse[1] - 1.0).abs() < 1e-6 && m.diffuse[0] < 1e-6, "sRGB green back to linear: {:?}", &m.diffuse[..3]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
