//! Output context — `bl_ui/properties_output.py`.
// Output context set — sections + RNA properties for `bpy.types.RenderSettings`
// output side. Blender ref: `bl_ui/properties_output.py`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

static IMAGE_FORMAT: &[BEnumItem] = &[
    en("PNG", "PNG"),
    en("JPEG", "JPEG"),
    en("TIFF", "TIFF"),
    en("BMP", "BMP"),
    en("OPEN_EXR", "OpenEXR"),
    en("OPEN_EXR_MULTILAYER", "OpenEXR MultiLayer"),
    en("HDR", "Radiance HDR"),
    en("CINEON", "Cineon"),
    en("DPX", "DPX"),
    en("WEBP", "WebP"),
    en("FFMPEG", "FFmpeg Video"),
    en("AVI_JPEG", "AVI JPEG"),
    en("AVI_RAW", "AVI Raw"),
];
static COLOR_MODE: &[BEnumItem] = &[en("BW", "BW"), en("RGB", "RGB"), en("RGBA", "RGBA")];
static COLOR_DEPTH: &[BEnumItem] = &[
    en("8", "8"),
    en("10", "10"),
    en("12", "12"),
    en("16", "16"),
    en("32", "32"),
];

static FORMAT: &[BPropSpec] = &[
    i("resolution_x", "Resolution X", 0, 65536),
    i("resolution_y", "Resolution Y", 0, 65536),
    i("resolution_percentage", "Resolution %", 1, 32767),
    b("use_border", "Render Region"),
    b("use_crop_to_border", "Crop to Render Region"),
    i("pixel_aspect_x", "Pixel Aspect X", 1, 200),
    i("pixel_aspect_y", "Pixel Aspect Y", 1, 200),
    i("fps", "Frame Rate", 1, 32767),
    f("fps_base", "Frame Rate Base", BUnit::None),
];

static STEREOSCOPY: &[BPropSpec] = &[
    b("use_multiview", "Stereoscopy"),
    e(
        "views_format",
        "Views Format",
        &[en("STEREO_3D", "Stereo 3D"), en("MULTIVIEW", "Multi-View")],
    ),
];

/// `RENDER_PT_encoding` and `RENDER_PT_encoding_video`, for FFmpeg Video:
/// `FFMpegCodecData` (`DNA_scene_types.h`, `imbuf/movie/MOV_enums.hh`).
static CONTAINER: &[BEnumItem] = &[
    en("MPEG1", "MPEG-1"),
    en("MPEG2", "MPEG-2"),
    en("MPEG4", "MPEG-4"),
    en("AVI", "AVI"),
    en("QUICKTIME", "QuickTime"),
    en("DV", "DV"),
    en("OGG", "Ogg"),
    en("MKV", "Matroska"),
    en("FLASH", "Flash"),
    en("WEBM", "WebM"),
];
static CODEC: &[BEnumItem] = &[
    en("NONE", "No Video"),
    en("H264", "H.264"),
    en("H265", "H.265 / HEVC"),
    en("AV1", "AV1"),
    en("DNXHD", "DNxHD"),
    en("DV", "DV"),
    en("FFV1", "FFmpeg video codec #1"),
    en("FLASH", "Flash Video"),
    en("HUFFYUV", "HuffYUV"),
    en("MPEG1", "MPEG-1"),
    en("MPEG2", "MPEG-2"),
    en("MPEG4", "MPEG-4 (divx)"),
    en("PNG", "PNG"),
    en("QTRLE", "QT rle / QT Animation"),
    en("THEORA", "Theora"),
    en("WEBM", "WebM / VP9"),
    en("PRORES", "ProRes"),
];
static CRF: &[BEnumItem] = &[
    en("NONE", "Constant Bitrate"),
    en("LOSSLESS", "Lossless"),
    en("PERC_LOSSLESS", "Perceptually Lossless"),
    en("HIGH", "High Quality"),
    en("MEDIUM", "Medium Quality"),
    en("LOW", "Low Quality"),
    en("VERYLOW", "Very Low Quality"),
    en("LOWEST", "Lowest Quality"),
];
static PRESET: &[BEnumItem] = &[en("BEST", "Slowest"), en("GOOD", "Good"), en("REALTIME", "Realtime")];
static ENCODING: &[BPropSpec] = &[
    e("ffmpeg.format", "Container", CONTAINER),
    e("ffmpeg.codec", "Video Codec", CODEC),
    e("ffmpeg.constant_rate_factor", "Output Quality", CRF),
    e("ffmpeg.ffmpeg_preset", "Encoding Speed", PRESET),
    i("ffmpeg.gop_size", "Keyframe Interval", 0, 500),
    i("ffmpeg.video_bitrate", "Bitrate", 1, 14000),
];

static OUTPUT: &[BPropSpec] = &[
    s("filepath", "Output Path"),
    b("use_overwrite", "Overwrite"),
    b("use_placeholder", "Placeholders"),
    b("use_file_extension", "File Extensions"),
    b("use_render_cache", "Cache Result"),
    e("image_settings.file_format", "File Format", IMAGE_FORMAT),
    e("image_settings.color_mode", "Color", COLOR_MODE),
    e("image_settings.color_depth", "Color Depth", COLOR_DEPTH),
];

static METADATA: &[BPropSpec] = &[
    b("use_stamp", "Include Metadata in Image"),
    b("use_stamp_time", "Time"),
    b("use_stamp_date", "Date"),
    b("use_stamp_render_time", "Render Time"),
    b("use_stamp_frame", "Frame"),
    b("use_stamp_camera", "Camera"),
    b("use_stamp_lens", "Lens"),
    b("use_stamp_scene", "Scene"),
    b("use_stamp_note", "Note"),
    s("stamp_note_text", "Note Text"),
];

static POST_PROCESSING: &[BPropSpec] = &[
    b("use_compositing", "Compositing"),
    b("use_sequencer", "Sequencer"),
    b("dither_intensity", "Dither"),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Format", FORMAT),
    sec("Stereoscopy", STEREOSCOPY).closed(),
    sec("Output", OUTPUT),
    sec("Encoding", ENCODING).closed(),
    sec("Metadata", METADATA).closed(),
    sec("Post Processing", POST_PROCESSING).closed(),
];
