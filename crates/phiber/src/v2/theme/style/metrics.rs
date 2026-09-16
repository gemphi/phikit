//! Typography / spacing / dimensions / strokes / resize style plus their
//! defaults. Split out of `style.rs` so the parent stays under the 200-line
//! cap.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BTypography {
    pub body: f32,
    pub header: f32,
    pub caption: f32,
}

pub const DEFAULT_TYPOGRAPHY: BTypography = BTypography {
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
    pub button_height: f32,
    pub scrubbing_height: f32,
    pub asset_shelf_height: f32,
    pub bottom_strip_height: f32,
    pub activity_width: f32,
    pub toolbar_width: f32,
    pub sidebar_width: f32,
    pub sidebar_min_width: f32,
    pub panel_width: f32,
    pub navbar_width: f32,
    pub channels_width: f32,
    pub tool_props_width: f32,
    pub area_left_width: f32,
    pub area_right_width: f32,
    /// Default width for a Left/Right zone when the user has not dragged an
    /// explicit `zone_size`. Prevents a greedy host from starving Center.
    pub side_zone_default_width: f32,
    pub file_date_column_width: f32,
    pub file_size_column_width: f32,
    pub indent_unit: f32,
    pub restriction_column_width: f32,
    pub gizmo_button_size: f32,
    pub gizmo_icon_size: f32,
    pub object_icon_size: f32,
    pub object_glyph_size: f32,
}

pub const DEFAULT_DIMENSIONS: BDimensions = BDimensions {
    header_height: 30.0,
    footer_height: 24.0,
    row_height: 26.0,
    button_height: 26.0,
    scrubbing_height: 24.0,
    asset_shelf_height: 120.0,
    bottom_strip_height: 80.0,
    activity_width: 44.0,
    toolbar_width: 44.0,
    sidebar_width: 44.0,
    sidebar_min_width: 160.0,
    panel_width: 240.0,
    navbar_width: 32.0,
    channels_width: 200.0,
    tool_props_width: 200.0,
    area_left_width: 156.0,
    area_right_width: 196.0,
    side_zone_default_width: 280.0,
    file_date_column_width: 140.0,
    file_size_column_width: 90.0,
    indent_unit: 20.0,
    restriction_column_width: 20.0,
    gizmo_button_size: 26.0,
    gizmo_icon_size: 15.0,
    object_icon_size: 28.0,
    object_glyph_size: 24.0,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BStrokes {
    pub hairline: f32,
    pub border: f32,
    pub grid: f32,
}

pub const DEFAULT_STROKES: BStrokes = BStrokes {
    hairline: 1.0,
    border: 1.0,
    grid: 1.0,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BResizeStyle {
    pub visible_seam: f32,
    pub hit_width: f32,
    pub gap: f32,
    pub inset: f32,
    pub snap_threshold: f32,
    pub min_extent: f32,
}

// Gap between stacked slots (Blender's `RGN_ALIGN_TOP` / `RGN_ALIGN_BOTTOM`
// slots share this hairline seam). 4 px feels close to Blender and reads as
// deliberate separation.
const RESIZE_GAP: f32 = 4.0;
// Width of the transparent drag hit target painted over the gap. Wider than
// the visible seam so the user can grab the boundary without pixel-hunting.
const RESIZE_HIT_WIDTH: f32 = 14.0;

pub const DEFAULT_RESIZE: BResizeStyle = BResizeStyle {
    // Visible seam is a 2px line — one step stronger than the hairline
    // border so the resize boundary reads as a deliberate affordance while
    // the wide transparent hit band (below) keeps it easy to grab.
    visible_seam: 2.0,
    hit_width: RESIZE_HIT_WIDTH,
    gap: RESIZE_GAP,
    // The hit-band overlay is rendered inside the LOWER slot with
    // `.top(inset).h(hit_width)`, so its centre lands at
    //   slot.top + inset + hit_width/2
    // The gap sits between the previous slot's bottom and this slot's top,
    // so the gap midpoint is  slot.top - gap/2.  Solving for centre:
    //   slot.top + inset + hit_width/2 = slot.top - gap/2
    //   inset = -(hit_width + gap) / 2
    // With hit_width=14 and gap=4  →  inset = -9. This corrects the
    // pre-existing 2 px offset that made the hit band centre trail below
    // the visible seam.
    inset: -(RESIZE_HIT_WIDTH + RESIZE_GAP) / 2.0,
    snap_threshold: 16.0,
    min_extent: 60.0,
};
