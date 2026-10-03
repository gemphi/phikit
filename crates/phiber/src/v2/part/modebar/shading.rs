//! Viewport shading radio — the four buttons at the right of Blender's
//! `VIEW3D_HT_header` (`layout.prop(shading, "type", expand=True)`).
//! Writes `ctx.view3d.shading`, which the viewport painter reads
//! (`scene::color`) to choose viewport vs. base colour.

use crate::v2::scene::{BShading, BXray, shading_key, view_shading};
use crate::v2::state::{BKey as _, BState, OpKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

pub fn view(s: &BState) -> BView {
    let active = view_shading(s);
    BView::row(
        BShading::ALL
            .iter()
            .map(|mode| {
                // A header toggle is a button, as the switcher beside it is.
                BView::Button(
                    BControl::new()
                        .with_icon(mode.icon())
                        .with_tooltip(mode.label())
                        .with_active(*mode == active)
                        .with_action(BAction::Set(shading_key(), mode.slug().to_string())),
                )
            })
            .collect(),
    )
}

/// The X-ray button before the radio (`view3d.toggle_xray`, Alt Z): pressed
/// while the shading in use has X-ray on, dimmed where X-ray cannot show —
/// Material Preview and Rendered, outside Edit Mode (`row.active`).
pub struct BXrayButton;

impl BXrayButton {
    pub fn view(s: &BState) -> BView {
        let shading = view_shading(s);
        let editing = crate::v2::part::modebar::active_mode(s, crate::v2::host::BHostType::View3D) == "Edit Mode";
        BView::Button(
            BControl::new()
                .with_icon("xray")
                .with_tooltip("Toggle X-Ray")
                .with_active(BXray::now(s).on(shading))
                .with_muted(!editing && !BXray::available(shading))
                .with_action(BAction::Set(OpKey::View3d.path(), "toggle_xray".to_string())),
        )
    }
}
