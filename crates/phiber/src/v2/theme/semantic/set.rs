//! What each slot gets — the surfaces, host panes, strips, tabs and
//! canvases, every one a step of the ladder (`super`), and the few
//! metrics the rails and tabs derive from the resolved spacing. The
//! strokes, checker and shadow are `stroke.rs`.

use super::{BControlState, BGradientColors, BSurfaceTone};
use crate::v2::host::BHostType;
use crate::v2::theme::{BColor, BResolvedUiStyle};

impl BResolvedUiStyle {
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

    /// The vertical pad a host's content region keeps above its first card:
    /// Properties sits its panels close (Blender's `UI_PANEL_MARGIN_Y`); every
    /// other region pads its content the module's `pad_md`.
    pub fn content_pad(&self, host: BHostType) -> f32 {
        match host {
            BHostType::Buts => self.spacing.pad_sm,
            _ => self.spacing.pad_md,
        }
    }

    /// Top inset before a rail's first cap. Beside a column of cards
    /// (Properties, Preferences) the cap sits level with the first row of
    /// that column — the region's pad, the body column's border, its inner
    /// pad and the card's border down — so a rail entry and the row beside
    /// it share one grid (Themes lines up with the second theme row). A rail
    /// with no cards beside it (a toolbar over a canvas, the activity bar)
    /// keeps the module's own lead.
    pub fn rail_lead(&self, host: BHostType) -> f32 {
        // A rail entry is the card-head module, so the first entry's top is
        // the first heading's top: nothing to centre.
        match host {
            BHostType::Buts | BHostType::Prefs => {
                self.content_pad(host) + 2.0 * self.strokes.border + self.spacing.pad_sm
            }
            _ => self.spacing.pad_sm + self.spacing.pad_md,
        }
    }

    /// A selected thing under the pointer: one step up.
    pub fn selected_hover(&self, selected: BColor) -> BColor {
        self.step(selected, 1)
    }

    /// The workspace behind the areas and its gutters: the app shade, so a
    /// seam reads as the window showing through.
    pub fn workspace_background(&self) -> BColor {
        self.colors.bg_app
    }

    pub fn resize_gutter_background(&self) -> BColor {
        self.workspace_background()
    }

    /// The resting seam is the theme's border shade, a line inside the gap;
    /// hover and drag show the accent.
    pub fn resize_seam_resting_color(&self) -> BColor {
        self.colors.border
    }

    pub fn resize_seam_active_color(&self) -> BColor {
        self.colors.accent
    }

    /// The one shade every mesh takes in Solid shading — the theme's, two
    /// steps off its dim text, so a cube, a sphere and a loaded model all
    /// read against the canvas the same way (Blender's `single_color`).
    pub fn solid_shade(&self) -> BColor {
        self.step(self.colors.text_dim, 2)
    }

    /// Whether a host is the outer chrome (top bar, status bar, activity
    /// rail) rather than an editor.
    pub fn is_chrome(host: BHostType) -> bool {
        matches!(host, BHostType::Topbar | BHostType::Statusbar | BHostType::Activity)
    }

    /// A host's resting pane: the chrome shade for the bars and the rail,
    /// the panel shade for every editor. Canvases paint their own shade over
    /// it (`viewport_surface`).
    pub fn host_background(&self, host: BHostType) -> BColor {
        match host {
            BHostType::Topbar => self.colors.bg_top,
            BHostType::Statusbar => self.colors.bg_bottom,
            _ => self.colors.bg_panel,
        }
    }

    /// A host's body: flat, every editor and the activity bar alike — a
    /// vertical strip takes the surface of the area it sits in, no darker
    /// shade of its own.
    pub fn host_body_surface(&self, host: BHostType) -> BGradientColors {
        Self::flat(self.host_background(host))
    }

    pub fn resting_host_background(&self, host: BHostType) -> BColor {
        self.host_background(host)
    }

    /// Header paint for a host: the bars are their own chrome shade; every
    /// editor header is the header surface.
    pub fn host_header_surface(&self, host: BHostType) -> BGradientColors {
        if Self::is_chrome(host) {
            return Self::flat(self.host_background(host));
        }
        self.surface(BSurfaceTone::Header)
    }

    /// A control's fill in a strip or rail. Docked chrome paints nothing at
    /// rest (`None`) and takes the accent at an alpha for hover and active.
    /// Floating chrome — the viewport's toolbar and sidebar tabs, sitting
    /// on the canvas with no rail behind them — is a button: the control
    /// shade at rest, the hover shade under the pointer, the accent when
    /// active, as the approved mock-ups draw them.
    pub fn control_fill(&self, state: BControlState, floating: bool) -> Option<BColor> {
        if floating {
            return Some(match state {
                BControlState::Rest => self.colors.bg_control,
                BControlState::Hover => self.colors.bg_hover,
                BControlState::Active => self.colors.accent,
            });
        }
        let (hue, saturation, lightness, _) = self.colors.accent;
        let alpha = match state {
            BControlState::Rest => return None,
            BControlState::Hover => self.effects.control_hover_alpha,
            BControlState::Active => self.effects.control_active_alpha,
        };
        Some((hue, saturation, lightness, alpha))
    }

    /// An icon's ink: the icon shade (14), a step quieter than text; the
    /// highlighted-text shade (18) when its control is active.
    pub fn icon_ink(&self, active: bool) -> BColor {
        use crate::v2::theme::palette::set::{SHADE_ICON, SHADE_TEXT_HI};
        self.colors.ladder.color(if active { SHADE_TEXT_HI } else { SHADE_ICON })
    }

    /// Hover, in every host: the hover shade.
    pub fn host_item_hover(&self, _host: BHostType) -> BColor {
        self.colors.bg_hover
    }

    /// Active, in every host: the active shade, swept from one step above.
    pub fn host_item_active_gradient(&self, _host: BHostType) -> BGradientColors {
        self.sheen(self.colors.bg_active)
    }

    /// A header strip inside a host: the header shade.
    pub fn strip_gradient(&self, host: BHostType) -> BGradientColors {
        if Self::is_chrome(host) {
            return Self::flat(self.host_background(host));
        }
        self.sheen(self.colors.bg_header)
    }

    /// `TH_CAMERA_PASSEPARTOUT` at the camera's `passepartalpha`: the
    /// ladder's darkest shade (Blender's is black), half transparent.
    pub fn camera_passepartout(&self) -> BColor {
        self.colors.ladder.at(1, crate::v2::scene::camera_view::PASSEPARTOUT_ALPHA)
    }

    /// `TH_VIEW_OVERLAY`: the camera frame's dashed line. Blender's is black;
    /// here the darkest shade, so it reads on the passepartout in any theme.
    pub fn view_overlay(&self) -> BColor {
        self.colors.ladder.color(1)
    }

    pub fn tab_hover(&self, host: BHostType) -> BColor {
        self.host_item_hover(host)
    }

    /// A Properties rail tab hands its shade to the panel beside it: the
    /// sweep starts one step over the panel at the rail's outer edge and
    /// lands on the panel's own shade at the inner edge, so the selected tab
    /// and the content read as one surface (the caller turns it onto the
    /// rail-to-panel axis with `along`).
    pub fn tab_active_gradient(&self, host: BHostType) -> BGradientColors {
        if host == BHostType::Buts {
            return self.sheen(self.colors.bg_panel);
        }
        self.host_item_active_gradient(host)
    }

    pub fn rail_tab_hover(&self, _host: BHostType) -> BColor {
        self.colors.bg_hover
    }

    /// Active fill for an icon-rail tab: the active shade, never the accent.
    pub fn rail_tab_active(&self, _host: BHostType) -> BGradientColors {
        self.sheen(self.colors.bg_active)
    }

    /// The canvas: the theme's sweep when it has one — a step lighter at
    /// the left edge, the canvas shade by mid-width — else flat. The 3D
    /// viewport's tracer paints the same sweep into its sky
    /// (`viewport_sweep_lift`); the node, image and time canvases paint it
    /// as a background.
    pub fn viewport_gradient(&self) -> BGradientColors {
        self.sheen(self.viewport_surface())
    }

    /// How much lighter the canvas sweep's left edge is than the canvas, as
    /// a fraction of full white (0 when the theme has no sweep) — what the
    /// tracer adds to its sky at the left edge (`Camera::sky.a`).
    pub fn viewport_sweep_lift(&self) -> f32 {
        self.viewport_gradient().lift() / 100.0
    }
}
