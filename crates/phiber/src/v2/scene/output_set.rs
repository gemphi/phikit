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
        // A format the panel lists but phiber does not write (FFmpeg and the
        // AVI pair). Keeping the default here rather than inventing an
        // imtype means a save still produces something rather than failing.
        _ => BlendImageFormat::Png,
    }
}

/// Read the Output panel. Anything the state does not hold keeps the value a
/// fresh `.blend` would have.
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
