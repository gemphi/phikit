//! Which of the theme's outline colours each object of the scene takes.
//!
//! Blender's outline prepass draws the selected objects alone and writes a
//! two-bit colour class with each one: the active object's, the rest of the
//! selection's, and the one for an object being transformed
//! (`outline_colorid_get`, `overlay_outline_prepass_vert.glsl`). The tracer
//! wants the same thing as one class an instance, in the order it
//! instances them — which is the order the scene list is in
//! (`rendered::scene::build`).
//!
//! The selection the classes are read from has already been through
//! `BGuide::outlined`, so Overlays ▸ Objects ▸ Outline Selected off, or
//! every overlay hidden, arrives here as an empty selection and nothing is
//! outlined.

use crate::v2::scene::BObject3D;
use crate::v2::scene::select::BSelection;
use phiray::trace::Outline;
use std::sync::Arc;

pub(super) trait BOutlined {
    /// One outline class an object: the selection in the select colour and
    /// the active object in the active colour, everything else not
    /// outlined at all. Only a *selected* object is outlined — Blender's
    /// own `if (object_is_selected(ob_ref))` round the prepass — so an
    /// active object nothing selected takes no line either.
    fn outline_class(&self, selected: &BSelection, active: &str) -> Arc<[Outline]>;
}

impl BOutlined for [BObject3D] {
    fn outline_class(&self, selected: &BSelection, active: &str) -> Arc<[Outline]> {
        self.iter()
            .map(|object| match crate::v2::scene::object_name(object) {
                name if !selected.contains(name) => Outline::None,
                name if name == active => Outline::Active,
                _ => Outline::Select,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The classes come back in the scene's own order — the order the
    /// tracer instances them in — and the active object outranks the rest
    /// of the selection it is part of.
    #[test]
    fn the_active_object_outranks_the_selection_it_is_in() {
        let of = |name: &str| BObject3D::Primitive { name: name.into(), shape: crate::v2::scene::BShape::Cube, x: 0.0, y: 0.0 };
        let scene = [of("Cube"), of("Ball"), of("Gem")];
        let selected = BSelection { list: vec!["Gem".into(), "Cube".into()] };
        assert_eq!(&*scene.outline_class(&selected, "Cube"), [Outline::Active, Outline::None, Outline::Select]);
        // Overlay off: `BGuide::outlined` hands over an empty selection,
        // and the active object is not outlined either — Blender's prepass
        // draws selected objects, active or not.
        assert_eq!(&*scene.outline_class(&BSelection::default(), "Cube"), [Outline::None; 3]);
        // Selected but not active: the select colour.
        assert_eq!(&*scene.outline_class(&BSelection { list: vec!["Ball".into()] }, "Cube"), [Outline::None, Outline::Select, Outline::None]);
    }
}
