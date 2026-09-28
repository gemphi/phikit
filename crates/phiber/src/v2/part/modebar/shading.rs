//! Viewport shading radio — the four buttons at the right of Blender's
//! `VIEW3D_HT_header` (`layout.prop(shading, "type", expand=True)`).
//! Writes `ctx.view3d.shading`, which the viewport painter reads
//! (`scene::color`) to choose viewport vs. base colour.

use crate::v2::scene::{BShading, shading_key, view_shading};
use crate::v2::state::BState;
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
