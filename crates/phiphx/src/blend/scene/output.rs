//! Output Properties: where a render is written and in what format.
//!
//! `Scene.r.pic` is the output path and `Scene.r.im_format` an embedded
//! `ImageFormatData`, so its fields are read at
//! `offset(Scene.r) + offset(RenderData.im_format) + offset(ImageFormatData.<field>)`.
//!
//! Two things about this struct are easy to get wrong, so they are named
//! here rather than left to the reader:
//!
//! * `depth` is **not** a bit count. It is a one-of-many flag
//!   (`eImageFormatDepth`): 8 bits is `1 << 1`, 16 is `1 << 4`, 32 is
//!   `1 << 6`. `BlendOutput::bits` does that translation.
//! * the colour mode carries the *total* bit count of the channels it has
//!   (`ImColorMode`: BW 8, BW+A 16, RGB 24, RGBA 32), which is why 5.x could
//!   rename the old `planes` field to `color_mode` without changing a value.

use crate::blend::file::types::*;

/// `ImageFormatData.imtype` — the file format a render is written in.
/// Only the ones a still render can produce are named; anything else
/// (video, and the formats phiber does not write) reads as `Other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendImageFormat {
    #[default]
    Png,
    OpenExr,
    /// OpenEXR with one part per render pass. Read, but phiber writes a
    /// single combined part, so this saves like `OpenExr`.
    OpenExrMultiLayer,
    Jpeg,
    Tiff,
    Bmp,
    Targa,
    RadianceHdr,
    WebP,
    Other(i8),
}

impl BlendImageFormat {
    /// `R_IMF_IMTYPE_*` (`DNA_scene_types.h`).
    pub fn of(imtype: i8) -> Self {
        match imtype {
            17 => Self::Png,
            23 => Self::OpenExr,
            28 => Self::OpenExrMultiLayer,
            4 => Self::Jpeg,
            22 => Self::Tiff,
            20 => Self::Bmp,
            0 | 14 => Self::Targa,
            21 => Self::RadianceHdr,
            35 => Self::WebP,
            other => Self::Other(other),
        }
    }

    /// Blender's own RNA identifier, which is what the Output panel's File
    /// Format enum stores (`bl_ui/properties_output.py`). A format phiber
    /// does not write keeps its raw imtype, since only the ones the panel
    /// offers need a name.
    pub fn rna(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::OpenExr => "OPEN_EXR",
            Self::OpenExrMultiLayer => "OPEN_EXR_MULTILAYER",
            Self::Jpeg => "JPEG",
            Self::Tiff => "TIFF",
            Self::Bmp => "BMP",
            Self::Targa => "TARGA",
            Self::RadianceHdr => "HDR",
            Self::WebP => "WEBP",
            Self::Other(24) => "FFMPEG",
            Self::Other(26) => "CINEON",
            Self::Other(27) => "DPX",
            Self::Other(29) => "DDS",
            Self::Other(30) => "JPEG2000",
            Self::Other(34) => "PSD",
            Self::Other(1 | 7) => "IRIS",
            Self::Other(37) => "AVIF",
            Self::Other(_) => "",
        }
    }

    /// The extension Blender appends when "File Extensions" is on. The first
    /// of `image_path_ext_from_imformat_impl`'s array, which is the one
    /// `BLI_path_extension_ensure` actually uses. Unknown formats fall
    /// through to `.jpg` there, and do the same here.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => ".png",
            Self::OpenExr | Self::OpenExrMultiLayer => ".exr",
            Self::Tiff => ".tif",
            Self::Bmp => ".bmp",
            Self::Targa => ".tga",
            Self::RadianceHdr => ".hdr",
            Self::WebP => ".webp",
            Self::Jpeg | Self::Other(_) => ".jpg",
        }
    }

    /// Which channel depths Blender offers for this format
    /// (`BKE_imtype_valid_depths`), in bits. A depth the file asks for that
    /// is not in here is a file written by an older Blender, or by hand.
    pub fn depth_list(self) -> &'static [u8] {
        match self {
            Self::Png => &[8, 16],
            Self::OpenExr | Self::OpenExrMultiLayer => &[16, 32],
            Self::Tiff => &[8, 16],
            Self::RadianceHdr => &[32],
            _ => &[8],
        }
    }

    /// Whether the format carries more than 8 bits a channel, and so wants
    /// the render's own floats rather than the 8-bit display image.
    pub fn is_deep(self) -> bool {
        matches!(self, Self::OpenExr | Self::OpenExrMultiLayer | Self::RadianceHdr)
    }
}

/// `Scene.r.pic` plus `Scene.r.im_format` — Properties → Output.
#[derive(Debug, Clone, PartialEq)]
pub struct BlendOutput {
    /// `RenderData.pic`, Blender's default being `"//"` (next to the file).
    pub path: String,
    pub format: BlendImageFormat,
    /// Bits a channel: 8, 16 or 32, already decoded from the flag.
    pub depth: u8,
    /// Channel count from the colour mode: 1 BW, 3 RGB, 4 RGBA.
    pub channel: u8,
    /// JPEG quality and PNG compression, both 0–100.
    pub quality: u8,
    pub compress: u8,
    /// `R_IMF_EXR_CODEC_*`: 0 none, 2 ZIP, 3 PIZ …
    pub exr_codec: u8,
}

impl Default for BlendOutput {
    fn default() -> Self {
        Self {
            path: "//".to_string(),
            format: BlendImageFormat::Png,
            depth: 8,
            channel: 4,
            quality: 90,
            compress: 15,
            exr_codec: 0,
        }
    }
}

/// The extensions `BLI_path_extension_check_array` treats as an image's, so
/// that changing format replaces one rather than stacking a second on
/// (`imb_ext_image`, `imbuf/intern/util.cc`).
const IMAGE_EXTENSION: &[&str] = &[
    ".png", ".tga", ".bmp", ".jpg", ".jpeg", ".sgi", ".rgb", ".rgba", ".tif", ".tiff", ".tx", ".avif", ".jp2", ".j2c", ".hdr", ".dds", ".dpx", ".cin", ".exr", ".psd", ".pdd", ".psb", ".webp",
];

impl BlendOutput {
    /// Whether the saved file keeps an alpha channel.
    pub fn has_alpha(&self) -> bool {
        self.channel == 4 || self.channel == 2
    }

    /// The file a still render lands in, given the path the file carries and
    /// the `.blend`'s own directory for `//`-relative paths. Follows
    /// `do_makepicstring`: resolve the path, give a bare directory the name
    /// "Untitled", then ensure the format's extension.
    ///
    /// Frame numbering (`BLI_path_frame`) is not applied — that is the
    /// animation path, and a still render has one frame.
    pub fn file_of(&self, base: &std::path::Path) -> std::path::PathBuf {
        let raw = self.path.replace('\\', "/");
        let mut file = match raw.strip_prefix("//") {
            Some(rest) => base.join(rest),
            None => std::path::PathBuf::from(&raw),
        };
        if raw.is_empty() || raw.ends_with('/') {
            file.push("Untitled");
        }
        let want = self.format.extension();
        let has = file.extension().map(|e| format!(".{}", e.to_string_lossy().to_ascii_lowercase()));
        if has.as_deref() == Some(want) {
            return file;
        }
        // `do_ensure_image_extension`: an extension that is already some
        // image format is *replaced*, so "shot.png" saved as EXR becomes
        // "shot.exr" rather than "shot.png.exr". Anything else — "shot.v2",
        // "shot" — keeps what it has and gains the extension.
        let name = file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".to_string());
        let stem = match has.as_deref() {
            Some(ext) if IMAGE_EXTENSION.contains(&ext) => name[..name.len() - ext.len()].to_string(),
            _ => name,
        };
        file.set_file_name(format!("{stem}{want}"));
        file
    }
}

impl BlendFile {
    /// Read Properties → Output off the scene block.
    pub(crate) fn extract_output(&self, sc: &BlendBlock, dna: &DnaType) -> BlendOutput {
        let ps = self.header.pointer_size as usize;
        let d = &sc.data;
        let dflt = BlendOutput::default();
        let Some(r) = Self::struct_field_offset_exact("Scene", "r", dna, ps) else {
            return dflt;
        };
        let imf = |field: &str| {
            Some(r + Self::struct_field_offset_exact("RenderData", "im_format", dna, ps)?
                + Self::struct_field_offset_exact("ImageFormatData", field, dna, ps)?)
        };
        let byte = |off: Option<usize>, or: u8| match off {
            Some(o) if o < d.len() => d[o],
            _ => or,
        };
        let path = Self::struct_field_offset_exact("RenderData", "pic[1024]", dna, ps)
            .map(|o| {
                let start = (r + o).min(d.len());
                let end = (start + 1024).min(d.len());
                let b = &d[start..end];
                let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
                // Blender turns the separators native as it reads
                // (`BLI_path_slash_native`), so a path authored on Windows
                // comes back with forward slashes on any other platform.
                // `PathBuf` is happy with `/` on Windows too, so one spelling
                // is kept rather than two.
                String::from_utf8_lossy(&b[..n]).replace('\\', "/")
            })
            .filter(|p| !p.is_empty())
            .unwrap_or(dflt.path.clone());
        let format = BlendImageFormat::of(byte(imf("imtype"), 17) as i8);
        // One-of-many flag, not a count: 1<<1 is 8 bits, 1<<4 is 16, 1<<6 is 32.
        let depth = match byte(imf("depth"), 1 << 1) {
            f if f & (1 << 6) != 0 => 32,
            f if f & (1 << 4) != 0 => 16,
            f if f & (1 << 3) != 0 => 12,
            f if f & (1 << 2) != 0 => 10,
            _ => 8,
        };
        // 5.x renamed `planes` to `color_mode` keeping the same values, so
        // either name answers; files older than that only have `planes`.
        let channel = match byte(imf("color_mode").or_else(|| imf("planes")), 32) {
            8 => 1,
            16 => 2,
            24 => 3,
            _ => 4,
        };
        BlendOutput {
            path,
            format,
            depth,
            channel,
            quality: byte(imf("quality"), 90),
            compress: byte(imf("compress"), 15),
            exr_codec: byte(imf("exr_codec"), 0),
        }
    }
}
