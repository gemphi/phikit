//! Writing a finished render to disk, in the format Properties → Output asks
//! for (`phiphx::BlendOutput`, read from `Scene.r.pic` and `Scene.r.im_format`).
//!
//! Blender ref: `BKE_image_format.hh` for what each format may hold, and
//! `IMB_saveiff` for the save itself. The rule that matters here is which
//! buffer a format reads:
//!
//! * **8 bits a channel** — the bytes already on screen. The view transform
//!   has been applied and the value quantised; there is nothing more to say.
//! * **16-bit PNG** — the *display* floats. The image has been through the
//!   view transform, exactly like the 8-bit one, but is quantised to 16 bits
//!   instead of 8. Saving a 16-bit PNG from scene-linear radiance would be
//!   wrong: Blender's PNG is a display-referred file whatever its depth.
//! * **EXR and Radiance HDR** — the scene-linear radiance, *before* the view
//!   transform. These are the deep formats, and baking Filmic or AgX into one
//!   would destroy the very thing it is for.
//!
//! Getting that split backwards is the classic mistake, so the three cases
//! are separate functions rather than one with a flag.

use crate::v2::scene::render_job::BRenderShot;
use phiphx::blend::{BPassKind, BlendImageFormat, BlendOutput, BlendPassSet};
use std::path::{Path, PathBuf};

/// Which write a `BlendOutput` asks for, once its format and depth are read
/// together. `Shallow` covers every 8-bit format — `image` dispatches those
/// on the extension, so they do not each need a case here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BSaveKind {
    /// 8 bits a channel, written from the displayed bytes.
    Shallow,
    /// 16-bit PNG, written from the display floats.
    DeepPng,
    /// OpenEXR half (16) or float (32), scene-linear.
    Exr { half: bool },
    /// The same, with one layer per render pass.
    ExrMultiLayer { half: bool },
    /// Radiance HDR — RGBE, so always 32-bit input and scene-linear.
    Hdr,
}

/// What `output` asks to be written, with the depth clamped to one the format
/// actually offers (`BKE_imtype_valid_depths`): a file can carry a depth an
/// older Blender allowed, and Blender itself falls back rather than refusing.
pub fn kind_of(output: &BlendOutput) -> BSaveKind {
    let depth = if output.format.depth_list().contains(&output.depth) {
        output.depth
    } else {
        output.format.depth_list().first().copied().unwrap_or(8)
    };
    match output.format {
        BlendImageFormat::OpenExr => BSaveKind::Exr { half: depth == 16 },
        BlendImageFormat::OpenExrMultiLayer => BSaveKind::ExrMultiLayer { half: depth == 16 },
        BlendImageFormat::RadianceHdr => BSaveKind::Hdr,
        BlendImageFormat::Png if depth == 16 => BSaveKind::DeepPng,
        _ => BSaveKind::Shallow,
    }
}

/// Write `shot` where `output` says, relative to the `.blend`'s own folder
/// for a `//` path. Returns the file it wrote.
pub fn save(shot: &BRenderShot, output: &BlendOutput, pass: BlendPassSet, blend_folder: &Path) -> Result<PathBuf, String> {
    let file = output.file_of(blend_folder);
    save_to(shot, output, pass, &file)?;
    Ok(file)
}

/// The file a turnaround view lands in: the still's name with `_<view>`
/// before the extension, any frame `#` run dropped — `render_front.png`.
pub fn view_file(output: &BlendOutput, blend_folder: &Path, view: &str) -> PathBuf {
    let still = output.file_of(blend_folder);
    let name = still.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".into());
    let (stem, ext) = match name.rfind('.') {
        Some(dot) => (name[..dot].to_string(), name[dot..].to_string()),
        None => (name.clone(), String::new()),
    };
    let stem: String = stem.chars().filter(|c| *c != '#').collect();
    let stem = stem.trim_end_matches(['_', '-', ' ']).to_string();
    still.with_file_name(format!("{}_{view}{ext}", if stem.is_empty() { "render" } else { &stem }))
}

/// The file frame `frame` of an animation lands in (`BLI_path_frame`): a
/// run of `#` in the name takes the zero-padded frame, else four digits go
/// before the extension — `render_0001.png`.
pub fn frame_file(output: &BlendOutput, blend_folder: &Path, frame: i32) -> PathBuf {
    let still = output.file_of(blend_folder);
    let name = still.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".into());
    let (stem, ext) = match name.rfind('.') {
        Some(dot) => (name[..dot].to_string(), name[dot..].to_string()),
        None => (name.clone(), String::new()),
    };
    let numbered = match stem.find('#') {
        Some(first) => {
            let hashes = stem[first..].chars().take_while(|c| *c == '#').count();
            format!("{}{:0width$}{}", &stem[..first], frame, &stem[first + hashes..], width = hashes)
        }
        None => format!("{stem}{frame:04}"),
    };
    still.with_file_name(format!("{numbered}{ext}"))
}

/// The same write to a path chosen by the caller — what Image → Save As does,
/// and what a test uses.
///
/// Refuses a render that has not settled: a frame under way has the bytes
/// the viewer shows (`BRenderShot::peek`) but not the floats a file is
/// written from, and one that never drew has nothing at all.
pub fn save_to(shot: &BRenderShot, output: &BlendOutput, pass: BlendPassSet, file: &Path) -> Result<(), String> {
    if !shot.has_image() {
        return Err("the render has no image yet".to_string());
    }
    if !shot.settled() {
        return Err("the render is still running — wait for it to finish, or cancel it, to save what it has".to_string());
    }
    if let Some(folder) = file.parent().filter(|folder| !folder.as_os_str().is_empty()) {
        std::fs::create_dir_all(folder).map_err(|err| format!("{}: {err}", folder.display()))?;
    }
    match kind_of(output) {
        BSaveKind::Shallow => save_shallow(shot, output, file),
        BSaveKind::DeepPng => save_deep_png(shot, output, file),
        BSaveKind::Exr { half } => save_exr(shot, output, file, half),
        BSaveKind::ExrMultiLayer { half } => save_exr_layers(shot, pass, file, half),
        BSaveKind::Hdr => save_hdr(shot, file),
    }
}

/// The displayed bytes, as RGBA8 (or RGB8 for a format with no alpha).
/// `image` picks the encoder from the extension, so this one arm covers PNG,
/// JPEG, BMP, TIFF, Targa and WebP alike.
fn save_shallow(shot: &BRenderShot, output: &BlendOutput, file: &Path) -> Result<(), String> {
    let rgba: Vec<u8> = swapped(&shot.bgra);
    let fail = |err: image::ImageError| format!("{}: {err}", file.display());
    if output.has_alpha() && !matches!(output.format, BlendImageFormat::Jpeg) {
        let buf = image::RgbaImage::from_raw(shot.w, shot.h, rgba).ok_or("the image is the wrong size")?;
        buf.save(file).map_err(fail)
    } else {
        let rgb: Vec<u8> = rgba.chunks_exact(4).flat_map(|px| [px[0], px[1], px[2]]).collect();
        let buf = image::RgbImage::from_raw(shot.w, shot.h, rgb).ok_or("the image is the wrong size")?;
        buf.save(file).map_err(fail)
    }
}

/// A 16-bit PNG from the display floats. The alpha comes from the 8-bit image
/// — the tracer keeps coverage as a byte — so it is widened rather than
/// requantised (`255 → 65535`, which is `× 257`, not `× 256`).
fn save_deep_png(shot: &BRenderShot, output: &BlendOutput, file: &Path) -> Result<(), String> {
    let quantise = |value: f32| (value.clamp(0.0, 1.0) * 65535.0).round() as u16;
    let fail = |err: image::ImageError| format!("{}: {err}", file.display());
    if output.has_alpha() {
        let mut deep: Vec<u16> = Vec::with_capacity(shot.display.len() * 4);
        for (px, bgra) in shot.display.iter().zip(shot.bgra.chunks_exact(4)) {
            deep.extend(px.map(quantise));
            deep.push(u16::from(bgra[3]) * 257);
        }
        let buf = image::ImageBuffer::<image::Rgba<u16>, _>::from_raw(shot.w, shot.h, deep).ok_or("the image is the wrong size")?;
        buf.save(file).map_err(fail)
    } else {
        let deep: Vec<u16> = shot.display.iter().flat_map(|px| px.map(quantise)).collect();
        let buf = image::ImageBuffer::<image::Rgb<u16>, _>::from_raw(shot.w, shot.h, deep).ok_or("the image is the wrong size")?;
        buf.save(file).map_err(fail)
    }
}

/// Scene-linear OpenEXR. `half` writes 16-bit samples, which is Blender's
/// default for the format and halves the file; otherwise 32-bit float.
///
/// Written through `exr` rather than `image`, whose encoder is float-only —
/// a half file asked for in the `.blend` would silently come back twice the
/// size, which is the kind of quiet mismatch this crate exists to avoid.
fn save_exr(shot: &BRenderShot, output: &BlendOutput, file: &Path, half: bool) -> Result<(), String> {
    use exr::prelude::*;
    let (width, height) = (shot.w as usize, shot.h as usize);
    let linear = shot.linear.clone();
    let coverage = if output.has_alpha() { shot.bgra.clone() } else { Vec::new() };
    let keep_alpha = output.has_alpha();
    let at = move |position: Vec2<usize>| -> (f32, f32, f32, f32) {
        let index = position.y() * width + position.x();
        let px = linear.get(index).copied().unwrap_or([0.0; 3]);
        // No alpha channel in the format means an opaque image, not a
        // transparent one — Blender writes 1.0 rather than leaving it unset.
        let alpha = if keep_alpha { coverage.get(index * 4 + 3).map_or(1.0, |byte| f32::from(*byte) / 255.0) } else { 1.0 };
        (px[0], px[1], px[2], alpha)
    };
    let fail = |err: exr::error::Error| format!("{}: {err}", file.display());
    if half {
        let channel = SpecificChannels::rgba(move |position: Vec2<usize>| {
            let (red, green, blue, alpha) = at(position);
            (f16::from_f32(red), f16::from_f32(green), f16::from_f32(blue), f16::from_f32(alpha))
        });
        Image::from_channels(Vec2(width, height), channel).write().to_file(file).map_err(fail)
    } else {
        let channel = SpecificChannels::rgba(at);
        Image::from_channels(Vec2(width, height), channel).write().to_file(file).map_err(fail)
    }
}

/// Radiance HDR — RGBE, scene-linear, and no alpha channel in the format.
fn save_hdr(shot: &BRenderShot, file: &Path) -> Result<(), String> {
    let px: Vec<image::Rgb<f32>> = shot.linear.iter().map(|c| image::Rgb(*c)).collect();
    let out = std::fs::File::create(file).map_err(|err| format!("{}: {err}", file.display()))?;
    image::codecs::hdr::HdrEncoder::new(std::io::BufWriter::new(out))
        .encode(&px, shot.w as usize, shot.h as usize)
        .map_err(|err| format!("{}: {err}", file.display()))
}

/// BGRA back to RGBA. The job stores what GPUI paints; a file wants the other
/// order.
fn swapped(bgra: &[u8]) -> Vec<u8> {
    let mut rgba = bgra.to_vec();
    rgba.chunks_exact_mut(4).for_each(|px| px.swap(0, 2));
    rgba
}


/// A multi-layer OpenEXR: one layer per render pass, named the way Blender
/// names them (`ViewLayer.<Pass>.<Channel>`, from `phiphx::BPassKind`).
///
/// Every layer is written into one part rather than one part each, because
/// that is what Blender itself produces — its `ImageFormatData` carries a
/// multipart flag, but a rendered `.exr` comes back single-part with the
/// layer name in every channel's own name.
///
/// A pass the tracer cannot produce is left out rather than written as
/// black: an empty layer in a compositor looks like a rendered one that came
/// out wrong, which costs more time than a missing layer does.
fn save_exr_layers(shot: &BRenderShot, pass: BlendPassSet, file: &Path, half: bool) -> Result<(), String> {
    use exr::prelude::*;
    let size = Vec2(shot.w as usize, shot.h as usize);
    let plane = |source: &[[f32; 3]], lane: usize| -> Vec<f32> { source.iter().map(|px| px[lane]).collect() };
    let mut layer_list: Vec<Layer<AnyChannels<FlatSamples>>> = Vec::new();

    for kind in pass.list() {
        // The channel names carry the layer's name as well as their own —
        // `ViewLayer.Depth.Z`, not `Z` — because that is what Blender
        // writes, and a compositor matches on the whole string.
        let named = |channel: &str| format!("ViewLayer.{}.{channel}", kind.name());
        let plane_list: Vec<(String, Vec<f32>)> = match kind {
            BPassKind::Combined => {
                let mut out: Vec<(String, Vec<f32>)> = ["R", "G", "B"].iter().enumerate().map(|(lane, name)| (named(name), plane(&shot.linear, lane))).collect();
                // The tracer keeps coverage as a byte, so alpha is widened
                // from the displayed image rather than carried as a float.
                out.push((named("A"), shot.bgra.chunks_exact(4).map(|px| f32::from(px[3]) / 255.0).collect()));
                out
            }
            BPassKind::Depth if !shot.depth.is_empty() => vec![(named("Z"), shot.depth.clone())],
            BPassKind::Normal if !shot.normal.is_empty() => ["X", "Y", "Z"].iter().enumerate().map(|(lane, name)| (named(name), plane(&shot.normal, lane))).collect(),
            BPassKind::DiffuseColor if !shot.albedo.is_empty() => ["R", "G", "B"].iter().enumerate().map(|(lane, name)| (named(name), plane(&shot.albedo, lane))).collect(),
            // Asked for, but not traced yet. An empty layer in a compositor
            // reads as a render that came out wrong, which costs more time
            // than a missing one does, so it is left out instead.
            _ => Vec::new(),
        };
        if plane_list.is_empty() {
            continue;
        }
        let channel_list: Vec<AnyChannel<FlatSamples>> = plane_list
            .into_iter()
            .map(|(name, data)| {
                let samples = if half { FlatSamples::F16(data.into_iter().map(f16::from_f32).collect()) } else { FlatSamples::F32(data) };
                AnyChannel::new(Text::from(name.as_str()), samples)
            })
            .collect();
        layer_list.push(Layer::new(
            size,
            LayerAttributes::named(Text::from(format!("ViewLayer.{}", kind.name()).as_str())),
            Encoding::FAST_LOSSLESS,
            AnyChannels::sort(channel_list.into_iter().collect()),
        ));
    }

    // One part per pass, which is what `R_IMF_EXR_FLAG_MULTIPART` means and
    // what a rendered `.exr` from Blender actually contains. Writing every
    // pass into a single part instead reads back as one layer, and the
    // passes disappear.
    Image::from_layers(ImageAttributes::with_size(size), layer_list)
        .write()
        .to_file(file)
        .map_err(|err| format!("{}: {err}", file.display()))
}

/// Which passes a multi-layer save will actually contain, given what was
/// asked for — so a caller can say what it left out rather than letting the
/// difference be discovered in a compositor.
pub fn written_pass(shot: &BRenderShot, pass: BlendPassSet) -> Vec<BPassKind> {
    pass.list()
        .into_iter()
        .filter(|kind| match kind {
            BPassKind::Combined => true,
            BPassKind::Depth => !shot.depth.is_empty(),
            BPassKind::Normal => !shot.normal.is_empty(),
            BPassKind::DiffuseColor => !shot.albedo.is_empty(),
            _ => false,
        })
        .collect()
}

#[cfg(test)]
mod frame_tests {
    use super::*;

    #[test]
    fn a_render_under_way_is_refused_by_the_writer() {
        let peek = BRenderShot { w: 2, h: 2, bgra: vec![255; 16], taken: 1, target: 8, ..Default::default() };
        let dir = std::env::temp_dir().join("phiber_peek_save");
        let err = save_to(&peek, &BlendOutput::default(), BlendPassSet::default(), &dir.join("peek.png")).unwrap_err();
        assert!(err.contains("still running"), "{err}");
        assert!(!dir.join("peek.png").exists());
    }

    #[test]
    fn frames_are_numbered_like_blender_numbers_them() {
        let mut output = BlendOutput::default();
        output.path = "//render/shot_".into();
        let file = frame_file(&output, Path::new("/tmp/proj"), 7);
        assert!(file.to_string_lossy().replace('\\', "/").ends_with("render/shot_0007.png"), "{}", file.display());
        output.path = "//render/shot_##".into();
        let file = frame_file(&output, Path::new("/tmp/proj"), 7);
        assert!(file.to_string_lossy().replace('\\', "/").ends_with("render/shot_07.png"), "{}", file.display());
    }

    #[test]
    fn a_turnaround_view_takes_the_stills_name_with_its_view_before_the_extension() {
        let mut output = BlendOutput::default();
        output.path = "//render/shot_".into();
        let file = view_file(&output, Path::new("/tmp/proj"), "front");
        assert!(file.to_string_lossy().replace('\\', "/").ends_with("render/shot_front.png"), "{}", file.display());
        // A frame `#` run is an animation's, not a view's: dropped.
        output.path = "//render/shot_####".into();
        let file = view_file(&output, Path::new("/tmp/proj"), "top");
        assert!(file.to_string_lossy().replace('\\', "/").ends_with("render/shot_top.png"), "{}", file.display());
    }
}
