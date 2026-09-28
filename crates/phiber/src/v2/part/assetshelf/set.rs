//! Asset Shelf set — the brushes a paint or sculpt mode offers.
//! Blender ref: `RGN_TYPE_ASSET_SHELF`, `asset_shelf_poll` per space type.
//!
//! Blender's shelf is not a permanent band at the foot of an editor: it is
//! polled per mode, and a space that is not in a painting mode has no shelf
//! at all. Declaring it unconditionally is what left the Image Editor with a
//! row of brushes and an empty strip under it in View mode.

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAssetTileSpec {
    pub thumb: &'static str,
    pub label: &'static str,
}

impl BSpec for BAssetTileSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::AssetShelf)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.thumb
    }
}

const fn t(thumb: &'static str, label: &'static str) -> BAssetTileSpec {
    BAssetTileSpec { thumb, label }
}

/// `sculpt_tool` defaults (`bl_ui/properties_paint_common.py`).
static SCULPT: &[BAssetTileSpec] = &[
    t("brush_draw", "Draw"),
    t("brush_clay", "Clay Strips"),
    t("brush_smooth", "Smooth"),
    t("brush_inflate", "Inflate"),
    t("brush_grab", "Grab"),
    t("brush_mask", "Mask"),
];

/// `image_tool` defaults — the texture / image paint brushes.
static PAINT: &[BAssetTileSpec] = &[
    t("brush_draw", "Draw"),
    t("brush_soften", "Soften"),
    t("brush_smear", "Smear"),
    t("brush_fill", "Fill"),
    t("brush_clone", "Clone"),
    t("brush_mask", "Mask"),
];

/// `vertex_tool` / `weight_tool` defaults.
static VERTEX: &[BAssetTileSpec] = &[
    t("brush_draw", "Draw"),
    t("brush_smooth", "Blur"),
    t("brush_smear", "Smear"),
    t("brush_inflate", "Average"),
];

/// The brushes this host offers in this mode. Empty means no shelf: a
/// viewport in Object Mode, or an Image Editor showing an image.
pub fn tiles_for(host: BHostType, mode: &str) -> &'static [BAssetTileSpec] {
    match (host, mode) {
        (BHostType::View3D, "Sculpt Mode") => SCULPT,
        (BHostType::View3D, "Texture Paint") => PAINT,
        (BHostType::View3D, "Vertex Paint" | "Weight Paint") => VERTEX,
        (BHostType::Image | BHostType::Uv, "Paint") => PAINT,
        _ => &[],
    }
}

/// Whether this host, in this mode, has a shelf at all.
pub fn has_shelf(host: BHostType, mode: &str) -> bool {
    !tiles_for(host, mode).is_empty()
}
