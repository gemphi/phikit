//! The Image editor's keys — `SpaceImage` and its `ImageUser`, under
//! `image.*`: the view (`sima->zoom`, `xof / yof`), the render slot, view
//! layer and pass shown (`iuser->layer / pass`), and the display channels.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImageKey {
    /// `sima->zoom`: pixels per image pixel; absent is 1.
    Zoom,
    /// `sima->xof / yof`: the view's offset from the image centre, in pixels.
    PanX,
    PanY,
    /// `iuser->slot` as shown: `Slot 1` … `Slot 8`.
    Slot,
    /// `RenderLayer` name, `ViewLayer` by default.
    ViewLayer,
    /// The render pass shown: `Combined` by default.
    Pass,
    /// `sima->flag & SI_USE_ALPHA / SI_SHOW_ALPHA / SI_SHOW_R…`: the
    /// display channels — `Color and Alpha`, `Color`, `Alpha`, `Red`, `Green`, `Blue`.
    Channels,
    /// `IMAGE_PT_gizmo_display`, still a plain toggle. The overlay one beside
    /// it was a key nothing read; the overlays are `overlay.image.*` now
    /// (`part::filter::overlay`), shared with the UV editor as Blender shares
    /// `SpaceImageEditor.overlay` between the two.
    Gizmo,
}

impl BKey for ImageKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Zoom => "zoom",
            Self::PanX => return BPath::owned(vec!["image", "pan", "x"]),
            Self::PanY => return BPath::owned(vec!["image", "pan", "y"]),
            Self::Slot => "slot",
            Self::ViewLayer => "view_layer",
            Self::Pass => "pass",
            Self::Channels => "channels",
            Self::Gizmo => "gizmo",
        };
        BPath::owned(vec!["image", tail])
    }
}
