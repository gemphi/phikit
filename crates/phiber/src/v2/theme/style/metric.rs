//! Typography / spacing / dimensions / strokes / resize style plus their
//! defaults. Split out of `style.rs` so the parent stays under the 200-line
//! cap.

use super::font::BFontFamily;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BTypography {
    /// The face every letter is set in (`font.rs`).
    pub family: BFontFamily,
    pub body: f32,
    pub header: f32,
    pub caption: f32,
}

pub const DEFAULT_TYPOGRAPHY: BTypography = BTypography {
    family: BFontFamily::DEFAULT,
    body: 11.0,
    header: 12.0,
    caption: 10.0,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSpacing {
    /// Explicit zero-gap token. Use to override an inherited gap (e.g. a
    /// flex column that carries `gap_sm` by default) when a container must
    /// pack children flush. Kept in the token set so the boundary linter can
    /// distinguish an intentional zero from a hardcoded literal.
    pub gap_none: f32,
    pub gap_xs: f32,
    pub gap_sm: f32,
    pub gap_md: f32,
    pub gap_lg: f32,
    pub pad_xs: f32,
    pub pad_sm: f32,
    pub pad_md: f32,
    pub pad_lg: f32,
}

pub const DEFAULT_SPACING: BSpacing = BSpacing {
    gap_none: 0.0,
    gap_xs: 4.0,
    gap_sm: 6.0,
    gap_md: 8.0,
    gap_lg: 10.0,
    pad_xs: 4.0,
    pad_sm: 6.0,
    pad_md: 8.0,
    pad_lg: 16.0,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BDimensions {
    pub header_height: f32,
    pub footer_height: f32,
    pub row_height: f32,
    /// A tree's row — the Outliner's: Blender's `UI_UNIT_Y` (20 px) at the
    /// 1.5× the module implies, a step under a control row so a tree reads
    /// as a list rather than a stack of buttons.
    pub tree_row_height: f32,
    pub button_height: f32,
    pub scrubbing_height: f32,
    pub asset_shelf_height: f32,
    pub bottom_strip_height: f32,
    /// The action log's drawer when unfolded above the status bar.
    pub status_log_height: f32,
    /// The Welcome banner: a strip tall enough to carry the large mark.
    pub banner_height: f32,
    pub activity_width: f32,
    pub toolbar_width: f32,
    /// Gap above a menu anchor or tab; the cap itself runs to the parent's
    /// bottom edge, so the popover / pane below it continues the same surface.
    pub menu_top_gap: f32,
    pub sidebar_width: f32,
    pub sidebar_min_width: f32,
    /// A number control's floor: below it the value clips. One for all, so a column lines up.
    pub number_min_width: f32,
    pub panel_width: f32,
    pub navbar_width: f32,
    pub channels_width: f32,
    pub tool_props_width: f32,
    pub area_left_width: f32,
    pub area_right_width: f32,
    /// Default width for a Left/Right zone when the user has not dragged an
    /// explicit zone size. Prevents a greedy host from starving Center.
    pub side_zone_default_width: f32,
    pub file_date_column_width: f32,
    pub file_size_column_width: f32,
    pub indent_unit: f32,
    pub restriction_column_width: f32,
    /// Every icon in the chrome is drawn at this size.
    ///
    /// Icons used to be sized off the type scale, which meant they could only
    /// change by changing the text beside them. They are their own thing, so
    /// they get their own number — and the whole set moves together.
    pub icon_size: f32,
    /// The smaller icon, for dense rows: menu items, tree rows, search.
    pub icon_size_sm: f32,
    /// The app's own mark: a shade over the icon size, so it reads as the
    /// mark without towering over the menus beside it (Blender's topbar logo
    /// is an ordinary icon's size). Batch 41 brought it down from 22.4.
    pub brand_size: f32,
    pub gizmo_button_size: f32,
    pub gizmo_icon_size: f32,
    pub object_icon_size: f32,
    pub object_glyph_size: f32,
}

/// One module: a header's height and, turned, every icon rail's width. The
/// window's whole grid is built from it, so growing the chrome is this one
/// number rather than a dozen independent widths.
pub const MODULE: f32 = 36.0;

pub const DEFAULT_DIMENSIONS: BDimensions = BDimensions {
    header_height: MODULE,
    // `ED_area_footersize` is `ED_area_headersize`.
    footer_height: MODULE,
    // A row is a control plus its room: one answer (`a_row_is_one_height`).
    row_height: MODULE,
    tree_row_height: 30.0,
    button_height: BUTTON_HEIGHT,
    scrubbing_height: 24.0,
    asset_shelf_height: 120.0,
    // The Timeline's header and its window (the scrub strip, the summary
    // row and room for the grid — Blender's default Layout gives it about
    // seventy pixels), over the status bar.
    bottom_strip_height: 140.0,
    // Six rows of the log and a little air, over the status bar.
    status_log_height: 168.0,
    // Two modules: the mark inside it is the strip less a small gutter
    // above and below, which puts it at the brand page's large size.
    banner_height: MODULE * 2.0,
    // The activity bar's entries are menu-cap-sized buttons (34 px), the same
    // height as the rows of the secondary strip beside them, so the rail is a
    // module plus an inset either side of the button.
    // Slack's rail: a stacked icon-and-name entry wide (`tab/stack.rs`).
    activity_width: 64.0,
    // The icon rails are as wide as a menu is tall, so the window's grid
    // reads as one module size.
    toolbar_width: MODULE,
    menu_top_gap: 2.0,
    sidebar_width: MODULE,
    sidebar_min_width: 160.0,
    number_min_width: 120.0,
    panel_width: 240.0,
    navbar_width: MODULE,
    channels_width: 200.0,
    tool_props_width: 200.0,
    area_left_width: 156.0,
    area_right_width: 196.0,
    side_zone_default_width: 280.0,
    file_date_column_width: 140.0,
    file_size_column_width: 90.0,
    indent_unit: 20.0,
    restriction_column_width: 20.0,
    icon_size: 14.0,
    icon_size_sm: 13.0,
    // The mark asset is cropped to its art, so this is the size of the mark
    // itself, not of padding around it. At 28 px it crowded the 34 px menu
    // cap; four fifths of that leaves it room above and below and still reads
    // as the head of the bar.
    brand_size: 16.0,
    gizmo_button_size: 26.0,
    gizmo_icon_size: 17.0,
    object_icon_size: 28.0,
    object_glyph_size: 24.0,
};

/// A control is the module less its room, and a row IS the module. The room
/// keeps a lit button off the rows either side (Blender's `PNL_HEADER` is 1.25
/// units over a 1-unit button). What this replaced, and why a strip tab is the
/// one heading that is still shorter, is in `tests/a_row_is_one_height.rs`.
pub const HEAD_ROOM: f32 = 4.0;
pub const BUTTON_HEIGHT: f32 = MODULE - HEAD_ROOM;
pub const MENU_ROW: f32 = BUTTON_HEIGHT - HEAD_ROOM;

impl BDimensions {
    /// Every row: [`Self::row_height`], the sum this used to compute itself.
    pub const fn card_head_height(&self) -> f32 {
        self.row_height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BStrokes {
    pub hairline: f32,
    pub border: f32,
    pub grid: f32,
}

pub const DEFAULT_STROKES: BStrokes = BStrokes { hairline: 1.0, border: 1.0, grid: 1.0 };

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BResizeStyle {
    /// The seam is this much thinner than the gap at rest…
    pub seam_rest_under: f32,
    /// …and this much thicker than it under the pointer.
    pub seam_hover_over: f32,
    pub hit_width: f32,
    pub gap: f32,
    pub inset: f32,
    pub snap_threshold: f32,
    pub min_extent: f32,
}

// The gap between stacked slots (Blender's `RGN_ALIGN_*` seam), one value
// for every seam — zone, area, panel, modal — and the transparent drag hit
// band over it, wide enough to grab without pixel-hunting.
const RESIZE_GAP: f32 = 4.0;
const RESIZE_HIT_WIDTH: f32 = 14.0;

impl BResizeStyle {
    /// The resize style for a theme whose seams are `gap` wide.
    pub const fn with_gap(gap: f32) -> Self {
        Self {
            // The seam sits inside the gap at rest (gap − 3) and swells past
            // it under the pointer (gap + 2): a line that answers the hand.
            seam_rest_under: 3.0,
            seam_hover_over: 2.0,
            hit_width: RESIZE_HIT_WIDTH,
            gap,
            // The hit band is drawn inside the lower slot at `.top(inset)`,
            // so centring it on the gap's midpoint (`slot.top − gap/2`)
            // gives `inset = −(hit_width + gap) / 2`: −9 for 14 and 4.
            inset: -(RESIZE_HIT_WIDTH + gap) / 2.0,
            snap_threshold: 16.0,
            min_extent: 60.0,
        }
    }

    /// The seam's thickness at rest: never below a hairline.
    pub const fn rest_thickness(&self) -> f32 {
        let thick = self.gap - self.seam_rest_under;
        if thick < 1.0 { 1.0 } else { thick }
    }

    /// The seam's thickness under the pointer.
    pub const fn hover_thickness(&self) -> f32 {
        let thick = self.gap + self.seam_hover_over;
        let rest = self.rest_thickness();
        if thick < rest { rest } else { thick }
    }
}

pub const DEFAULT_RESIZE: BResizeStyle = BResizeStyle::with_gap(RESIZE_GAP);
