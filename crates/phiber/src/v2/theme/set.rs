//! Metric compatibility projection for renderers not yet migrated to
//! `BUiStyleDefaults`. Literal design defaults are owned by `theme::style`.
//! Values are in device-independent pixels (UI_UNIT scale = 1.0).

use super::style::{
    DEFAULT_DIMENSIONS, DEFAULT_RESIZE, DEFAULT_SPACING, DEFAULT_STROKES, DEFAULT_TYPOGRAPHY,
};
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BMetric {
    /// Compact spacing between adjacent widgets (button gap).
    pub gap_xs: f32,
    pub gap_sm: f32,
    pub gap_md: f32,
    pub gap_lg: f32,

    /// Paddings inside a region / pane / panel.
    pub pad_xs: f32,
    pub pad_sm: f32,
    pub pad_md: f32,
    pub pad_lg: f32,

    /// Fixed heights: Blender header row = 26 px (HEADERY), footer = 21 px.
    pub h_header: f32,
    pub h_footer: f32,
    pub h_row: f32,
    pub h_button: f32,
    pub h_scrubbing: f32,    // Region::Scrubbing scrub strip
    pub h_asset_shelf: f32,  // Region::AssetShelf drawer
    pub h_bottom_strip: f32, // Zone::Bottom (Timeline + Statusbar)

    /// Fixed widths (Region and Area).
    pub w_activity: f32, // Phiber Activity Bar (44 px in V1).
    pub w_toolbar: f32,    // Region::Tools (T-panel collapsed).
    pub w_sidebar: f32,    // Region::UI/Tools legacy stub.
    pub w_panel: f32,      // Region::UI / Channels (expanded N-panel).
    pub w_navbar: f32,     // Region::NavBar (Properties `NAV_BAR`).
    pub w_channels: f32,   // Animation channels column narrower variant.
    pub w_tool_props: f32, // Region::ToolProps (T-panel props strip).
    pub w_area_left: f32,  // Area::Left column (Outliner).
    pub w_area_right: f32, // Area::Right column (Properties).

    /// File-browser table columns (Blender `space_file` details columns).
    pub w_col_date: f32, // Date Modified column (fixed).
    pub w_col_size: f32, // Size column (fixed, right-aligned).

    /// Tree indent per depth level (Blender `UI_UNIT_X`).
    pub indent_unit: f32,
    /// Width of one right-aligned restriction-toggle column in the Outliner.
    pub w_restrict_col: f32,

    /// Ordinary border/stroke width. Radius defaults are owned exclusively by
    /// `BUiStyleDefaults` and are intentionally absent from this projection.
    pub stroke: f32,

    /// Font sizes.
    pub font_body: f32,
    pub font_header: f32,
    pub font_caption: f32,

    // Area-resize metrics (Requirement 54, phiber-blender-space-parity).
    pub min_area_extent: f32,
    pub resize_handle_hit_width: f32,
    pub resize_snap_threshold: f32,
    /// Visible thickness of the resize seam stroke painted in the boundary
    /// gutter. Independent of `gap_xs` (the layout gap) so the seam can read
    /// as a bolder grab affordance than the plain zone seams.
    pub resize_seam_thickness: f32,
}

impl BSpec for BMetric {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        "metric"
    }
}

pub static METRICS: BMetric = BMetric {
    gap_xs: DEFAULT_SPACING.gap_xs,
    gap_sm: DEFAULT_SPACING.gap_sm,
    gap_md: DEFAULT_SPACING.gap_md,
    gap_lg: DEFAULT_SPACING.gap_lg,
    pad_xs: DEFAULT_SPACING.pad_xs,
    pad_sm: DEFAULT_SPACING.pad_sm,
    pad_md: DEFAULT_SPACING.pad_md,
    pad_lg: DEFAULT_SPACING.pad_lg,

    h_header: DEFAULT_DIMENSIONS.header_height,
    h_footer: DEFAULT_DIMENSIONS.footer_height,
    h_row: DEFAULT_DIMENSIONS.row_height,
    h_button: DEFAULT_DIMENSIONS.button_height,
    h_scrubbing: DEFAULT_DIMENSIONS.scrubbing_height,
    h_asset_shelf: DEFAULT_DIMENSIONS.asset_shelf_height,
    h_bottom_strip: DEFAULT_DIMENSIONS.bottom_strip_height,

    w_activity: DEFAULT_DIMENSIONS.activity_width,
    w_toolbar: DEFAULT_DIMENSIONS.toolbar_width,
    w_sidebar: DEFAULT_DIMENSIONS.sidebar_width,
    w_panel: DEFAULT_DIMENSIONS.panel_width,
    w_navbar: DEFAULT_DIMENSIONS.navbar_width,
    w_channels: DEFAULT_DIMENSIONS.channels_width,
    w_tool_props: DEFAULT_DIMENSIONS.tool_props_width,
    w_area_left: DEFAULT_DIMENSIONS.area_left_width,
    w_area_right: DEFAULT_DIMENSIONS.area_right_width,
    w_col_date: DEFAULT_DIMENSIONS.file_date_column_width,
    w_col_size: DEFAULT_DIMENSIONS.file_size_column_width,
    indent_unit: DEFAULT_DIMENSIONS.indent_unit,
    w_restrict_col: DEFAULT_DIMENSIONS.restriction_column_width,

    stroke: DEFAULT_STROKES.border,
    font_body: DEFAULT_TYPOGRAPHY.body,
    font_header: DEFAULT_TYPOGRAPHY.header,
    font_caption: DEFAULT_TYPOGRAPHY.caption,

    min_area_extent: DEFAULT_RESIZE.min_extent,
    resize_handle_hit_width: DEFAULT_RESIZE.hit_width,
    resize_snap_threshold: DEFAULT_RESIZE.snap_threshold,
    resize_seam_thickness: DEFAULT_RESIZE.visible_seam,
};
