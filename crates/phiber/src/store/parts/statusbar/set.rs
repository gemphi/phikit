//! Statusbar set - host-to-statusbar association and default items.

use super::StatusItem;
use super::def;
use crate::store::hosts::BHostType;

pub static DEFAULT_ITEMS: &[StatusItem] = &[
    StatusItem {
        idx: def::STATUS_SCENE_ID,
        label: def::STATUS_SCENE_LABEL,
    },
    StatusItem {
        idx: def::STATUS_OBJECTS_ID,
        label: def::STATUS_OBJECTS_LABEL,
    },
    StatusItem {
        idx: def::STATUS_VERTS_ID,
        label: def::STATUS_VERTS_LABEL,
    },
    StatusItem {
        idx: def::STATUS_MEMORY_ID,
        label: def::STATUS_MEMORY_LABEL,
    },
];

/// Status bar items.
pub fn with_idx(_host: BHostType) -> &'static [super::StatusItem] {
    DEFAULT_ITEMS
}
