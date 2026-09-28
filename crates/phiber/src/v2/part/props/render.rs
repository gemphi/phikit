//! Render context — Cycles + EEVEE common sections.
//! Blender ref: `bl_ui/properties_render.py`, `rna_scene.cc` RenderSettings,
//! `intern/cycles/blender/*.cpp` for Cycles-specific properties.
// Render context set — Cycles + EEVEE common sections and RNA properties.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

static ENGINE: &[BEnumItem] = &[
    en("BLENDER_EEVEE_NEXT", "EEVEE"),
    en("BLENDER_WORKBENCH", "Workbench"),
    en("CYCLES", "Cycles"),
    // Phiber's own path tracer. A file saved by Blender never names it, so it
    // is only ever chosen here — which is why it sits after the three engines
    // a `.blend` can actually carry.
    en("PHIRAY", "Phiray"),
];
static DEVICE: &[BEnumItem] = &[en("CPU", "CPU"), en("GPU", "GPU Compute")];
static DENOISER: &[BEnumItem] = &[
    en("OPTIX", "OptiX"),
    en("OPENIMAGEDENOISE", "OpenImageDenoise"),
];
static MOTION_BLUR_POS: &[BEnumItem] = &[
    en("START", "Start on Frame"),
    en("CENTER", "Center on Frame"),
    en("END", "End on Frame"),
];
static PIXEL_FILTER: &[BEnumItem] = &[
    en("BOX", "Box"),
    en("GAUSSIAN", "Gaussian"),
    en("BLACKMAN_HARRIS", "Blackman-Harris"),
];
static CURVE_SHAPE: &[BEnumItem] = &[en("RIBBONS", "Rounded Ribbons"), en("THICK", "3D Curves")];
static THREADS_MODE: &[BEnumItem] = &[en("AUTO", "Auto-Detect"), en("FIXED", "Fixed")];
static VIEW_TRANSFORM: &[BEnumItem] = &[
    en("Standard", "Standard"),
    en("AgX", "AgX"),
    en("Filmic", "Filmic"),
    en("Khronos PBR Neutral", "Khronos PBR Neutral"),
    en("Raw", "Raw"),
    en("False Color", "False Color"),
];
static LOOK: &[BEnumItem] = &[
    en("None", "None"),
    en("Very Low Contrast", "Very Low Contrast"),
    en("Low Contrast", "Low"),
    en("Medium Contrast", "Medium"),
    en("High Contrast", "High"),
    en("Very High Contrast", "Very High"),
];

// `RENDER_PT_context` (`properties_render.py`) — a `HIDE_HEADER` panel at the
// head of the tab: the engine, and Cycles' device beside it.
static CONTEXT: &[BPropSpec] = &[
    e("engine", "Render Engine", ENGINE),
    e("cycles.device", "Device", DEVICE),
];

// `CYCLES_RENDER_PT_sampling` has no rows of its own; its sub-panels do.
static SAMPLING_VIEWPORT: &[BPropSpec] = &[i("cycles.preview_samples", "Max Samples", 0, 100000).at(1024)];
static SAMPLING_RENDER: &[BPropSpec] = &[
    b("cycles.use_adaptive_sampling", "Adaptive Sampling"),
    f("cycles.adaptive_threshold", "Noise Threshold", BUnit::None),
    i("cycles.samples", "Max Samples", 0, 100000).at(4096),
    i("cycles.adaptive_min_samples", "Min Samples", 0, 100000),
    f("cycles.time_limit", "Time Limit", BUnit::Time),
];
// `CYCLES_RENDER_PT_sampling_render_denoise`: the toggle is its header.
static DENOISE_HEADER: &[BPropSpec] = &[b("cycles.use_denoising", "Denoise").on()];
static DENOISE: &[BPropSpec] = &[e("cycles.denoiser", "Denoiser", DENOISER)];
static SAMPLING_RENDER_SUBS: &[BSectionSpec] = &[sec("Denoise", DENOISE).closed().header(DENOISE_HEADER)];
static SAMPLING_SUBS: &[BSectionSpec] = &[
    sec("Viewport", SAMPLING_VIEWPORT),
    sec("Render", SAMPLING_RENDER).subs(SAMPLING_RENDER_SUBS),
];

// `CYCLES_RENDER_PT_light_paths` — rows live in its sub-panels.
static MAX_BOUNCES: &[BPropSpec] = &[
    i("cycles.max_bounces", "Total", 0, 1024),
    i("cycles.diffuse_bounces", "Diffuse", 0, 1024).at(4),
    i("cycles.glossy_bounces", "Glossy", 0, 1024),
    i("cycles.transmission_bounces", "Transmission", 0, 1024),
    i("cycles.volume_bounces", "Volume", 0, 1024),
    i("cycles.transparent_max_bounces", "Transparent", 0, 1024),
];
static CAUSTICS: &[BPropSpec] = &[
    f("cycles.blur_glossy", "Filter Glossy", BUnit::None),
    b("cycles.caustics_reflective", "Reflective"),
    b("cycles.caustics_refractive", "Refractive"),
];
static LIGHT_PATHS_SUBS: &[BSectionSpec] = &[sec("Max Bounces", MAX_BOUNCES), sec("Caustics", CAUSTICS)];

static VOLUMES: &[BPropSpec] = &[
    f("cycles.volume_step_rate", "Step Rate Render", BUnit::None),
    f(
        "cycles.volume_preview_step_rate",
        "Step Rate Viewport",
        BUnit::None,
    ),
    i("cycles.volume_max_steps", "Max Steps", 1, 65536),
];

// `CYCLES_RENDER_PT_motion_blur`: enabled from its header.
static MOTION_BLUR_HEADER: &[BPropSpec] = &[b("render.use_motion_blur", "Motion Blur")];
static MOTION_BLUR: &[BPropSpec] = &[
    e("render.motion_blur_position", "Position", MOTION_BLUR_POS),
    f("render.motion_blur_shutter", "Shutter", BUnit::None),
];

// `CYCLES_RENDER_PT_film` and its sub-panels: Transparent is enabled from its
// header; Pixel Filter holds the type and width.
static FILM_TRANSPARENT_HEADER: &[BPropSpec] = &[b("render.film_transparent", "Transparent")];
static PIXEL_FILTER_ROWS: &[BPropSpec] = &[
    e("cycles.pixel_filter_type", "Type", PIXEL_FILTER),
    f("render.filter_size", "Width", BUnit::None),
];
static FILM_SUBS: &[BSectionSpec] = &[
    sec("Transparent", &[]).header(FILM_TRANSPARENT_HEADER),
    sec("Pixel Filter", PIXEL_FILTER_ROWS),
];

// `CYCLES_RENDER_PT_performance`: Threads and Final Render sub-panels.
static THREADS: &[BPropSpec] = &[
    e("render.threads_mode", "Threads Mode", THREADS_MODE),
    i("render.threads", "Threads", 1, 1024),
];
static FINAL_RENDER: &[BPropSpec] = &[b("render.use_persistent_data", "Persistent Data")];
static PERFORMANCE_SUBS: &[BSectionSpec] = &[sec("Threads", THREADS), sec("Final Render", FINAL_RENDER)];

static CURVES: &[BPropSpec] = &[
    e("cycles_curves.shape", "Shape", CURVE_SHAPE),
    i("cycles_curves.subdivisions", "Curve Subdivisions", 0, 24),
];

static COLOR_MANAGEMENT: &[BPropSpec] = &[
    e(
        "view_settings.view_transform",
        "View Transform",
        VIEW_TRANSFORM,
    ),
    e("view_settings.look", "Look", LOOK),
    f("view_settings.exposure", "Exposure", BUnit::None),
    f("view_settings.gamma", "Gamma", BUnit::None),
];

/// The Render tab, as Blender with Cycles draws it: panel order, which
/// panels start closed (`DEFAULT_CLOSED`) and each panel's sub-panels
/// (`bl_parent_id`), from `properties_render.py` and Cycles' `addon/ui.py`.
pub static SECTIONS: &[BSectionSpec] = &[
    sec("Context", CONTEXT).headless(),
    sec("Sampling", &[]).subs(SAMPLING_SUBS),
    sec("Light Paths", &[]).closed().subs(LIGHT_PATHS_SUBS),
    sec("Volumes", VOLUMES).closed(),
    sec("Motion Blur", MOTION_BLUR).closed().header(MOTION_BLUR_HEADER),
    sec("Film", &[]).closed().subs(FILM_SUBS),
    sec("Performance", &[]).closed().subs(PERFORMANCE_SUBS),
    sec("Curves", CURVES).closed(),
    sec("Color Management", COLOR_MANAGEMENT).closed(),
];

/// The Render tab's own spec for `key`, if the tab draws it at all.
///
/// Two things need it. A file stores its Cycles settings as ID properties,
/// whose types are the add-on's, not the panel's — a flag can arrive as an
/// integer — so seeding has to write the value in the category the *control*
/// reads (`state/val.rs`). And the key it writes under has to be the panel's
/// own `'static` string, so the state key can be a `ButsKey` rather than a
/// path spelt by hand.
pub fn spec_of(key: &str) -> Option<&'static BPropSpec> {
    fn walk(sections: &'static [BSectionSpec], key: &str) -> Option<&'static BPropSpec> {
        for section in sections {
            if let Some(found) = section.props.iter().chain(section.header).find(|s| s.key == key) {
                return Some(found);
            }
            if let Some(found) = walk(section.list, key) {
                return Some(found);
            }
        }
        None
    }
    walk(SECTIONS, key)
}
