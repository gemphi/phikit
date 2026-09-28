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

/// The render settings phiray honours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BRenderSet {
    /// `cycles.samples` — samples for a final render.
    pub samples: u32,
    /// `cycles.preview_samples` — samples the viewport refines to.
    pub preview_samples: u32,
    /// `cycles.diffuse_bounces` — indirect diffuse depth.
    pub diffuse_bounces: u32,
    /// `cycles.use_denoising`.
    pub denoise: bool,
}

impl Default for BRenderSet {
    fn default() -> Self {
        // Blender's Cycles defaults.
        Self { samples: 4096, preview_samples: 1024, diffuse_bounces: 4, denoise: true }
    }
}

/// The state stores every value as the string `BValue::encode` produced, so
/// reading one back is a parse in the property's own category.
fn value_at(s: &BState, key: &'static str, kind: BValueKind) -> Option<BValue> {
    BValue::parse(kind, s.get(ButsKey::Value(key))?)
}

fn int_at(s: &BState, key: &'static str, fallback: u32) -> u32 {
    match value_at(s, key, BValueKind::Int) {
        Some(BValue::Int(v)) => v.clamp(0, i64::from(u32::MAX)) as u32,
        _ => fallback,
    }
}

fn bool_at(s: &BState, key: &'static str, fallback: bool) -> bool {
    match value_at(s, key, BValueKind::Bool) {
        Some(BValue::Bool(v)) => v,
        _ => fallback,
    }
}

/// Read the Render panel.
pub fn render_set(s: &BState) -> BRenderSet {
    let d = BRenderSet::default();
    BRenderSet {
        samples: int_at(s, "cycles.samples", d.samples),
        preview_samples: int_at(s, "cycles.preview_samples", d.preview_samples),
        diffuse_bounces: int_at(s, "cycles.diffuse_bounces", d.diffuse_bounces),
        denoise: bool_at(s, "cycles.use_denoising", d.denoise),
    }
}
