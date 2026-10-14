//! What each slot gets — the surfaces, host panes, strips, tabs and
//! canvases, every one a step of the ladder (`super`). The metrics the
//! rails and tabs derive from the resolved spacing are `rail.rs`; the
//! strokes, checker and shadow are `stroke.rs`.

use super::{BControlState, BGradientColors, BSurfaceTone};
use crate::v2::host::BHostType;
use crate::v2::theme::{BColor, BResolvedUiStyle};

impl BResolvedUiStyle {
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
    ///
    /// The Properties editor used to be the exception: its body took one step
    /// under its background so the strip beside it read as a strip and the
    /// selected tab, painted in the body's shade, as a hole cut onto the
    /// panel. That put a colour behind the strip that the column beside it
    /// did not have. The strip is the column's own surface now, and the
    /// selected tab carries the accordions' shade instead
    /// (`tab_active_gradient`), so the step is gone.
    pub fn host_body_surface(&self, host: BHostType) -> BGradientColors {
        Self::flat(self.host_background(host))
    }

    /// A vertical strip's own surface: the host's body, flat — the strip and the
    /// content beside it are one colour.
    ///
    /// Blender's, read rather than reasoned. `region_background_color_id`
    /// (`editors/screen/area.cc:3012`) returns `TH_HEADER` for a header,
    /// `TH_PREVIEW_BACK` for a preview and **`TH_BACK` for everything else** —
    /// `RGN_TYPE_NAV_BAR` included. So the Properties navigation bar is painted
    /// in exactly the same colour as the panel it sits beside; there is no
    /// darker rail shade anywhere in Blender.
    ///
    /// What carries the distinction is the TABS, not the strip.
    /// `userdef_default_theme.c`'s `wcol_tab` has `inner = 0x1d1d1d` and
    /// `inner_sel = 0x303030` against a `space_properties.back` of `0x303030`:
    /// an UNSELECTED tab is a darker chip, and the selected one is the
    /// background's own shade, so it reads as a hole cut onto the panel.
    ///
    /// This used to be `step(body, -1)` — the whole strip darkened — with the
    /// active tab at the body's shade. That is the same two shades as Blender's
    /// with the roles swapped: the dark belonged on the resting tab
    /// (`rail_tab_resting`) and the strip belonged level. A darkened strip reads
    /// as a separate panel bolted to the side of the editor rather than as part
    /// of it. It is the body's own surface (`host_body_surface`) — the one
    /// colour the strip and the column beside it share.
    pub fn rail_surface(&self, host: BHostType) -> BGradientColors {
        self.host_body_surface(host)
    }

    /// A rail tab at rest: one step under the strip it sits on — Blender's
    /// `wcol_tab.inner` against `TH_BACK`. No Properties tab draws it any more;
    /// that step belongs to the body (`host_body_surface`).
    pub fn rail_tab_resting(&self, host: BHostType) -> BColor {
        self.step(self.host_background(host), -1)
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

    /// A control's fill in a strip or rail.
    ///
    /// Every button carries the control shade at rest, docked or floating:
    /// a header's icon reads as the same object as the viewport toolbar's,
    /// which is what a button looks like here. Docked chrome used to paint
    /// nothing at rest, so the same control was a tile on the canvas and a
    /// bare glyph in a header.
    ///
    /// They still part under the pointer: floating chrome sits on the canvas
    /// with no rail behind it and takes the hover shade, then the accent
    /// when active; docked chrome washes the accent over its rail at an
    /// alpha instead, so the strip shows through.
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
            // The resting fill at `control_rest_alpha`, which was declared,
            // given a value and never read — Rest returned `bg_control` whole.
            // So a menu row's resting ground was as solid as the panel under
            // it, and the knob for saying otherwise sat there unused. Wired,
            // and set to half.
            BControlState::Rest => {
                let (hue, saturation, lightness, alpha) = self.colors.bg_control;
                return Some((hue, saturation, lightness, alpha * self.effects.control_rest_alpha));
            }
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
    /// The active tab in a rail. In Properties it is exactly the surface of
    /// the region beside it (`host_body_surface`) — flat, not a sheen: the
    /// tab is a hole cut in the strip onto the panel it selects, and a
    /// gradient on one side of that join makes the two read as two things.
    /// The selected tab's fill. A Properties tab is the accordion heading
    /// beside it — the same `heading_surface(Card)` the panels' heads are
    /// painted in, lit edge and all — so the selected tab and the first
    /// accordion, which sit level (`rail_lead`), read as one row of the same
    /// material: the tab names the panel it opens by wearing its colour. It
    /// is not the body (`host_body_surface`) any more: the strip is the body
    /// now, and a tab in the strip's own colour would mark nothing.
    pub fn tab_active_gradient(&self, host: BHostType) -> BGradientColors {
        if host == BHostType::Buts {
            return self.heading_surface(BSurfaceTone::Card);
        }
        self.host_item_active_gradient(host)
    }

    pub fn rail_tab_hover(&self, _host: BHostType) -> BColor {
        self.colors.bg_hover
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
