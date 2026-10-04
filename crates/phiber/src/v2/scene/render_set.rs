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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRenderSet {
    /// `cycles.samples` — samples for a final render.
    pub samples: u32,
    /// `cycles.preview_samples` — samples the viewport refines to.
    pub preview_samples: u32,
    /// `cycles.diffuse_bounces` — indirect diffuse depth.
    pub diffuse_bounces: u32,
    /// `cycles.use_denoising`.
    pub denoise: bool,
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
}

impl Default for BRenderSet {
    fn default() -> Self {
        // Blender's Cycles defaults — but Viewport Denoising on, Phiber's
        // own default (`part/props/render.rs`).
        Self {
            samples: 4096,
            preview_samples: 1024,
            diffuse_bounces: 4,
            denoise: true,
            preview_denoise: true,
            preview_denoise_from: 1,
            resolution: BResolution::default(),
            film_transparent: false,
            border: Default::default(),
        }
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
        preview_denoise: bool_at(s, "cycles.use_preview_denoising", d.preview_denoise),
        preview_denoise_from: int_at(s, "cycles.preview_denoising_start_sample", d.preview_denoise_from),
        resolution: BResolution::now(s),
        film_transparent: bool_at(s, "render.film_transparent", d.film_transparent),
        border: super::render_border::BRenderBorder::now(s, super::source::with_loaded(|m| m.map(|m| m.render.border).unwrap_or_default())),
    }
}

/// Output ▸ Format ▸ Resolution X, Y and % as the panel holds them: seeded
/// from the file when it opens (`app::seed`), then whatever is typed there —
/// F12, the camera's frame in the viewport and a save all take it, as
/// Blender's `RenderData.xsch / ysch / size` are one set of fields. A value
/// the panel has never held is the file's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BResolution {
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub percent: Option<u32>,
}

thread_local! {
    /// The panel's resolution for the readers with no state in hand — the
    /// camera's frame and its overlay shape — set with the loaded model each
    /// render pass (`source::set_loaded_at`).
    static SCOPED: std::cell::Cell<BResolution> = const { std::cell::Cell::new(BResolution { x: None, y: None, percent: None }) };
}

impl BResolution {
    pub fn now(s: &BState) -> Self {
        let at = |key| match value_at(s, key, BValueKind::Int) {
            Some(BValue::Int(v)) => Some(v.clamp(1, 65536) as u32),
            _ => None,
        };
        Self { x: at("resolution_x"), y: at("resolution_y"), percent: at("resolution_percentage") }
    }

    /// Resolution X and Y, before the percentage.
    pub fn base(&self, model: &crate::phiphx::scene::SceneModel) -> (u32, u32) {
        let r = &model.render;
        (self.x.unwrap_or(r.resolution_x.max(1) as u32).max(1), self.y.unwrap_or(r.resolution_y.max(1) as u32).max(1))
    }

    /// The pixels F12 renders: the base scaled by the percentage, never zero.
    pub fn pixels(&self, model: &crate::phiphx::scene::SceneModel) -> (u32, u32) {
        let pct = f64::from(self.percent.unwrap_or(model.render.resolution_percent.max(1) as u32)) / 100.0;
        let (x, y) = self.base(model);
        let side = |n: u32| ((f64::from(n) * pct).round() as u32).max(1);
        (side(x), side(y))
    }

    /// What a save writes (`Scene.r.xsch / ysch / size`): the three, when
    /// any differs from the file's.
    pub fn edited(&self, model: &crate::phiphx::scene::SceneModel) -> Option<[i32; 3]> {
        let ((x, y), r) = (self.base(model), &model.render);
        let pct = self.percent.unwrap_or(r.resolution_percent.max(1) as u32);
        let now = [x as i32, y as i32, pct as i32];
        (now != [r.resolution_x, r.resolution_y, r.resolution_percent]).then_some(now)
    }

    pub fn scope(s: &BState) {
        SCOPED.with(|c| c.set(Self::now(s)));
    }

    pub fn scoped() -> Self {
        SCOPED.with(std::cell::Cell::get)
    }
}

/// Render ▸ Color Management's View Transform, Exposure and Gamma
/// (`Scene.view_settings`) as the panel holds them — seeded from the file
/// when it opens (`app::seed`), then whatever is chosen there — over the
/// file's. The viewport and F12 both look through them, as every Blender
/// view does, and a save writes them.
#[derive(Clone, Debug, PartialEq)]
pub struct BColorManagement {
    pub transform: String,
    pub exposure: f32,
    pub gamma: f32,
}

impl BColorManagement {
    pub fn now(s: &BState, model: Option<&crate::phiphx::scene::SceneModel>) -> Self {
        let file = model.map_or(Self { transform: "Standard".into(), exposure: 0.0, gamma: 1.0 }, |m| Self::of_file(m));
        let float = |key, d: f32| match value_at(s, key, BValueKind::Float) {
            Some(BValue::Float(v)) => v,
            _ => d,
        };
        let transform = s.get(ButsKey::Value("view_settings.view_transform")).filter(|t| !t.is_empty()).map_or(file.transform, str::to_string);
        Self { transform, exposure: float("view_settings.exposure", file.exposure), gamma: float("view_settings.gamma", file.gamma) }
    }

    pub fn of_file(model: &crate::phiphx::scene::SceneModel) -> Self {
        Self { transform: model.render.view_transform.clone(), exposure: model.render.exposure, gamma: model.render.gamma }
    }

    /// `Camera::view`: what the tracer's view kernel applies.
    pub fn words(&self) -> [f32; 4] {
        phiray::View::from_blender(&self.transform).words(self.exposure, self.gamma)
    }

    /// What a save writes, when it differs from the file's.
    pub fn edited(&self, model: &crate::phiphx::scene::SceneModel) -> Option<(String, f32, f32)> {
        (*self != Self::of_file(model)).then(|| (self.transform.clone(), self.exposure, self.gamma))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_s_resolution_over_the_file_s() {
        let mut model = crate::phiphx::scene::SceneModel::default();
        (model.render.resolution_x, model.render.resolution_y, model.render.resolution_percent) = (1920, 1080, 50);
        let mut s = BState::new();
        assert_eq!(BResolution::now(&s).pixels(&model), (960, 540), "the file's until the panel holds one");
        assert_eq!(BResolution::now(&s).edited(&model), None);
        s.set(ButsKey::Value("resolution_y"), BValue::Int(1920).encode());
        s.set(ButsKey::Value("resolution_percentage"), BValue::Int(100).encode());
        let now = BResolution::now(&s);
        assert_eq!((now.base(&model), now.pixels(&model)), ((1920, 1920), (1920, 1920)));
        assert_eq!(now.edited(&model), Some([1920, 1920, 100]));
        assert_eq!(render_set(&s).resolution, now);
    }
}
