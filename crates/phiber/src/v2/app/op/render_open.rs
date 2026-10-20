//! Where a render is shown — Blender's `render_view_open`
//! (`editors/render/render_view.cc`) for Preferences ▸ Interface ▸ Editors
//! ▸ Render In: **Image Editor**.
//!
//! The render goes into the workspace on screen rather than over it: the
//! Image editor already showing the Render Result (`find_area_showing_
//! render_result`); else any Image editor (`find_area_image_empty`); else
//! the largest other area becomes one (`biggest_non_image_area`, the
//! Center here), remembering the editor it was (`SI_PREVSPACE`) so Escape
//! or F11 puts it back (`ED_area_prevspace`). The Properties down the
//! right and the Timeline along the bottom stay where they are, as in
//! Blender's Rendering workspace — the panes the render settings and its
//! status live in.

use crate::v2::area::current::{active_host, area_key};
use crate::v2::area::BAreaType;
use crate::v2::host::BHostType;
use crate::v2::part::image::shown::{BShownImage, RENDER_RESULT};
use crate::v2::space::BSpaceList;
use crate::v2::state::{BState, ImageKey, ModalKey, log, val::{V_EMPTY, V_MODAL_RENDER}};

/// Show the Render Result in an Image editor of the workspace on screen.
pub fn render_view_open(state: &mut BState) {
    // The render window, if it was up, is the other display — not both.
    if state.get(ModalKey::Open) == Some(V_MODAL_RENDER) {
        state.set(ModalKey::Open, V_EMPTY);
    }
    let space = BSpaceList::active(state);
    let areas = space.spec().areas;
    let host_of = |state: &BState, a| active_host(space, a, |k| state.get(k).map(str::to_string));
    let image = areas.iter().find(|a| a.hosts.contains(&BHostType::Image) && host_of(state, a) == BHostType::Image);
    if image.is_none() {
        // `biggest_non_image_area`: the Center flexes, so it is the largest.
        let Some(center) = areas.iter().find(|a| a.kind == BAreaType::Center && a.hosts.contains(&BHostType::Image)) else {
            log::push("Render: no area of this workspace can show an image".to_string());
            return;
        };
        let was = host_of(state, center);
        let key = area_key(space, center);
        state.set(ImageKey::PrevSpace, format!("{}={}", key.encode(), was.slug()));
        state.set(&key, BHostType::Image.slug());
    }
    // `sima->image = Render Result`.
    if BShownImage::now(state) != BShownImage::RenderResult {
        state.set(ImageKey::Shown, RENDER_RESULT);
    }
}

/// `ED_area_prevspace`: the area F12 took gives its editor back. `false`
/// when no area was taken.
pub fn render_view_close(state: &mut BState) -> bool {
    let Some(prev) = state.get(ImageKey::PrevSpace).map(str::to_string) else { return false };
    state.remove(ImageKey::PrevSpace);
    let Some((key, host)) = prev.rsplit_once('=') else { return false };
    state.set_encoded(key, host.to_string());
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::space::BSpaceType;
    use crate::v2::state::CtxKey;

    fn center_host(state: &BState, space: BSpaceType) -> BHostType {
        let center = space.spec().areas.iter().find(|a| a.kind == BAreaType::Center).unwrap();
        active_host(space, center, |k| state.get(k).map(str::to_string))
    }

    #[test]
    fn layout_lends_its_viewport_to_the_render_and_gets_it_back() {
        let mut s = BState::new();
        s.set(CtxKey::Space, BSpaceType::Layout.slug());
        let before = center_host(&s, BSpaceType::Layout);
        render_view_open(&mut s);
        assert_eq!(center_host(&s, BSpaceType::Layout), BHostType::Image, "the render is in the workspace");
        assert_eq!(BShownImage::now(&s), BShownImage::RenderResult);
        assert!(render_view_close(&mut s));
        assert_eq!(center_host(&s, BSpaceType::Layout), before, "Escape gives the viewport back");
        assert!(!render_view_close(&mut s), "nothing left to give back");
    }

    #[test]
    fn rendering_already_shows_an_image_editor_and_keeps_it() {
        let mut s = BState::new();
        s.set(CtxKey::Space, BSpaceType::Rendering.slug());
        render_view_open(&mut s);
        assert_eq!(center_host(&s, BSpaceType::Rendering), BHostType::Image);
        assert!(s.get(ImageKey::PrevSpace).is_none(), "no area was taken");
    }
}
