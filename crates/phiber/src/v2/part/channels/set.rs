//! Channels set — placeholder host-scoped channel names.
//!
//! In the full impl, channels come from the animation data on the active
//! scene/object (Blender ref: `bAnimContext`, `ANIM_animdata_filter`).
//! Until the scene bridge is wired, hosts expose a static placeholder list.

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BChannelSpec {
    pub label: &'static str,
}

impl BSpec for BChannelSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Channels)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

static TIME_PLACEHOLDER: &[BChannelSpec] = &[
    BChannelSpec {
        label: "Cube.location",
    },
    BChannelSpec {
        label: "Cube.rotation",
    },
    BChannelSpec {
        label: "Camera.location",
    },
];

pub fn channels_for(host: BHostType) -> &'static [BChannelSpec] {
    match host {
        BHostType::Time | BHostType::Action | BHostType::Nla | BHostType::Ipo => TIME_PLACEHOLDER,
        _ => &[],
    }
}
