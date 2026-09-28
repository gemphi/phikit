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

/// Every animation editor draws its channels in its own canvas
/// (`canvas/time/row`), from the file's actions and NLA; no host lists a
/// stand-in any more.
pub fn channels_for(_host: BHostType) -> &'static [BChannelSpec] {
    &[]
}
