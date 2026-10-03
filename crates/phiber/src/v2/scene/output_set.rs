//! What the Properties → Output panel asks for, read back out of the state.
//!
//! `seed_output_props` writes the file's own values into `ctx.buts.*` when a
//! `.blend` is opened, and the panel's controls overwrite them as the user
//! edits. This reads them back into the same `BlendOutput` the reader
//! produces, so the writer (`render_save.rs`) takes one type whether the
//! settings came from the file or from the panel.
//!
//! The round trip matters: a user who switches File Format to OpenEXR in the
//! panel and then saves must get an EXR, not the PNG the file was saved with.

use crate::v2::state::{BState, ButsKey};
use phiphx::blend::{BPassKind, BlendImageFormat, BlendOutput, BlendPassSet};

/// The RNA identifiers the panel's File Format enum offers
/// (`part/props/output.rs`), back to the imtype they stand for.
fn format_of(rna: &str) -> BlendImageFormat {
    match rna {
        "OPEN_EXR" => BlendImageFormat::OpenExr,
        "OPEN_EXR_MULTILAYER" => BlendImageFormat::OpenExrMultiLayer,
        "JPEG" => BlendImageFormat::Jpeg,
        "TIFF" => BlendImageFormat::Tiff,
        "BMP" => BlendImageFormat::Bmp,
        "TARGA" => BlendImageFormat::Targa,
        "HDR" => BlendImageFormat::RadianceHdr,
        "WEBP" => BlendImageFormat::WebP,
        "PNG" => BlendImageFormat::Png,
        // FFmpeg Video: an animation render encodes a movie (`scene::video`);
        // a still saved under it is a PNG, as Blender's Image ▸ Save As
        // falls back to (`BKE_image_format_init_for_write`).
        "FFMPEG" => BlendImageFormat::Other(24),
        // A format the panel lists but phiber does not write (the AVI pair).
        // Keeping the default here rather than inventing an imtype means a
        // save still produces something rather than failing.
        _ => BlendImageFormat::Png,
    }
}

/// Read the Output panel. Anything the state does not hold keeps the value a
/// fresh `.blend` would have.
/// Whether the Output panel asks for a movie rather than an image sequence.
pub fn wants_movie(output: &BlendOutput) -> bool {
    output.format == BlendImageFormat::Other(24)
}

/// The output with a movie format replaced by PNG: what one frame, or a
/// still, is written as on its way into (or instead of) the movie.
pub fn as_still(output: &BlendOutput) -> BlendOutput {
    if wants_movie(output) { BlendOutput { format: BlendImageFormat::Png, ..output.clone() } } else { output.clone() }
}

/// Output ▸ Encoding as the panel has it, over the file's `movie` for what
/// the panel leaves unsaid — Blender's RNA identifiers to the DNA values
/// (`rna_scene.cc`'s `ffmpeg_format_items`, `ffmpeg_codec_items`, …).
pub fn movie_set(state: &BState, file: crate::phiphx::blend::BlendMovie) -> crate::phiphx::blend::BlendMovie {
    let text = |key: &'static str| state.get(ButsKey::Value(key)).filter(|v| !v.is_empty());
    let int = |key: &'static str, or: i32| text(key).and_then(|v| v.parse::<f64>().ok()).map_or(or, |v| v as i32);
    let container = match text("ffmpeg.format") {
        Some("MPEG1") => 0, Some("MPEG2") => 1, Some("MPEG4") => 2, Some("AVI") => 3, Some("QUICKTIME") => 4,
        Some("DV") => 5, Some("FLASH") => 8, Some("MKV") => 9, Some("OGG") => 10, Some("WEBM") => 12,
        _ => file.container,
    };
    let codec = match text("ffmpeg.codec") {
        Some("NONE") => 0, Some("H264") => 27, Some("H265") => 173, Some("AV1") => 226, Some("DNXHD") => 99, Some("DV") => 24,
        Some("FFV1") => 33, Some("FLASH") => 21, Some("HUFFYUV") => 25, Some("MPEG1") => 1, Some("MPEG2") => 2, Some("MPEG4") => 12,
        Some("PNG") => 61, Some("QTRLE") => 55, Some("THEORA") => 30, Some("WEBM") => 167, Some("PRORES") => 147,
        _ => file.codec,
    };
    let crf = match text("ffmpeg.constant_rate_factor") {
        Some("NONE") => -1, Some("LOSSLESS") => 0, Some("PERC_LOSSLESS") => 17, Some("HIGH") => 20, Some("MEDIUM") => 23,
        Some("LOW") => 26, Some("VERYLOW") => 29, Some("LOWEST") => 32,
        _ => file.crf,
    };
    let preset = match text("ffmpeg.ffmpeg_preset") {
        Some("GOOD") => 10, Some("BEST") => 11, Some("REALTIME") => 12,
        _ => file.preset,
    };
    crate::phiphx::blend::BlendMovie { container, codec, crf, preset, gop: int("ffmpeg.gop_size", file.gop), bitrate: int("ffmpeg.video_bitrate", file.bitrate), ..file }
}

pub fn output_set(state: &BState) -> BlendOutput {
    let fallback = BlendOutput::default();
    let text = |key: &'static str| state.get(ButsKey::Value(key)).filter(|v| !v.is_empty()).map(str::to_string);
    let format = text("image_settings.file_format").map_or(fallback.format, |rna| format_of(&rna));
    let depth = text("image_settings.color_depth").and_then(|d| d.parse().ok()).unwrap_or(fallback.depth);
    let channel = match text("image_settings.color_mode").as_deref() {
        Some("BW") => 1,
        Some("BW_A") => 2,
        Some("RGB") => 3,
        Some("RGBA") => 4,
        _ => fallback.channel,
    };
    BlendOutput { path: text("filepath").unwrap_or(fallback.path), format, depth, channel, ..fallback }
}


/// The View Layer panel's pass toggle for each pass
/// (`part/props/view_layer.rs`, which follows `bl_ui/properties_view_layer.py`).
/// Blender's RNA names do not all match the pass name: Depth is
/// `use_pass_z` and Emission is `use_pass_emit`.
fn toggle_of(kind: BPassKind) -> Option<&'static str> {
    Some(match kind {
        BPassKind::Combined => "use_pass_combined",
        BPassKind::Depth => "use_pass_z",
        BPassKind::Mist => "use_pass_mist",
        BPassKind::Normal => "use_pass_normal",
        BPassKind::Position => "use_pass_position",
        BPassKind::Vector => "use_pass_vector",
        BPassKind::Uv => "use_pass_uv",
        BPassKind::ObjectIndex => "use_pass_object_index",
        BPassKind::MaterialIndex => "use_pass_material_index",
        BPassKind::DiffuseColor => "use_pass_diffuse_color",
        BPassKind::Emission => "use_pass_emit",
        BPassKind::Environment => "use_pass_environment",
        BPassKind::AmbientOcclusion => "use_pass_ambient_occlusion",
    })
}

/// Read the View Layer panel's pass toggles back into the bitfield the file
/// stores. A toggle the state does not hold keeps the file's own bit, so a
/// panel the user has not touched does not silently clear what the file
/// asked for.
pub fn pass_set(state: &BState, from_file: BlendPassSet) -> BlendPassSet {
    let mut flag = from_file.flag;
    for kind in BPassKind::list() {
        let Some(key) = toggle_of(*kind) else { continue };
        match state.get(ButsKey::Value(key)) {
            Some("true") => flag |= kind.bit(),
            Some("false") => flag &= !kind.bit(),
            _ => {}
        }
    }
    BlendPassSet { flag }
}

/// Write a file's pass flags into the View Layer panel, so the toggles show
/// what the file asks for (`app/seed.rs` calls this on open).
pub fn pass_state(set: BlendPassSet) -> Vec<(&'static str, &'static str)> {
    BPassKind::list().iter().filter_map(|kind| Some((toggle_of(*kind)?, if set.has(*kind) { "true" } else { "false" }))).collect()
}

#[cfg(test)]
mod movie_tests {
    use super::*;
    use crate::phiphx::blend::{BlendImageFormat, BlendMovie};

    #[test]
    fn the_encoding_panel_overrides_the_file_and_ffmpeg_video_is_a_movie() {
        let mut s = BState::new();
        let file = BlendMovie { container: BlendMovie::MPEG4, codec: 27, crf: 23, ..BlendMovie::default() };
        assert_eq!(movie_set(&s, file), file, "nothing set: the file's");
        s.set(ButsKey::Value("ffmpeg.format"), "WEBM");
        s.set(ButsKey::Value("ffmpeg.codec"), "WEBM");
        s.set(ButsKey::Value("ffmpeg.constant_rate_factor"), "HIGH");
        s.set(ButsKey::Value("ffmpeg.ffmpeg_preset"), "BEST");
        s.set(ButsKey::Value("ffmpeg.gop_size"), "12");
        let set = movie_set(&s, file);
        assert_eq!((set.container, set.codec, set.crf, set.preset, set.gop), (12, 167, 20, 11, 12));
        assert_eq!(set.extension(), ".webm");
        s.set(ButsKey::Value("image_settings.file_format"), "FFMPEG");
        let out = output_set(&s);
        assert!(wants_movie(&out), "{out:?}");
        assert_eq!(as_still(&out).format, BlendImageFormat::Png, "a frame on its way to the encoder is a PNG");
    }
}
