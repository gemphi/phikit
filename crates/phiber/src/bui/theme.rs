//! Theme system — switchable color palettes with gradient support.
//! Themes drive all panel/element colors. Gradients use color-to-transparent
//! transitions (top→bottom, bottom→top).
//! Radii and metric scaling provide setting-driven smooth rounded layouts.

use gpui::{Background, Hsla, hsla, linear_color_stop, linear_gradient};

/// Available themes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BTheme {
    BlenderDark,
    AppleDark,
}

/// A complete color palette and metric specification for one theme.
#[derive(Clone, Copy, Debug)]
pub struct BPalette {
    pub name: &'static str,
    pub bg_app: Hsla,
    pub bg_top: Hsla,
    pub bg_panel: Hsla,
    pub bg_header: Hsla,
    pub bg_toolbar: Hsla,
    pub bg_viewport: Hsla,
    pub bg_status: Hsla,
    pub bg_active: Hsla,
    pub bg_hover: Hsla,
    pub bg_icon: Hsla,
    pub bg_icon_hover: Hsla,
    pub bg_icon_active: Hsla,
    pub border: Hsla,
    pub text: Hsla,
    pub text_dim: Hsla,
    pub accent: Hsla,
    // Viewport canvas tokens (Blender `ThemeSpace` for SPACE_VIEW3D).
    pub viewport_grid: Hsla,
    pub axis_x: Hsla,
    pub axis_y: Hsla,
    pub axis_z: Hsla,
    /// Wire color of the active/selected object (Blender orange).
    pub wire_active: Hsla,
    /// Whether to use gradient fills; if false, all fills fall back to solid color.
    pub use_gradients: bool,
    pub radius_xs: f32,
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub radius_xl: f32,
    pub h_tab: f32,
    pub h_header: f32,
    pub h_footer: f32,
    pub h_icon: f32,
    pub h_separator: f32,
    pub w_sidebar: f32,
    pub w_separator: f32,
    pub w_toolbar: f32,
    pub w_panel: f32,
    pub w_channels: f32,
    pub w_popup: f32,
    pub w_left: f32,
    pub w_right: f32,
    pub h_bottom: f32,
    pub pad_xs: f32,
    pub pad_sm: f32,
    pub pad_md: f32,
    pub gap_xs: f32,
    pub gap_sm: f32,
    pub gap_md: f32,
}

impl BPalette {
    /// Blender dark theme — warm grey-blue with smooth, setting-scaled radii.
    /// Color hierarchy: bg_app (darkest) < areas < inner content < headings/active.
    pub const fn blender_dark() -> Self {
        Self {
            name: "Blender Dark",
            // Outermost bg — darkest.
            bg_app: hsla(220. / 360., 0.14, 0.11, 1.),
            // Areas — distinct shades.
            bg_top: hsla(220. / 360., 0.14, 0.16, 1.),
            bg_panel: hsla(222. / 360., 0.13, 0.165, 1.),
            bg_toolbar: hsla(220. / 360., 0.11, 0.145, 1.),
            // Inner content — lighter than areas.
            bg_header: hsla(220. / 360., 0.14, 0.185, 1.),
            bg_viewport: hsla(220. / 360., 0.14, 0.22, 1.),
            bg_status: hsla(220. / 360., 0.14, 0.13, 1.),
            // Headings/active and hover.
            bg_active: hsla(210. / 360., 0.25, 0.28, 1.),
            bg_hover: hsla(220. / 360., 0.14, 0.24, 1.),
            bg_icon: hsla(220. / 360., 0.12, 0.20, 1.),
            bg_icon_hover: hsla(220. / 360., 0.15, 0.26, 1.),
            bg_icon_active: hsla(210. / 360., 0.25, 0.30, 1.),
            border: hsla(220. / 360., 0.14, 0.22, 1.),
            text: hsla(0., 0., 0.85, 1.),
            text_dim: hsla(0., 0., 0.55, 1.),
            accent: hsla(210. / 360., 0.50, 0.55, 1.),
            // Blender 5.2 ThemeSpace defaults.
            viewport_grid: hsla(210. / 360., 0.02, 0.34, 1.),
            axis_x: hsla(348.6 / 360., 1.0, 0.60, 1.),
            axis_y: hsla(81. / 360., 1.0, 0.43, 1.),
            axis_z: hsla(210. / 360., 1.0, 0.58, 1.),
            wire_active: hsla(27.6 / 360., 1.0, 0.53, 1.),
            use_gradients: true,
            radius_xs: 3.,
            radius_sm: 6.,
            radius_md: 10.,
            radius_lg: 14.,
            radius_xl: 18.,
            h_tab: 28.,
            h_header: 36.,
            h_footer: 28.,
            h_icon: 32.,
            h_separator: 6.,
            w_sidebar: 32.,
            w_separator: 6.,
            w_toolbar: 44.,
            w_panel: 260.,
            w_channels: 200.,
            w_popup: 200.,
            w_left: 240.,
            w_right: 300.,
            h_bottom: 160.,
            pad_xs: 4.,
            pad_sm: 8.,
            pad_md: 10.,
            gap_xs: 2.,
            gap_sm: 4.,
            gap_md: 8.,
        }
    }

    /// Apple dark theme — blue-grey, cooler tones with smooth, generous curves.
    /// Color hierarchy: bg_app (darkest) < areas < inner content < headings/active.
    pub const fn apple_dark() -> Self {
        Self {
            name: "Apple Dark",
            // Outermost bg — darkest.
            bg_app: hsla(215. / 360., 0.15, 0.10, 1.),
            // Areas — lighter than bg_app.
            bg_top: hsla(215. / 360., 0.18, 0.14, 1.),
            bg_panel: hsla(215. / 360., 0.14, 0.155, 1.),
            bg_toolbar: hsla(215. / 360., 0.12, 0.135, 1.),
            // Inner content — lighter than areas.
            bg_header: hsla(215. / 360., 0.16, 0.175, 1.),
            bg_viewport: hsla(215. / 360., 0.10, 0.22, 1.),
            bg_status: hsla(215. / 360., 0.15, 0.12, 1.),
            // Headings/active and hover.
            bg_active: hsla(211. / 360., 0.28, 0.28, 1.),
            bg_hover: hsla(215. / 360., 0.12, 0.23, 1.),
            bg_icon: hsla(215. / 360., 0.12, 0.19, 1.),
            bg_icon_hover: hsla(215. / 360., 0.15, 0.25, 1.),
            bg_icon_active: hsla(211. / 360., 0.30, 0.30, 1.),
            border: hsla(215. / 360., 0.10, 0.22, 1.),
            text: hsla(0., 0., 0.90, 1.),
            text_dim: hsla(215. / 360., 0.08, 0.55, 1.),
            accent: hsla(211. / 360., 0.60, 0.55, 1.),
            viewport_grid: hsla(215. / 360., 0.05, 0.34, 1.),
            axis_x: hsla(348.6 / 360., 1.0, 0.60, 1.),
            axis_y: hsla(81. / 360., 1.0, 0.43, 1.),
            axis_z: hsla(211. / 360., 1.0, 0.58, 1.),
            wire_active: hsla(27.6 / 360., 1.0, 0.53, 1.),
            use_gradients: true,
            radius_xs: 3.,
            radius_sm: 6.,
            radius_md: 10.,
            radius_lg: 14.,
            radius_xl: 18.,
            h_tab: 28.,
            h_header: 36.,
            h_footer: 28.,
            h_icon: 32.,
            h_separator: 6.,
            w_sidebar: 32.,
            w_separator: 6.,
            w_toolbar: 44.,
            w_panel: 260.,
            w_channels: 200.,
            w_popup: 200.,
            w_left: 240.,
            w_right: 300.,
            h_bottom: 160.,
            pad_xs: 4.,
            pad_sm: 8.,
            pad_md: 10.,
            gap_xs: 2.,
            gap_sm: 4.,
            gap_md: 8.,
        }
    }

    pub fn for_theme(theme: BTheme) -> Self {
        match theme {
            BTheme::BlenderDark => Self::blender_dark(),
            BTheme::AppleDark => Self::apple_dark(),
        }
    }

    /// Resolve palette from state — reads `BKey::Theme` and `BKey::UseGradients`.
    pub fn from_state(state: &super::state::BState) -> Self {
        let theme = state
            .get(&super::state::BKey::Theme)
            .and_then(BTheme::from_name)
            .unwrap_or_default();
        let mut p = Self::for_theme(theme);
        p.use_gradients = state.get(&super::state::BKey::UseGradients) != Some("false");
        p
    }

    /// Scale radii by a setting multiplier (e.g. 1.25 for ultra-smooth).
    pub fn scaled(&self, scale: f32) -> Self {
        let mut p = *self;
        p.radius_xs *= scale;
        p.radius_sm *= scale;
        p.radius_md *= scale;
        p.radius_lg *= scale;
        p.radius_xl *= scale;
        p
    }

    /// Modern smooth preset scaling.
    pub fn smooth(&self) -> Self {
        self.scaled(1.25)
    }

    /// Gradient from a solid color at top fading to transparent at bottom.
    pub fn grad_t_t_b(color: Hsla) -> Background {
        let transparent = hsla(color.h, color.s, color.l, 0.0);
        linear_gradient(
            180.0,
            linear_color_stop(color, 0.0),
            linear_color_stop(transparent, 1.0),
        )
    }

    /// Gradient from transparent at top fading to a solid color at bottom.
    pub fn grad_b_t_t(color: Hsla) -> Background {
        let transparent = hsla(color.h, color.s, color.l, 0.0);
        linear_gradient(
            0.0,
            linear_color_stop(transparent, 0.0),
            linear_color_stop(color, 1.0),
        )
    }

    /// Fill helpers — respect the `use_gradients` setting.
    pub fn fill_solid(&self, color: Hsla) -> Background {
        color.into()
    }

    pub fn fill_sheen(&self, color: Hsla) -> Background {
        if self.use_gradients {
            Self::grad_sheen(color)
        } else {
            color.into()
        }
    }

    pub fn fill_top(&self, color: Hsla) -> Background {
        if self.use_gradients {
            Self::grad_b_t_t(color)
        } else {
            color.into()
        }
    }

    pub fn fill_bottom(&self, color: Hsla) -> Background {
        if self.use_gradients {
            Self::grad_t_t_b(color)
        } else {
            color.into()
        }
    }

    /// Gradient from color to a slightly lighter version (subtle sheen).
    pub fn grad_sheen(color: Hsla) -> Background {
        let lighter = hsla(color.h, color.s, (color.l + 0.04).min(1.0), 1.0);
        linear_gradient(
            180.0,
            linear_color_stop(lighter, 0.0),
            linear_color_stop(color, 1.0),
        )
    }
}

impl Default for BPalette {
    fn default() -> Self {
        Self::for_theme(BTheme::default())
    }
}

impl Default for BTheme {
    fn default() -> Self {
        BTheme::BlenderDark
    }
}

impl BTheme {
    pub fn name(self) -> &'static str {
        match self {
            BTheme::BlenderDark => "BlenderDark",
            BTheme::AppleDark => "AppleDark",
        }
    }

    pub fn from_name(name: &str) -> Option<BTheme> {
        match name {
            "BlenderDark" => Some(BTheme::BlenderDark),
            "AppleDark" => Some(BTheme::AppleDark),
            _ => None,
        }
    }
}
