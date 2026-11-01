//! Resolved theme data types — the baked result after composing all layers.
//!
//! These are the concrete structs that the registry produces and that
//! draw functions consume. They hold the final resolved values for
//! colors, fonts, geometry, and capabilities.

use crate::NSColor;
use std::collections::HashMap;

use super::styles::{Material, Variant};
use super::appearance::AppearanceMode;

// =============================================================================
// ThemeId — identifies which layers produced this theme
// =============================================================================

#[derive(Debug, Clone)]
pub struct ThemeId {
    pub variant: Variant,
    pub mode: AppearanceMode,
    pub material: Material,
}

impl ThemeId {
    pub fn new(variant: Variant, mode: AppearanceMode, material: Material) -> Self {
        Self {
            variant,
            mode,
            material,
        }
    }
}

// =============================================================================
// ThemeProvider trait — for custom theme builders
// =============================================================================

pub trait ThemeProvider {
    fn build_theme(&self) -> ThemeData;
}

// =============================================================================
// ThemeFont — resolved font stack
// =============================================================================

#[derive(Debug, Clone)]
pub struct ThemeFont {
    pub ui_font: String,
    pub ui_font_size: f32,
    pub title_font: String,
    pub title_font_size: f32,
    pub monospace_font: String,
    pub monospace_font_size: f32,
    pub icon_font: Option<String>,
}

// =============================================================================
// ThemeLayout — resolved geometry tokens
// =============================================================================

#[derive(Debug, Clone)]
pub struct ThemeLayout {
    pub geometry: HashMap<String, f32>,
}

// =============================================================================
// ThemeData — the complete resolved theme
// =============================================================================

#[derive(Debug, Clone)]
pub struct ThemeData {
    pub id: ThemeId,
    pub colors: HashMap<String, NSColor>,
    pub font: ThemeFont,
    pub layout: ThemeLayout,
    pub supports_transparency: bool,
    pub supports_animations: bool,
}

// =============================================================================
// Appearance — the full resolved selection from all layers
// =============================================================================

use super::appearance::*;
use super::styles::*;

/// Full appearance configuration — one pick from each composable layer.
/// This is the "intent" struct that gets resolved into a `ThemeData`.
#[derive(Debug, Clone)]
pub struct Appearance {
    // Apple layers
    pub name: AppearanceName,
    pub mode: AppearanceMode,
    pub blending: BlendingMode,
    pub state: AppearanceState,
    pub background_style: BackgroundStyle,
    pub control_size: ControlSize,
    pub vibrancy: Vibrancy,
    // Our style layers
    pub variant: Variant,
    pub typography: Typography,
    pub radius: RadiusStyle,
    pub material: Material,
    pub gradient: ColorGradient,
    pub effects: VisualEffects,
    pub density: LayoutDensity,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            name: AppearanceName::DarkAqua,
            mode: AppearanceMode::Dark,
            blending: BlendingMode::Opaque,
            state: AppearanceState::FollowsWindowActiveState,
            background_style: BackgroundStyle::Normal,
            control_size: ControlSize::Regular,
            vibrancy: Vibrancy::Off,
            variant: Variant::Apple,
            typography: Typography::System,
            radius: RadiusStyle::Standard,
            material: Material::Standard,
            gradient: ColorGradient::Subtle,
            effects: VisualEffects::Standard,
            density: LayoutDensity::Standard,
        }
    }
}
