//! The images open in the session beside the Render Result — Blender's
//! `Image` data-blocks the Image editor's `template_ID` lists: one opened
//! from a file (`image.open`), one made blank or as a grid (`image.new`,
//! `BKE_image_buf_fill_*`, `image_gen.cc`), and the pixel operators the
//! Image menu runs on whichever is shown (`image.invert`, `image.flip`,
//! `image.rotate_orthogonal`). Kept process-wide as the renders are
//! (`render_job`), BGRA rows top down as GPUI paints them; `seq` moves on
//! with every change so the canvas redraws.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

/// One image: its name as the data-block's, where it came from, its pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct BImage {
    pub name: String,
    pub path: Option<PathBuf>,
    pub w: u32,
    pub h: u32,
    pub bgra: Vec<u8>,
}

/// `image.new`'s Generated Type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BGenerated {
    Blank,
    UvGrid,
    ColorGrid,
}

impl BGenerated {
    pub const NAME_LIST: [&'static str; 3] = ["Blank", "UV Grid", "Color Grid"];
    pub fn of_name(name: &str) -> Self {
        match name {
            "UV Grid" => Self::UvGrid,
            "Color Grid" => Self::ColorGrid,
            _ => Self::Blank,
        }
    }
}

/// The Image menu's pixel operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BPixelOp {
    /// `image.invert` with its channel flags `[r, g, b, a]`.
    Invert([bool; 4]),
    /// `image.flip`: `use_flip_x`, `use_flip_y`.
    Flip(bool, bool),
    /// `image.rotate_orthogonal`: 90, 180 or 270 degrees clockwise.
    Rotate(u32),
}

impl BPixelOp {
    /// The menu's slug: `invert:rgb`, `invert:r|g|b|a`, `flip:x|y`, `rotate:90|180|270`.
    pub fn of_slug(slug: &str) -> Option<Self> {
        Some(match slug {
            "invert:rgb" => Self::Invert([true, true, true, false]),
            "invert:r" => Self::Invert([true, false, false, false]),
            "invert:g" => Self::Invert([false, true, false, false]),
            "invert:b" => Self::Invert([false, false, true, false]),
            "invert:a" => Self::Invert([false, false, false, true]),
            "flip:x" => Self::Flip(true, false),
            "flip:y" => Self::Flip(false, true),
            "rotate:90" | "rotate:180" | "rotate:270" => Self::Rotate(slug[7..].parse().ok()?),
            _ => return None,
        })
    }

    /// Over BGRA rows of a `w × h` picture; the size after (a quarter turn
    /// swaps the sides).
    pub fn apply(self, w: u32, h: u32, bgra: &mut Vec<u8>) -> (u32, u32) {
        let (w, h) = (w as usize, h as usize);
        let at = |x: usize, y: usize| 4 * (y * w + x);
        match self {
            Self::Invert([r, g, b, a]) => {
                for px in bgra.chunks_exact_mut(4) {
                    for (i, on) in [b, g, r, a].into_iter().enumerate() {
                        if on {
                            px[i] = 255 - px[i];
                        }
                    }
                }
                (w as u32, h as u32)
            }
            Self::Flip(x, y) => {
                let src = bgra.clone();
                for row in 0..h {
                    for col in 0..w {
                        let (sx, sy) = (if x { w - 1 - col } else { col }, if y { h - 1 - row } else { row });
                        bgra[at(col, row)..at(col, row) + 4].copy_from_slice(&src[at(sx, sy)..at(sx, sy) + 4]);
                    }
                }
                (w as u32, h as u32)
            }
            Self::Rotate(180) => Self::Flip(true, true).apply(w as u32, h as u32, bgra),
            Self::Rotate(deg) => {
                // Clockwise on the screen's rows (row 0 at the top): the
                // pixel at (x, y) lands at (h − 1 − y, x) for 90°.
                let src = bgra.clone();
                let (nw, nh) = (h, w);
                for row in 0..h {
                    for col in 0..w {
                        let (dx, dy) = if deg == 90 { (h - 1 - row, col) } else { (row, w - 1 - col) };
                        let d = 4 * (dy * nw + dx);
                        bgra[d..d + 4].copy_from_slice(&src[at(col, row)..at(col, row) + 4]);
                    }
                }
                (nw as u32, nh as u32)
            }
        }
    }
}

impl BImage {
    /// `image.open`: the file read, named as the file is (`BLI_path_basename`).
    pub fn open(path: &Path) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("{}: {e}", path.display()))?.to_rgba8();
        let (w, h) = img.dimensions();
        let mut bgra = img.into_raw();
        bgra.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".into());
        Ok(Self { name, path: Some(path.to_path_buf()), w, h, bgra })
    }

    /// `image.reload`: read again from where it came; nothing for a made one.
    pub fn reload(&mut self) -> Result<(), String> {
        let Some(path) = self.path.clone() else { return Ok(()) };
        let again = Self::open(&path)?;
        (self.w, self.h, self.bgra) = (again.w, again.h, again.bgra);
        Ok(())
    }

    /// `image.new`: `w × h` of `color` (display-referred, as the dialog's
    /// swatch), or the UV / Color Grid (`BKE_image_buf_fill_checker`,
    /// `_checker_color` without the lettering); `alpha` off keeps it opaque.
    pub fn generated(name: &str, w: u32, h: u32, color: [f32; 4], alpha: bool, kind: BGenerated) -> Self {
        let (w, h) = (w.max(1), h.max(1));
        let n = (w * h) as usize;
        let mut bgra = match kind {
            BGenerated::Blank => {
                let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                [byte(color[2]), byte(color[1]), byte(color[0]), byte(color[3])].repeat(n)
            }
            BGenerated::UvGrid => super::image_gen::BImageGrid::uv(w, h),
            BGenerated::ColorGrid => super::image_gen::BImageGrid::color(w, h),
        };
        if !alpha {
            bgra.chunks_exact_mut(4).for_each(|p| p[3] = 255);
        }
        Self { name: name.to_string(), path: None, w, h, bgra }
    }

    /// The pixels as a PNG (the clipboard's, `image.clipboard_copy`; a save).
    pub fn png(&self) -> Vec<u8> {
        let mut rgba = self.bgra.clone();
        rgba.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
        let mut out = std::io::Cursor::new(Vec::new());
        if let Some(img) = image::RgbaImage::from_raw(self.w, self.h, rgba) {
            let _ = img.write_to(&mut out, image::ImageFormat::Png);
        }
        out.into_inner()
    }

    /// `image.save_as`: written in the format the extension names.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let mut rgba = self.bgra.clone();
        rgba.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
        let img = image::RgbaImage::from_raw(self.w, self.h, rgba).ok_or("no pixels")?;
        img.save(path).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn run(&mut self, op: BPixelOp) {
        (self.w, self.h) = op.apply(self.w, self.h, &mut self.bgra);
    }
}

struct BStore {
    list: Mutex<Vec<BImage>>,
    seq: AtomicU32,
}

fn store() -> &'static BStore {
    static STORE: OnceLock<BStore> = OnceLock::new();
    STORE.get_or_init(|| BStore { list: Mutex::new(Vec::new()), seq: AtomicU32::new(0) })
}

/// The session's images, one owner over the store.
pub struct BImageList;

impl BImageList {
    /// Add `image`, its name made unique as Blender's `.001` does; the name kept.
    pub fn add(mut image: BImage) -> String {
        let s = store();
        if let Ok(mut list) = s.list.lock() {
            let base = image.name.clone();
            let mut n = 1;
            while list.iter().any(|i| i.name == image.name) {
                image.name = format!("{base}.{n:03}");
                n += 1;
            }
            let name = image.name.clone();
            list.push(image);
            s.seq.fetch_add(1, Ordering::Relaxed);
            return name;
        }
        image.name
    }

    pub fn get(name: &str) -> Option<BImage> {
        store().list.lock().ok().and_then(|l| l.iter().find(|i| i.name == name).cloned())
    }

    pub fn name_list() -> Vec<String> {
        store().list.lock().map(|l| l.iter().map(|i| i.name.clone()).collect()).unwrap_or_default()
    }

    /// Change the named image in place; `true` when it was there.
    pub fn with(name: &str, change: impl FnOnce(&mut BImage)) -> bool {
        let s = store();
        let Ok(mut list) = s.list.lock() else { return false };
        let Some(image) = list.iter_mut().find(|i| i.name == name) else { return false };
        change(image);
        s.seq.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn seq() -> u32 {
        store().seq.load(Ordering::Relaxed)
    }

    /// A new document: no images (`forget`, as the renders are forgotten).
    pub fn forget() {
        let s = store();
        if let Ok(mut list) = s.list.lock() {
            list.clear();
        }
        s.seq.fetch_add(1, Ordering::Relaxed);
    }
}
