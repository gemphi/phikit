//! Switcher metadata — categories classify the existing per-area host catalog.
//! Candidate hosts still come from `BAreaSpec::hosts` (or `SWITCHABLE` as a
//! fallback); this module never carries a second editor catalog.

use crate::v2::host::BHostType;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BHostCategory {
    General,
    Animation,
    Scripting,
    Data,
}

impl BHostCategory {
    pub const ALL: &'static [Self] = &[Self::General, Self::Animation, Self::Scripting, Self::Data];

    pub const fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Animation => "Animation",
            Self::Scripting => "Scripting",
            Self::Data => "Data",
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Self::General => "layout_grid",
            Self::Animation => "film",
            Self::Scripting => "code",
            Self::Data => "database",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Animation => "animation",
            Self::Scripting => "scripting",
            Self::Data => "data",
        }
    }
}

pub fn hosts() -> &'static [BHostType] {
    BHostType::SWITCHABLE
}

/// Classify host metadata without duplicating the authoritative host catalog.
pub const fn category(host: BHostType) -> BHostCategory {
    match host {
        BHostType::Time
        | BHostType::Action
        | BHostType::Nla
        | BHostType::Seq
        | BHostType::Drivers
        | BHostType::Ipo => BHostCategory::Animation,
        BHostType::Text | BHostType::Info | BHostType::Console => BHostCategory::Scripting,
        BHostType::Outliner
        | BHostType::Buts
        | BHostType::File
        | BHostType::Node
        | BHostType::Logic
        | BHostType::Compositor
        | BHostType::Shader
        | BHostType::GeoNode
        | BHostType::Clip
        | BHostType::Asset
        | BHostType::Spreadsheet
        | BHostType::Prefs => BHostCategory::Data,
        _ => BHostCategory::General,
    }
}

pub fn category_key(switcher: &BPath, category: BHostCategory) -> BPath {
    let mut segments = Vec::with_capacity(switcher.segs.len() + 1);
    segments.push("switcher_category");
    segments.extend(switcher.segs.iter().skip(1).copied());
    segments.push(category.slug());
    BPath::owned(segments)
}
