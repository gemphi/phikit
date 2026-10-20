//! Display mode — `SpaceOutliner.display_mode`.
//!
//! Blender ref: `bl_ui/space_outliner.py` and
//! `editors/space_outliner/`.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BOutlineMode {
    Scenes,
    ViewLayer,
    Sequence,
    LibraryOverrides,
    /// `SO_LIBRARIES` — the "Blender File" display mode.
    Libraries,
    DataApi,
    OrphanData,
    /// What happened — renders and their settings, progress and outcome —
    /// as a tree of key/value pairs (`state::event`). Not one of Blender's:
    /// it is where the status bar's line leads.
    Status,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOutlineModeSpec {
    pub kind: BOutlineMode,
    pub label: &'static str,
}

impl BSpec for BOutlineModeSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

pub static MODES: &[BOutlineModeSpec] = &[
    BOutlineModeSpec {
        kind: BOutlineMode::ViewLayer,
        label: "View Layer",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::Scenes,
        label: "Scenes",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::Libraries,
        label: "Blender File",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::Sequence,
        label: "Video Sequencer",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::LibraryOverrides,
        label: "Library Overrides",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::DataApi,
        label: "Data API",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::OrphanData,
        label: "Orphan Data",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::Status,
        label: "Status",
    },
];

impl BOutlineMode {
    pub fn slug(self) -> &'static str {
        match self {
            BOutlineMode::ViewLayer => "ViewLayer",
            BOutlineMode::Scenes => "Scenes",
            BOutlineMode::Sequence => "Sequence",
            BOutlineMode::LibraryOverrides => "LibraryOverrides",
            BOutlineMode::Libraries => "Libraries",
            BOutlineMode::DataApi => "DataApi",
            BOutlineMode::OrphanData => "OrphanData",
            BOutlineMode::Status => "Status",
        }
    }
    /// Accepts either the compact slug (`"ViewLayer"`) or the UI label
    /// (`"View Layer"`) — the modebar dropdown writes labels.
    pub fn from_slug(s: &str) -> Option<Self> {
        MODES
            .iter()
            .find(|m| m.kind.slug() == s || m.label == s)
            .map(|m| m.kind)
    }
    /// Blender shows the restriction columns only in View Layer and Scenes;
    /// Data API, the Sequencer and Blender File draw none.
    pub fn shows_restrictions(self) -> bool {
        matches!(self, BOutlineMode::ViewLayer | BOutlineMode::Scenes)
    }
}
