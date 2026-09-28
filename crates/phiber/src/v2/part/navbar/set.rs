//! NavBar set — Properties context tabs (Blender 5.2 has 15 with Collection).
//! Blender ref: `editors/space_buttons/buttons_context.cc` context list.

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BNavTabSpec {
    pub icon: &'static str,
    pub label: &'static str,
}

impl BSpec for BNavTabSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::NavBar)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

static BUTS_TABS: &[BNavTabSpec] = &[
    BNavTabSpec {
        icon: "tool",
        label: "Active Tool",
    },
    BNavTabSpec {
        icon: "render",
        label: "Render",
    },
    BNavTabSpec {
        icon: "output",
        label: "Output",
    },
    BNavTabSpec {
        icon: "render_layer",
        label: "View Layer",
    },
    BNavTabSpec {
        icon: "scene",
        label: "Scene",
    },
    BNavTabSpec {
        icon: "world",
        label: "World",
    },
    BNavTabSpec {
        icon: "folder",
        label: "Collection",
    },
    BNavTabSpec {
        icon: "object",
        label: "Object",
    },
    BNavTabSpec {
        icon: "modifier",
        label: "Modifiers",
    },
    BNavTabSpec {
        icon: "particle",
        label: "Particles",
    },
    BNavTabSpec {
        icon: "physics",
        label: "Physics",
    },
    BNavTabSpec {
        icon: "constraint",
        label: "Constraints",
    },
    BNavTabSpec {
        icon: "data",
        label: "Data",
    },
    BNavTabSpec {
        icon: "material",
        label: "Material",
    },
    BNavTabSpec {
        icon: "texture",
        label: "Texture",
    },
];

pub fn tabs_for(host: BHostType) -> &'static [BNavTabSpec] {
    match host {
        BHostType::Buts => BUTS_TABS,
        // Preferences has a nav bar for the same reason Properties does: a
        // column of contexts, one of which is showing. Its tabs come from the
        // settings categories so the two can never drift apart.
        BHostType::Prefs => crate::v2::modal::setting::NAV_TABS,
        _ => &[],
    }
}
