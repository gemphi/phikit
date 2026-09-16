//! Asset Shelf set — brush thumbnails / paint texture shelf.
//! Blender ref: `RGN_TYPE_ASSET_SHELF`. Populated per host+mode.

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

static VIEW3D_SCULPT: &[BAssetTileSpec] = &[
    t("brush-draw", "Draw"),
    t("brush-clay", "Clay Strips"),
    t("brush-smooth", "Smooth"),
    t("brush-inflate", "Inflate"),
    t("brush-grab", "Grab"),
    t("brush-mask", "Mask"),
];

pub fn tiles_for(host: BHostType) -> &'static [BAssetTileSpec] {
    match host {
        BHostType::View3D => VIEW3D_SCULPT,
        BHostType::Image => VIEW3D_SCULPT, // paint-mode brushes share defaults
        _ => &[],
    }
}
