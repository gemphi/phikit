//! Flat or spatial: how a node editor shows its tree, and the header toggle
//! that switches it.
//!
//! Per host, remembered per host (`NodeKey::Mode`), so Shading can be spatial
//! while Compositing stays flat — the way every `SpaceNode` setting is the
//! editor's own in Blender. Absent means flat, which keeps the parity path
//! the default and the one a file opens on.
//!
//! The toggle is a two-button radio at the header's right end, drawn as the
//! 3D viewport's shading radio is (`part/modebar/shading.rs`): a button per
//! mode, the current one pressed. Switching writes one key and nothing else —
//! the camera is derived from the flat view both ways (`camera.rs`), so there
//! is nothing to carry across and the toggle is instant.

use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, BState, NodeKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BNodeMode {
    /// Blender's node editor: cards on a plane. The default.
    #[default]
    Flat,
    /// The same graph as solids over a floor, looked at and left.
    Spatial,
}

impl BNodeMode {
    pub const ALL: [Self; 2] = [Self::Flat, Self::Spatial];

    /// What the store holds.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Spatial => "spatial",
        }
    }

    /// The button's tooltip.
    pub fn label(self) -> &'static str {
        match self {
            Self::Flat => "Flat",
            Self::Spatial => "Spatial",
        }
    }

    /// The button's icon — a square for the plane, a cube for the solids.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Flat => "square",
            Self::Spatial => "cube",
        }
    }

    /// A stored slug back; anything else is flat.
    pub fn parse(raw: &str) -> Self {
        Self::ALL.into_iter().find(|m| m.slug() == raw).unwrap_or_default()
    }

    /// The mode `host` is in.
    pub fn of(state: &BState, host: BHostType) -> Self {
        state.get(NodeKey::Mode(host)).map_or(Self::Flat, Self::parse)
    }

    /// The header radio for `host`: one button per mode, the current one
    /// pressed, each writing the mode it stands for.
    pub fn radio(state: &BState, host: BHostType) -> BView {
        let active = Self::of(state, host);
        BView::row(
            Self::ALL
                .iter()
                .map(|mode| {
                    BView::Button(
                        BControl::new()
                            .with_icon(mode.icon())
                            .with_tooltip(mode.label())
                            .with_active(*mode == active)
                            .with_action(BAction::Set(NodeKey::Mode(host).path(), mode.slug().to_string())),
                    )
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_editor_opens_flat_and_remembers_spatial_per_host() {
        let mut s = BState::default();
        assert_eq!(BNodeMode::of(&s, BHostType::Shader), BNodeMode::Flat, "the parity path is the default");
        s.set(NodeKey::Mode(BHostType::Shader), BNodeMode::Spatial.slug());
        assert_eq!(BNodeMode::of(&s, BHostType::Shader), BNodeMode::Spatial);
        assert_eq!(BNodeMode::of(&s, BHostType::Compositor), BNodeMode::Flat, "another editor keeps its own mode");
    }

    #[test]
    fn an_unreadable_mode_is_flat() {
        assert_eq!(BNodeMode::parse("sideways"), BNodeMode::Flat);
        for mode in BNodeMode::ALL {
            assert_eq!(BNodeMode::parse(mode.slug()), mode);
        }
    }
}
