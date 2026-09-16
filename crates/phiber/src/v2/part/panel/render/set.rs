//! Render context set — Cycles + EEVEE common sections and RNA properties.

use super::super::prop::*;
use super::super::set::{BSectionSpec, sec};

static ENGINE: &[BEnumItem] = &[
    en("BLENDER_EEVEE_NEXT", "EEVEE"),
    en("BLENDER_WORKBENCH", "Workbench"),
    en("CYCLES", "Cycles"),
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

static SAMPLING: &[BPropSpec] = &[
    e("engine", "Render Engine", ENGINE),
    e("cycles.device", "Device", DEVICE),
    i("cycles.samples", "Render Samples", 0, 100000),
    i("cycles.preview_samples", "Viewport Samples", 0, 100000),
    b("cycles.use_adaptive_sampling", "Adaptive Sampling"),
    f("cycles.adaptive_threshold", "Noise Threshold", BUnit::None),
    i("cycles.adaptive_min_samples", "Min Samples", 0, 100000),
    f("cycles.time_limit", "Time Limit", BUnit::Time),
    b("cycles.use_denoising", "Denoise"),
    e("cycles.denoiser", "Denoiser", DENOISER),
];

static LIGHT_PATHS: &[BPropSpec] = &[
    i("cycles.max_bounces", "Total Max Bounces", 0, 1024),
    i("cycles.diffuse_bounces", "Diffuse", 0, 1024),
    i("cycles.glossy_bounces", "Glossy", 0, 1024),
    i("cycles.transmission_bounces", "Transmission", 0, 1024),
    i("cycles.volume_bounces", "Volume", 0, 1024),
    i("cycles.transparent_max_bounces", "Transparent", 0, 1024),
    b("cycles.caustics_reflective", "Reflective Caustics"),
    b("cycles.caustics_refractive", "Refractive Caustics"),
    f("cycles.blur_glossy", "Filter Glossy", BUnit::None),
];

static VOLUMES: &[BPropSpec] = &[
    f("cycles.volume_step_rate", "Step Rate Render", BUnit::None),
    f(
        "cycles.volume_preview_step_rate",
        "Step Rate Viewport",
        BUnit::None,
    ),
    i("cycles.volume_max_steps", "Max Steps", 1, 65536),
];

static MOTION_BLUR: &[BPropSpec] = &[
    b("render.use_motion_blur", "Motion Blur"),
    f("render.motion_blur_shutter", "Shutter", BUnit::None),
    e("render.motion_blur_position", "Position", MOTION_BLUR_POS),
];

static FILM: &[BPropSpec] = &[
    f("render.filter_size", "Pixel Filter Width", BUnit::None),
    b("render.film_transparent", "Transparent"),
    e("cycles.pixel_filter_type", "Type", PIXEL_FILTER),
];

static PERFORMANCE: &[BPropSpec] = &[
    i("render.threads", "Threads", 1, 1024),
    e("render.threads_mode", "Mode", THREADS_MODE),
    b("render.use_persistent_data", "Persistent Data"),
];

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

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Sampling", SAMPLING),
    sec("Light Paths", LIGHT_PATHS),
    sec("Volumes", VOLUMES),
    sec("Motion Blur", MOTION_BLUR),
    sec("Film", FILM),
    sec("Performance", PERFORMANCE),
    sec("Curves", CURVES),
    sec("Color Management", COLOR_MANAGEMENT),
];
