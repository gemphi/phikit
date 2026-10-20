//! Render ▸ Color Management as the panel holds it.
//!
//! Split out of `render_set.rs`: the view transform, exposure and gamma are
//! one set of fields that the viewport, F12 and a save all read, so they
//! own a file rather than riding along with the sampling rows.

use super::render_set::value_at;
use crate::v2::state::{
    BState, ButsKey,
    val::{BValue, BValueKind},
};

/// Render ▸ Color Management's View Transform, Exposure and Gamma
/// (`Scene.view_settings`) as the panel holds them — seeded from the file
/// when it opens (`app::seed`), then whatever is chosen there — over the
/// file's. The viewport and F12 both look through them, as every Blender
/// view does, and a save writes them.
#[derive(Clone, Debug, PartialEq)]
pub struct BColorManagement {
    pub transform: String,
    /// `view_settings.look` ("None", "Very High Contrast", …).
    pub look: String,
    pub exposure: f32,
    pub gamma: f32,
}

impl BColorManagement {
    pub fn now(s: &BState, model: Option<&crate::phiphx::scene::SceneModel>) -> Self {
        let file = model.map_or(Self { transform: "Standard".into(), look: "None".into(), exposure: 0.0, gamma: 1.0 }, |m| Self::of_file(m));
        let float = |key, d: f32| match value_at(s, key, BValueKind::Float) {
            Some(BValue::Float(v)) => v,
            _ => d,
        };
        let transform = s.get(ButsKey::Value("view_settings.view_transform")).filter(|t| !t.is_empty()).map_or(file.transform, str::to_string);
        let look = s.get(ButsKey::Value("view_settings.look")).filter(|t| !t.is_empty()).map_or(file.look, str::to_string);
        Self { transform, look, exposure: float("view_settings.exposure", file.exposure), gamma: float("view_settings.gamma", file.gamma) }
    }

    pub fn of_file(model: &crate::phiphx::scene::SceneModel) -> Self {
        Self { transform: model.render.view_transform.clone(), look: model.render.look.clone(), exposure: model.render.exposure, gamma: model.render.gamma }
    }

    /// `Camera::view`: what the tracer's view kernel applies.
    pub fn words(&self) -> [f32; 4] {
        phiray::View::from_blender(&self.transform).words_with(self.exposure, self.gamma, phiray::Look::from_blender(&self.look))
    }

    /// What a save writes, when it differs from the file's.
    pub fn edited(&self, model: &crate::phiphx::scene::SceneModel) -> Option<(String, f32, f32)> {
        (*self != Self::of_file(model)).then(|| (self.transform.clone(), self.exposure, self.gamma))
    }
}
