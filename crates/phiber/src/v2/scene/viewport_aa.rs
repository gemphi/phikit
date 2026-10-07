//! Viewport Anti-Aliasing (`U.viewport_aa`, Preferences ▸ Viewport ▸
//! Quality; `rna_enum_userdef_viewport_aa_items`): how many jittered
//! samples Workbench draws a Solid or Wireframe view with
//! (`SceneState::samples_len`) — none, a single pass with SMAA alone, or
//! 5, 8, 11, 16 or 32 — the default 8. Blender takes it from the user
//! preferences for Solid and Wireframe, from `Scene.display.viewport_aa`
//! only when a Rendered or Material view runs on the Workbench engine, and
//! `render_aa` for a render (`BStillLook::for_engine`). Session state on
//! `ctx.pref.viewport_aa` (`AppKey::ViewportAa`), as the render device is.

use crate::v2::state::val::BValue;
use crate::v2::state::{AppKey, BKey as _, BPath, BState};
use crate::v2::view::prim::{BCtrl, BCtrlKind, BOption};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BViewportAa;

impl BViewportAa {
    /// Blender's items: the RNA identifier, its name, and the count
    /// (`eSceneDisplay_AA`'s value).
    pub const ITEMS: [(&'static str, &'static str, u32); 7] = [
        ("OFF", "No Anti-Aliasing", 0),
        ("FXAA", "Single Pass Anti-Aliasing", 1),
        ("5", "5 Samples", 5),
        ("8", "8 Samples", 8),
        ("11", "11 Samples", 11),
        ("16", "16 Samples", 16),
        ("32", "32 Samples", 32),
    ];
    pub const DEFAULT: &'static str = "8";

    pub fn key() -> BPath {
        AppKey::ViewportAa.path()
    }

    /// The identifier chosen; an unknown or unset value is the default.
    pub fn chosen(s: &BState) -> &'static str {
        let v = s.get(AppKey::ViewportAa).unwrap_or(Self::DEFAULT);
        Self::ITEMS.iter().find(|(id, ..)| *id == v).map_or(Self::DEFAULT, |(id, ..)| id)
    }

    /// The sample count the viewport's Solid and Wireframe draw with
    /// (`Tracer::set_aa_samples`): 0 off, 1 the single pass.
    pub fn samples(s: &BState) -> u32 {
        let chosen = Self::chosen(s);
        Self::ITEMS.iter().find(|(id, ..)| *id == chosen).map_or(8, |(.., n)| *n)
    }

    /// The CPU rasteriser's supersample factor for the chosen AA — the Solid
    /// and Wireframe views have no MSAA, so they oversample the whole frame and
    /// average it down (`view3d::mesh::paint`). Blender's jittered sample COUNTS
    /// map onto a supersample grid (f² samples a pixel): OFF and the single FXAA
    /// pass draw at 1x (the CPU path has no SMAA post to stand in for FXAA),
    /// then 2x, 3x, 4x. 4x (sixteen samples) is the ceiling, so 32 does not turn
    /// one frame into a thousand-sample CPU rasterise.
    pub fn supersample(s: &BState) -> usize {
        match Self::samples(s) {
            0 | 1 => 1,
            5 => 2,
            8 | 11 => 3,
            _ => 4,
        }
    }

    /// The Quality card's select over the preference.
    pub fn control(open: BPath, s: &BState) -> BCtrl {
        let options = Self::ITEMS.iter().map(|(id, label, _)| BOption { key: id.to_string(), label: label.to_string(), icon: String::new() });
        BCtrl::read(BCtrlKind::Select, Self::key(), BValue::Enum(Self::DEFAULT.to_string()), s).with_label("Viewport Anti-Aliasing").with_options(options).with_menu(open, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_preference_is_blenders_items_and_eight_until_set() {
        let mut s = BState::new();
        assert_eq!((BViewportAa::chosen(&s), BViewportAa::samples(&s)), ("8", 8));
        s.set(AppKey::ViewportAa, "FXAA");
        assert_eq!(BViewportAa::samples(&s), 1, "the single pass: one sample, SMAA over it");
        s.set(AppKey::ViewportAa, "OFF");
        assert_eq!(BViewportAa::samples(&s), 0);
        s.set(AppKey::ViewportAa, "32");
        assert_eq!(BViewportAa::samples(&s), 32);
        s.set(AppKey::ViewportAa, "7");
        assert_eq!((BViewportAa::chosen(&s), BViewportAa::samples(&s)), ("8", 8), "not an item: the default");
        let ctrl = BViewportAa::control(BPath::owned(vec!["x"]), &s);
        assert_eq!(ctrl.options.len(), 7);
        assert_eq!(ctrl.options[1].label, "Single Pass Anti-Aliasing");
        assert_eq!(BViewportAa::key().encode(), "ctx.pref.viewport_aa");
    }

    #[test]
    fn the_cpu_rasteriser_supersamples_by_the_chosen_aa() {
        let mut s = BState::new();
        assert_eq!(BViewportAa::supersample(&s), 3, "default (8 samples) oversamples 3x");
        for (id, want) in [("OFF", 1), ("FXAA", 1), ("5", 2), ("8", 3), ("11", 3), ("16", 4), ("32", 4)] {
            s.set(AppKey::ViewportAa, id);
            assert_eq!(BViewportAa::supersample(&s), want, "{id} → {want}x");
        }
    }
}
