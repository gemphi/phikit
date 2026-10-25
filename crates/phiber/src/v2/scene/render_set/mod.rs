//! What the Properties → Render panel asks the renderer for.
//!
//! The panel's rows are RNA properties stored at `ctx.buts.<key>`
//! (`part/props/render.rs`, `state/key/buts.rs`). This reads the handful that
//! phiray can act on and hands them over as one value, so the panel stops
//! being decoration: changing Max Samples, Diffuse bounces or Denoise in the
//! UI changes the next frame.
//!
//! Values not in the state fall back to the property's RNA default, which is
//! Blender's — so a fresh session behaves like a fresh `.blend` rather than
//! like zero.

use crate::v2::state::{
    BState, ButsKey,
    val::{BValue, BValueKind},
};

/// Resolving the panel's adaptive rows lives next door; it stayed
/// reachable here because that is where callers already look for it.
pub use super::render_adaptive::BAdaptive;

/// Colour management moved to its own file for the same reason; callers
/// still reach it through here.
pub use super::render_color::BColorManagement;

/// The render settings phiray honours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRenderSet {
    /// `cycles.samples` — samples for a final render.
    pub samples: u32,
    /// `cycles.preview_samples` — samples the viewport refines to.
    pub preview_samples: u32,
    /// `cycles.diffuse_bounces` — indirect diffuse depth.
    pub diffuse_bounces: u32,
    /// Light Paths ▸ Max Bounces' other rows: `cycles.max_bounces`
    /// (Total), `transparent_max_bounces`, `transmission_bounces` and
    /// `glossy_bounces` (`phiray::Camera::with_depth`).
    pub max_bounces: u32,
    pub transparent_bounces: u32,
    pub transmission_bounces: u32,
    pub glossy_bounces: u32,
    /// `cycles.use_denoising`.
    pub denoise: bool,
    /// `cycles.use_adaptive_sampling`, `cycles.adaptive_threshold` and
    /// `cycles.adaptive_min_samples` as the file holds them — **before**
    /// Cycles' automatic rules are applied to them. `BAdaptive::of`
    /// resolves them; these three are the raw rows, so a `0` here still
    /// means "automatic", as the panel shows it.
    pub adaptive: bool,
    pub adaptive_threshold: f32,
    pub adaptive_min_samples: u32,
    /// `cycles.time_limit`: seconds a final render may take, `0` for no
    /// limit. Cycles (`session.cpp`, `Session::run_main_render_loop`)
    /// stops after the sample batch that crosses it, whatever Max Samples
    /// says — "give it twenty minutes" rather than a count. It was read
    /// and saved and never applied, so 4096 samples at seven seconds each
    /// had no way to be told to stop.
    pub time_limit: f32,
    /// Light Paths ▸ Clamping: the most one sample's direct and indirect
    /// light may add, `0` for no limit (`phiray::Camera::limit`).
    pub clamp_direct: f32,
    pub clamp_indirect: f32,
    /// `cycles.use_preview_denoising` and `preview_denoising_start_sample`:
    /// the viewport shown filtered while it refines, from that sample on.
    pub preview_denoise: bool,
    pub preview_denoise_from: u32,
    /// Output ▸ Format's resolution, where the panel holds one.
    pub resolution: BResolution,
    /// Film ▸ Transparent (`render.film_transparent`): a final render's
    /// background cut away to alpha 0 (`phiray::Camera::film`).
    pub film_transparent: bool,
    /// Output ▸ Format's Render Region and Crop to Render Region with the
    /// rectangle set (`scene::render_border`).
    pub border: crate::phiphx::blend::BlendRenderBorder,
    /// Film ▸ Pixel Filter: `cycles.pixel_filter_type` and
    /// `cycles.filter_width` in pixels (`phiray::Camera::integrator`) —
    /// Box always one pixel, as `blender/sync.cpp` sets it.
    pub pixel_filter: phiray::PixelFilter,
    pub filter_size: f32,
    /// Light Paths ▸ Caustics: `cycles.blur_glossy` (Filter Glossy),
    /// `caustics_reflective` and `caustics_refractive`.
    pub filter_glossy: f32,
    pub caustics_reflective: bool,
    pub caustics_refractive: bool,
    /// Render ▸ Engine as the panel holds it, the file's when the panel has
    /// never held one (`scene::render_engine`). Workbench rasterizes; every
    /// other engine, `PHIRAY` among them, path-traces.
    pub engine: super::render_engine::BRenderEngine,
}

impl Default for BRenderSet {
    fn default() -> Self {
        // Blender's Cycles defaults — but Viewport Denoising on, Phiber's
        // own default (`part/props/render.rs`).
        Self {
            samples: 4096,
            preview_samples: 1024,
            diffuse_bounces: 4,
            max_bounces: 12,
            transparent_bounces: 8,
            transmission_bounces: 12,
            glossy_bounces: 4,
            denoise: true,
            // Cycles' own defaults (`scene/integrator.cpp`):
            //   SOCKET_BOOLEAN(use_adaptive_sampling, true)
            //   SOCKET_FLOAT(adaptive_threshold, 0.01f)
            //   SOCKET_INT(adaptive_min_samples, 0)
            adaptive: true,
            adaptive_threshold: 0.01,
            adaptive_min_samples: 0,
            //   SOCKET_FLOAT(time_limit, 0.0f)
            time_limit: 0.0,
            //   SOCKET_FLOAT(sample_clamp_direct, 0.0f)
            //   SOCKET_FLOAT(sample_clamp_indirect, 10.0f)
            clamp_direct: 0.0,
            clamp_indirect: 10.0,
            preview_denoise: true,
            preview_denoise_from: 1,
            resolution: BResolution::default(),
            film_transparent: false,
            border: Default::default(),
            //   SOCKET_ENUM(filter_type, …, FILTER_BLACKMAN_HARRIS) and
            //   `RenderData.gauss` 1.5 (`DNA_scene_defaults.h`); Filter
            //   Glossy 1 and both caustics on (`addon/properties.py`).
            pixel_filter: phiray::PixelFilter::BlackmanHarris,
            filter_size: 1.5,
            filter_glossy: 1.0,
            caustics_reflective: true,
            caustics_refractive: true,
            engine: Default::default(),
        }
    }
}

/// The state stores every value as the string `BValue::encode` produced, so
/// reading one back is a parse in the property's own category.
pub(super) fn value_at(s: &BState, key: &'static str, kind: BValueKind) -> Option<BValue> {
    BValue::parse(kind, s.get(ButsKey::Value(key))?)
}

fn int_at(s: &BState, key: &'static str, fallback: u32) -> u32 {
    match value_at(s, key, BValueKind::Int) {
        Some(BValue::Int(v)) => v.clamp(0, i64::from(u32::MAX)) as u32,
        _ => fallback,
    }
}

fn float_at(s: &BState, key: &'static str, fallback: f32) -> f32 {
    match value_at(s, key, BValueKind::Float) {
        Some(BValue::Float(v)) => v,
        _ => fallback,
    }
}

fn bool_at(s: &BState, key: &'static str, fallback: bool) -> bool {
    match value_at(s, key, BValueKind::Bool) {
        Some(BValue::Bool(v)) => v,
        _ => fallback,
    }
}

/// Render ▸ Engine: the panel's row, and — until the panel holds one —
/// the engine phiber actually has for this file
/// (`BRenderQuality::engine_for_file`). A `.blend` names EEVEE or Cycles
/// and phiber runs neither, so the file's name is a preference and the
/// tracer is what renders it; `BLENDER_WORKBENCH` is kept, because
/// Workbench is an engine phiber really does have.
fn engine_at(s: &BState) -> super::render_engine::BRenderEngine {
    use super::render_engine::BRenderEngine;
    use super::render_quality::BRenderQuality;
    match s.get(ButsKey::Value("engine")).filter(|e| !e.is_empty()) {
        Some(chosen) => BRenderEngine::of(chosen),
        None => super::source::with_loaded(|m| m.map_or_else(BRenderEngine::default, |m| BRenderQuality::engine_for_file(&m.render.engine))),
    }
}

/// Read the Render panel.
pub fn render_set(s: &BState) -> BRenderSet {
    let d = BRenderSet::default();
    BRenderSet {
        samples: int_at(s, "cycles.samples", d.samples),
        preview_samples: int_at(s, "cycles.preview_samples", d.preview_samples),
        diffuse_bounces: int_at(s, "cycles.diffuse_bounces", d.diffuse_bounces),
        max_bounces: int_at(s, "cycles.max_bounces", d.max_bounces),
        transparent_bounces: int_at(s, "cycles.transparent_max_bounces", d.transparent_bounces),
        transmission_bounces: int_at(s, "cycles.transmission_bounces", d.transmission_bounces),
        glossy_bounces: int_at(s, "cycles.glossy_bounces", d.glossy_bounces),
        denoise: bool_at(s, "cycles.use_denoising", d.denoise),
        adaptive: bool_at(s, "cycles.use_adaptive_sampling", d.adaptive),
        adaptive_threshold: float_at(s, "cycles.adaptive_threshold", d.adaptive_threshold),
        adaptive_min_samples: int_at(s, "cycles.adaptive_min_samples", d.adaptive_min_samples),
        time_limit: float_at(s, "cycles.time_limit", d.time_limit).max(0.0),
        clamp_direct: float_at(s, "cycles.sample_clamp_direct", d.clamp_direct).max(0.0),
        clamp_indirect: float_at(s, "cycles.sample_clamp_indirect", d.clamp_indirect).max(0.0),
        preview_denoise: bool_at(s, "cycles.use_preview_denoising", d.preview_denoise),
        preview_denoise_from: int_at(s, "cycles.preview_denoising_start_sample", d.preview_denoise_from),
        resolution: BResolution::now(s),
        film_transparent: bool_at(s, "render.film_transparent", d.film_transparent),
        border: super::render_border::BRenderBorder::now(s, super::source::with_loaded(|m| m.map(|m| m.render.border).unwrap_or_default())),
        pixel_filter: match value_at(s, "cycles.pixel_filter_type", BValueKind::Enum) {
            Some(BValue::Enum(name)) => phiray::PixelFilter::of(&name),
            _ => d.pixel_filter,
        },
        filter_size: float_at(s, "cycles.filter_width", d.filter_size).clamp(0.01, 10.0),
        filter_glossy: float_at(s, "cycles.blur_glossy", d.filter_glossy).max(0.0),
        caustics_reflective: bool_at(s, "cycles.caustics_reflective", d.caustics_reflective),
        caustics_refractive: bool_at(s, "cycles.caustics_refractive", d.caustics_refractive),
        // The panel's row, and the file's engine only until the panel holds
        // one — which is how every other row in this struct reads.
        engine: engine_at(s),
    }
}

pub use resolution::BResolution;
mod resolution;

#[cfg(test)]
mod tests;
