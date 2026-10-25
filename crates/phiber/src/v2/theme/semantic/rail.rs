//! The few numbers the rails and tabs derive from the resolved spacing.
//!
//! Split out of `set.rs`, which is the surfaces: a lead, an inset and an
//! overlap are geometry, not colour, and they were the one thing in that
//! file that answered in pixels. Each is a derivation rather than a
//! literal, so a theme that changes its spacing moves the rail with it.

use crate::v2::host::BHostType;
use crate::v2::theme::BResolvedUiStyle;

impl BResolvedUiStyle {
    /// Every rail's collapsed width — the toolbar, the sidebar, the
    /// Properties and Preferences navbars, docked or floating over a canvas:
    /// one card-head square tile and `pad_xs` either side of it. One number,
    /// so no host's strip can come out a different width from another's.
    pub fn rail_width(&self) -> f32 {
        self.dimensions.card_head_height() + 2.0 * self.spacing.pad_xs
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
        // the first heading's top: nothing to centre. The rail's own bar pads
        // `pad_xs` inside its ground (`rail.rs`), so the region pads that much
        // less.
        let lead = match host {
            BHostType::Buts | BHostType::Prefs => {
                // The body column's border; a card has no border of its own.
                self.content_pad(host) + self.strokes.border + self.spacing.pad_sm
            }
            _ => self.spacing.pad_sm + self.spacing.pad_md,
        };
        lead - self.spacing.pad_xs
    }
}
