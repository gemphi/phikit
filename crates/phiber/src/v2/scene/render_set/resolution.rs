//! The render resolution the panel holds (split from `mod.rs`).

use super::*;

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
