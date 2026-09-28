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
    b("use_border", "Border Render"),
    b("use_crop_to_border", "Crop to Border"),
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
    sec("Metadata", METADATA).closed(),
    sec("Post Processing", POST_PROCESSING).closed(),
];
