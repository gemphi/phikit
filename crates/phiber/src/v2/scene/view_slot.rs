//! One view per 3D Viewport — Blender gives every 3D Viewport its own
//! `RegionView3D`: orbiting, zooming or framing one leaves the others
//! where they were. Each view field (yaw, pitch, zoom, perspective, pan,
//! pivot, focal length) is kept per area under
//! `ctx.view3d.<scope>.<slot>.<field>` and read through the area it is
//! asked for: the area being drawn (`BViewSlot::with`, set by the area
//! renderer around its host), else the one the pointer last moved over
//! (`BViewSlot::hover`, set by the viewport canvas, which the keys and the
//! menus act on), else none — the shared `ctx.view3d.<field>`. An area
//! that never changed its view reads the shared value, which is also what
//! a file's saved view writes (`app/load_ui.rs`), so every viewport opens
//! on the file's view and goes its own way from there.

use crate::v2::state::{BKey as _, BPath, BState, View3dKey};
use std::cell::Cell;

/// A 3D Viewport's place: its workspace and its area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BViewSlot {
    pub scope: &'static str,
    pub slot: &'static str,
}

thread_local! {
    static DRAWING: Cell<Option<BViewSlot>> = const { Cell::new(None) };
    static HOVERED: Cell<Option<BViewSlot>> = const { Cell::new(None) };
}

/// The view fields an area keeps its own copy of.
const FIELDS: [View3dKey; 8] = [
    View3dKey::Yaw,
    View3dKey::Pitch,
    View3dKey::Zoom,
    View3dKey::Perspective,
    View3dKey::PanX,
    View3dKey::PanY,
    View3dKey::Pivot,
    View3dKey::Lens,
];

impl BViewSlot {
    pub const fn new(scope: &'static str, slot: &'static str) -> Self {
        Self { scope, slot }
    }

    /// The view asked about now: the area being drawn, else the one under
    /// the pointer last.
    pub fn current() -> Option<Self> {
        DRAWING.with(Cell::get).or_else(|| HOVERED.with(Cell::get))
    }

    /// Run `f` with this area as the one being drawn.
    pub fn with<R>(self, f: impl FnOnce() -> R) -> R {
        let prev = DRAWING.with(|c| c.replace(Some(self)));
        let out = f();
        DRAWING.with(|c| c.set(prev));
        out
    }

    /// The pointer moved over this area's viewport: the keys act on it.
    pub fn hover(self) {
        HOVERED.with(|c| c.set(Some(self)));
    }

    /// Forget the hovered viewport (a new window, a test).
    pub fn forget_hover() {
        HOVERED.with(|c| c.set(None));
    }

    /// This area's key for `field`.
    pub fn path(self, field: View3dKey) -> BPath {
        match field.field() {
            Some(name) => View3dKey::Slot(self.scope, self.slot, name).path(),
            None => field.path(),
        }
    }

    /// The key a change of `field` goes to: the current area's, else the
    /// shared one.
    pub fn key(field: View3dKey) -> BPath {
        Self::current().map_or_else(|| field.path(), |slot| slot.path(field))
    }

    /// `field` as the current area sees it: its own value, else the shared.
    pub fn get(s: &BState, field: View3dKey) -> Option<&str> {
        Self::current().and_then(|slot| s.get(&slot.path(field))).or_else(|| s.get(field))
    }

    /// Every area's own view dropped, so each reads the shared one again:
    /// a file opens on its saved view in every viewport.
    pub fn reset(s: &mut BState) {
        let names: Vec<&str> = FIELDS.iter().filter_map(|f| f.field()).collect();
        let owned: Vec<String> = s
            .iter_keys()
            .filter(|k| {
                let segs: Vec<&str> = k.split('.').collect();
                segs.len() == 5 && segs[1] == "view3d" && names.contains(&segs[4])
            })
            .cloned()
            .collect();
        for key in owned {
            s.remove_encoded(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_area_keeps_its_own_view_and_reads_the_shared_one_until_then() {
        BViewSlot::forget_hover();
        let (left, centre) = (BViewSlot::new("Animation", "Left"), BViewSlot::new("Animation", "Center"));
        let mut s = BState::new();
        s.set(View3dKey::Yaw, "30");
        centre.with(|| s.set(&BViewSlot::key(View3dKey::Yaw), "120"));
        assert_eq!(left.with(|| BViewSlot::get(&s, View3dKey::Yaw)), Some("30"), "untouched: the shared view");
        assert_eq!(centre.with(|| BViewSlot::get(&s, View3dKey::Yaw)), Some("120"));
        assert_eq!(BViewSlot::get(&s, View3dKey::Yaw), Some("30"), "outside any area: the shared view");
        centre.hover();
        assert_eq!(BViewSlot::get(&s, View3dKey::Yaw), Some("120"), "the keys act on the hovered viewport");
        assert_eq!(left.with(|| BViewSlot::get(&s, View3dKey::Yaw)), Some("30"), "drawing wins over hovering");
        BViewSlot::reset(&mut s);
        assert_eq!(centre.with(|| BViewSlot::get(&s, View3dKey::Yaw)), Some("30"), "a file's view resets them all");
        assert_eq!(BViewSlot::key(View3dKey::Shading), View3dKey::Shading.path(), "not a view field: shared");
        BViewSlot::forget_hover();
    }
}
