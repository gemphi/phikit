//! Effects surface: shadow style + `BEffects` + palette presets. Split out
//! of `style.rs` so the parent stays under the 200-line cap.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BShadowStyle {
    None,
    ExtraLarge,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BEffects {
    pub gradients: bool,
    pub gradient_lift: f32,
    pub sheen_lift: f32,
    pub selected_hover_lift: f32,
    pub footer_lift: f32,
    pub sidebar_lift: f32,
    pub card_lift: f32,
    pub section_lift: f32,
    pub strip_recess: f32,
    pub strip_gradient_lift: f32,
    pub tab_accent_mix: f32,
    pub tab_saturation_lift: f32,
    pub tab_top_lift: f32,
    pub viewport_lightness_delta: f32,
    pub viewport_grid_alpha: f32,
    pub axis_alpha: f32,
    pub node_grid_lift: f32,
    pub node_grid_alpha: f32,
    pub image_light_lift: f32,
    pub image_dark_lift: f32,
    pub surface_shadow: BShadowStyle,
}

const fn effects(gradient_lift: f32) -> BEffects {
    BEffects {
        gradients: true,
        gradient_lift,
        sheen_lift: 0.02,
        selected_hover_lift: 0.06,
        footer_lift: -0.02,
        sidebar_lift: -0.04,
        card_lift: -0.03,
        section_lift: 0.035,
        strip_recess: 2.0,
        strip_gradient_lift: -0.02,
        tab_accent_mix: 0.25,
        tab_saturation_lift: 0.06,
        tab_top_lift: 0.02,
        viewport_lightness_delta: 0.06,
        viewport_grid_alpha: 0.10,
        axis_alpha: 0.65,
        node_grid_lift: 0.12,
        node_grid_alpha: 0.18,
        image_light_lift: 0.09,
        image_dark_lift: -0.03,
        surface_shadow: BShadowStyle::ExtraLarge,
    }
}

pub const APPLE_EFFECTS: BEffects = effects(0.035);
pub const BLENDER_EFFECTS: BEffects = effects(0.02);
pub const TOKYO_NIGHT_EFFECTS: BEffects = effects(0.025);
