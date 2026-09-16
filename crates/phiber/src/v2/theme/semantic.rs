//! Kit-neutral semantic color, gradient, stroke, and effect derivations.
//!
//! GPUI renderers adapt these values to backend types. The resolved style owns
//! every design decision so compatibility renderers do not invent local shade
//! ladders or geometry defaults while focused renderer migration is pending.

use super::{BColor, BResolvedUiStyle, BShadowStyle};
use crate::v2::host::BHostType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BGradientColors {
    pub top: BColor,
    pub bottom: BColor,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BStrokeStyle {
    pub color: BColor,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCheckerColors {
    pub light: BColor,
    pub dark: BColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BSurfaceTone {
    Header,
    Footer,
    Sidebar,
    Body,
    Viewport,
    Modal,
    Card,
    Section,
}

impl BResolvedUiStyle {
    pub fn shade(&self, base: BColor, delta: f32) -> BColor {
        (
            base.0,
            base.1,
            (base.2 + delta * 100.0).clamp(0.0, 100.0),
            base.3,
        )
    }

    pub fn gradient(&self, base: BColor, top_delta: f32, bottom_delta: f32) -> BGradientColors {
        if !self.effects.gradients {
            return BGradientColors {
                top: base,
                bottom: base,
                enabled: false,
            };
        }
        BGradientColors {
            top: self.shade(base, top_delta),
            bottom: self.shade(base, bottom_delta),
            enabled: true,
        }
    }

    pub fn sheen(&self, base: BColor) -> BGradientColors {
        self.gradient(base, self.effects.gradient_lift, 0.0)
    }

    pub fn subtle_sheen(&self, base: BColor) -> BGradientColors {
        self.gradient(base, self.effects.sheen_lift, 0.0)
    }

    pub fn surface(&self, tone: BSurfaceTone) -> BGradientColors {
        let (base, delta) = match tone {
            BSurfaceTone::Header => (self.colors.bg_header, 0.0),
            BSurfaceTone::Footer => (self.colors.bg_panel, self.effects.footer_lift),
            BSurfaceTone::Sidebar => (self.colors.bg_panel, self.effects.sidebar_lift),
            BSurfaceTone::Body | BSurfaceTone::Modal => (self.colors.bg_panel, 0.0),
            BSurfaceTone::Viewport => {
                let color = self.viewport_surface();
                return BGradientColors {
                    top: color,
                    bottom: color,
                    enabled: false,
                };
            }
            BSurfaceTone::Card => (self.colors.bg_panel, self.effects.card_lift),
            BSurfaceTone::Section => (self.colors.bg_panel, self.effects.section_lift),
        };
        self.sheen(self.shade(base, delta))
    }

    /// Opaque, flat View3D base derived from the palette viewport token. The
    /// resolved delta lifts the old midpoint to its former sky value without
    /// retaining the sky/floor gradient or horizon band.
    pub fn viewport_surface(&self) -> BColor {
        let mut color = self.shade(
            self.colors.bg_viewport,
            self.effects.viewport_lightness_delta,
        );
        color.3 = 1.0;
        color
    }

    /// Left edge of the Properties rail-tab paint layer. Two resolved border
    /// strokes preserve the requested 2px default while remaining compatible
    /// with runtime stroke overrides.
    pub fn properties_tab_background_inset(&self) -> f32 {
        self.strokes.border * 2.0
    }

    /// Content-facing overlap for Properties rail-tab paint. The paint layer
    /// extends without changing the tab or strip allocation.
    pub fn properties_tab_overlap(&self) -> f32 {
        self.spacing.gap_xs.max(self.strokes.border)
    }

    pub fn selected_hover(&self, selected: BColor) -> BColor {
        self.shade(selected, self.effects.selected_hover_lift)
    }

    pub fn workspace_background(&self) -> BColor {
        // Area cards use the neutral panel surface. Painting the workspace and
        // its resize gutters with the same token prevents layout spacing from
        // reading as a separate colored band.
        self.colors.bg_panel
    }

    pub fn resize_gutter_background(&self) -> BColor {
        self.workspace_background()
    }

    /// Resting resize lines retain the semantic border hue but paint with no
    /// opacity. The broad overlay remains interactive; only hover/drag reveals
    /// the accent line.
    pub fn resize_seam_resting_color(&self) -> BColor {
        let mut color = self.colors.border;
        color.3 = 0.0;
        color
    }

    pub fn resize_seam_active_color(&self) -> BColor {
        self.colors.accent
    }

    pub fn host_lightness(&self, host: BHostType) -> f32 {
        match host {
            BHostType::View3D => 24.0,
            BHostType::Buts
            | BHostType::Ipo
            | BHostType::Action
            | BHostType::Nla
            | BHostType::Image
            | BHostType::Uv
            | BHostType::Prefs
            | BHostType::Clip
            | BHostType::Statusbar
            | BHostType::Drivers => 19.0,
            BHostType::Outliner
            | BHostType::File
            | BHostType::Asset
            | BHostType::Spreadsheet
            | BHostType::Time
            | BHostType::Empty => 16.0,
            BHostType::Text => 14.0,
            BHostType::Activity => 12.0,
            BHostType::Info | BHostType::Console | BHostType::Logic => 11.0,
            BHostType::Node | BHostType::Shader | BHostType::Compositor | BHostType::GeoNode => {
                10.0
            }
            BHostType::Topbar | BHostType::Seq => 9.0,
        }
    }

    pub fn host_background(&self, host: BHostType) -> BColor {
        let panel = self.colors.bg_panel;
        (panel.0, panel.1, self.host_lightness(host), 1.0)
    }

    /// Resting host paint. Global Topbar chrome is intentionally transparent
    /// so the neutral workspace/area surface remains the visual owner.
    pub fn resting_host_background(&self, host: BHostType) -> BColor {
        if host == BHostType::Topbar {
            let mut neutral = self.workspace_background();
            neutral.3 = 0.0;
            neutral
        } else {
            self.host_background(host)
        }
    }

    /// Header paint for a host. Editor-host headers use the resolved header
    /// surface tone (`bg_header`) — a subtle chrome band in the same tonal
    /// family as the pane, NOT the near-black app frame. Topbar keeps its
    /// transparent-over-workspace treatment.
    pub fn host_header_surface(&self, host: BHostType) -> BGradientColors {
        if host == BHostType::Topbar {
            return BGradientColors {
                top: self.resting_host_background(host),
                bottom: self.resting_host_background(host),
                enabled: false,
            };
        }
        self.surface(BSurfaceTone::Header)
    }

    /// Light, host-matched feedback for interactive chrome. Topbar derives
    /// from the adjacent neutral workspace instead of its historical dark
    /// host rung; other hosts preserve their recessed-strip relationship.
    pub fn host_item_hover(&self, host: BHostType) -> BColor {
        if host == BHostType::Topbar {
            self.shade(self.workspace_background(), self.effects.sheen_lift)
        } else {
            let pane = self.host_background(host);
            let strip = self.strip_gradient(host).bottom;
            (pane.0, pane.1, (strip.2 + pane.2) * 0.5, 1.0)
        }
    }

    pub fn host_item_active_gradient(&self, host: BHostType) -> BGradientColors {
        let pane = if host == BHostType::Topbar {
            self.host_item_hover(host)
        } else {
            self.host_background(host)
        };
        if !self.effects.gradients {
            return BGradientColors {
                top: pane,
                bottom: pane,
                enabled: false,
            };
        }
        let accent = self.colors.accent;
        let top = (
            pane.0 + (accent.0 - pane.0) * self.effects.tab_accent_mix,
            (pane.1 + self.effects.tab_saturation_lift * 100.0).min(100.0),
            (pane.2 + (self.effects.gradient_lift + self.effects.tab_top_lift) * 100.0).min(100.0),
            pane.3,
        );
        BGradientColors {
            top,
            bottom: pane,
            enabled: true,
        }
    }

    pub fn strip_gradient(&self, host: BHostType) -> BGradientColors {
        let pane = self.host_background(host);
        let base = (
            pane.0,
            pane.1,
            (pane.2 - self.effects.strip_recess).max(2.0),
            1.0,
        );
        self.gradient(base, self.effects.strip_gradient_lift, 0.0)
    }

    /// A lighter variant of the strip gradient used for icon-only rails
    /// (Properties navbar, activity bar) where strong contrast between icons
    /// and the strip bg is unwanted. Uses half the strip recess.
    pub fn strip_gradient_soft(&self, host: BHostType) -> BGradientColors {
        let pane = self.host_background(host);
        let base = (
            pane.0,
            pane.1,
            (pane.2 - self.effects.strip_recess * 0.4).max(2.0),
            1.0,
        );
        self.gradient(base, self.effects.strip_gradient_lift * 0.5, 0.0)
    }

    pub fn tab_hover(&self, host: BHostType) -> BColor {
        self.host_item_hover(host)
    }

    pub fn tab_active_gradient(&self, host: BHostType) -> BGradientColors {
        // Properties rail tabs (icon-only, BHostType::Buts) use the flat body
        // surface so the active tab reads as a seamless continuation of the
        // content panel behind it — same bg_panel tone, no accent tint.
        if host == BHostType::Buts {
            return self.surface(BSurfaceTone::Body);
        }
        self.host_item_active_gradient(host)
    }

    /// Tab hover for icon-only rails (activity bar, non-Buts navbar).
    /// Uses the soft strip base rather than the full strip so the hover
    /// lift is proportionally lighter relative to the softened resting bg.
    pub fn rail_tab_hover(&self, host: BHostType) -> BColor {
        let soft = self.strip_gradient_soft(host).bottom;
        self.shade(soft, self.effects.sheen_lift * 1.5)
    }

    /// Active fill for icon-only rail tabs (activity bar, non-Buts navbar).
    ///
    /// Deliberately does NOT tint toward `colors.accent` (avoids the "foreign
    /// blue" active look) — the icon rail sits on the app chrome, not on a
    /// coloured host, so its active state must stay in the parent's tonal
    /// family. Same shape as a boxed slot / shader-host card: a small vertical
    /// lift over the parent strip, using the shared `effects.tab_top_lift`
    /// magnitude so all lifts scale together.
    pub fn rail_tab_active(&self, host: BHostType) -> BGradientColors {
        let base = self.strip_gradient_soft(host).bottom;
        let lift = self.effects.tab_top_lift + self.effects.sheen_lift;
        let top = self.shade(base, lift);
        BGradientColors {
            top,
            bottom: base,
            enabled: self.effects.gradients,
        }
    }

    /// Compatibility accessor for callers expecting a gradient-shaped value.
    /// The View3D base is deliberately flat regardless of global sheen use.
    pub fn viewport_gradient(&self) -> BGradientColors {
        let color = self.viewport_surface();
        BGradientColors {
            top: color,
            bottom: color,
            enabled: false,
        }
    }

    pub fn border_stroke(&self) -> BStrokeStyle {
        BStrokeStyle {
            color: self.colors.border,
            width: self.strokes.border,
        }
    }

    pub fn separator_stroke(&self) -> BStrokeStyle {
        BStrokeStyle {
            color: self.colors.sep,
            width: self.strokes.hairline,
        }
    }

    pub fn viewport_grid_stroke(&self) -> BStrokeStyle {
        let mut color = self.colors.viewport_grid;
        color.3 = self.effects.viewport_grid_alpha;
        BStrokeStyle {
            color,
            width: self.strokes.grid,
        }
    }

    pub fn axis_stroke(&self, mut color: BColor) -> BStrokeStyle {
        color.3 = self.effects.axis_alpha;
        BStrokeStyle {
            color,
            width: self.strokes.grid,
        }
    }

    pub fn node_grid_stroke(&self) -> BStrokeStyle {
        let mut color = self.shade(self.colors.bg_viewport, self.effects.node_grid_lift);
        color.3 = self.effects.node_grid_alpha;
        BStrokeStyle {
            color,
            width: self.strokes.grid,
        }
    }

    pub fn image_checker_colors(&self) -> BCheckerColors {
        BCheckerColors {
            light: self.shade(self.colors.bg_viewport, self.effects.image_light_lift),
            dark: self.shade(self.colors.bg_viewport, self.effects.image_dark_lift),
        }
    }

    pub fn sidebar_min_width(&self) -> f32 {
        self.dimensions
            .sidebar_width
            .max(self.dimensions.sidebar_min_width)
    }

    pub fn surface_shadow(&self) -> BShadowStyle {
        self.effects.surface_shadow
    }
}
