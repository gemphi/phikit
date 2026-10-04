//! Restriction flags — the toggles Blender's Outliner draws at a row's right
//! edge, as the file stores them.
//!
//! Blender ref (`DNA_object_types.h`, `eObjectVisibilityFlag`):
//! `Object.visibility_flag` holds `OB_HIDE_VIEWPORT = 1<<0` (the monitor:
//! disable in viewports), `OB_HIDE_SELECT = 1<<1` (the arrow: not
//! selectable) and `OB_HIDE_RENDER = 1<<2` (the camera: disable in renders).
//! That is RNA's name; the DNA member is still spelt `restrictflag` in
//! every file so far (`OB_RESTRICT_VIEW / SELECT / RENDER` before 2.80,
//! the same bits), so the reader takes whichever the file's DNA has.
//! `DNA_collection_types.h`:
//! `Collection.flag` holds `COLLECTION_HIDE_VIEWPORT = 1<<0`,
//! `COLLECTION_HIDE_SELECT = 1<<1` and `COLLECTION_HIDE_RENDER = 1<<3`.
//! The eye is not here: it is the view layer's `BASE_HIDDEN` /
//! `LAYER_COLLECTION_HIDE` (`layer.rs`), per layer rather than per datablock.

use crate::blend::file::types::*;

pub const OB_HIDE_VIEWPORT: u32 = 1 << 0;
pub const OB_HIDE_SELECT: u32 = 1 << 1;
pub const OB_HIDE_RENDER: u32 = 1 << 2;
pub const COLLECTION_HIDE_VIEWPORT: u32 = 1 << 0;
pub const COLLECTION_HIDE_SELECT: u32 = 1 << 1;
pub const COLLECTION_HIDE_RENDER: u32 = 1 << 3;

/// The three restriction toggles of an object or a collection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlendVisibility {
    /// Disabled in viewports (the monitor).
    pub hide_viewport: bool,
    /// Not selectable (the arrow).
    pub hide_select: bool,
    /// Disabled in renders (the camera).
    pub hide_render: bool,
}

impl BlendVisibility {
    pub fn of_object(flag: u32) -> Self {
        Self {
            hide_viewport: flag & OB_HIDE_VIEWPORT != 0,
            hide_select: flag & OB_HIDE_SELECT != 0,
            hide_render: flag & OB_HIDE_RENDER != 0,
        }
    }

    pub fn of_collection(flag: u32) -> Self {
        Self {
            hide_viewport: flag & COLLECTION_HIDE_VIEWPORT != 0,
            hide_select: flag & COLLECTION_HIDE_SELECT != 0,
            hide_render: flag & COLLECTION_HIDE_RENDER != 0,
        }
    }

    pub fn any(&self) -> bool {
        self.hide_viewport || self.hide_select || self.hide_render
    }
}

/// Whether an object casts a shadow: Viewport Display ▸ Shadow
/// (`Object.dtx`'s `OB_DRAW_NO_SHADOW_CAST`, Workbench's) and Ray
/// Visibility ▸ Shadow (`Object.visibility_flag`'s `OB_HIDE_SHADOW`,
/// Cycles' and EEVEE's). Both on by default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlendShadowCast {
    pub display: bool,
    pub ray: bool,
}

impl Default for BlendShadowCast {
    fn default() -> Self {
        Self { display: true, ray: true }
    }
}

impl BlendShadowCast {
    pub const OB_DRAW_NO_SHADOW_CAST: u32 = 1 << 9;
    pub const OB_HIDE_SHADOW: u32 = 1 << 8;

    pub fn of_object(dtx: u32, visibility_flag: u32) -> Self {
        Self { display: dtx & Self::OB_DRAW_NO_SHADOW_CAST == 0, ray: visibility_flag & Self::OB_HIDE_SHADOW == 0 }
    }

    /// Whether the object casts a shadow in a viewport that draws display
    /// shading (Solid, Wireframe: Viewport Display's toggle) or, else, in
    /// a ray-traced one (Material Preview, Rendered: Visibility ▸ Ray's).
    pub fn casts_in(self, display_shading: bool) -> bool {
        if display_shading {
            self.display
        } else {
            self.ray
        }
    }
}

impl BlendFile {
    /// The DNA member that holds the object's toggles in this file:
    /// `visibility_flag` where a file spells it so, else `restrictflag`.
    pub(crate) fn object_visibility_field(&self, dna: &DnaType) -> &'static str {
        if self.off("Object", "visibility_flag", dna).is_some() { "visibility_flag" } else { "restrictflag" }
    }

    /// `Object.visibility_flag` / `Object.restrictflag`: the same three bits.
    pub(crate) fn object_visibility(&self, block: &BlendBlock, dna: &DnaType) -> BlendVisibility {
        let field = self.object_visibility_field(dna);
        BlendVisibility::of_object(self.int_field(block, "Object", field, dna))
    }

    /// `Object.dtx` and `Object.visibility_flag`'s shadow bits.
    pub(crate) fn object_shadow(&self, block: &BlendBlock, dna: &DnaType) -> BlendShadowCast {
        let field = self.object_visibility_field(dna);
        BlendShadowCast::of_object(self.int_field(block, "Object", "dtx", dna), self.int_field(block, "Object", field, dna))
    }

    /// `Object.col`: the object colour, white when unreadable.
    pub(crate) fn object_color(&self, block: &BlendBlock, dna: &DnaType) -> [f32; 4] {
        match self.off("Object", "col[4]", dna) {
            Some(o) if o + 16 <= block.data.len() => std::array::from_fn(|i| self.read_f32(&block.data[o + 4 * i..o + 4 * i + 4])),
            _ => [1.0; 4],
        }
    }

    /// `Collection.flag`.
    pub(crate) fn collection_visibility(&self, block: &BlendBlock, dna: &DnaType) -> BlendVisibility {
        BlendVisibility::of_collection(self.int_field(block, "Collection", "flag", dna))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bits_are_blenders() {
        let object = BlendVisibility::of_object(OB_HIDE_SELECT | OB_HIDE_RENDER);
        assert_eq!(object, BlendVisibility { hide_viewport: false, hide_select: true, hide_render: true });
        let collection = BlendVisibility::of_collection(COLLECTION_HIDE_RENDER | 1 << 2);
        assert_eq!(collection, BlendVisibility { hide_viewport: false, hide_select: false, hide_render: true }, "1<<2 is the retired bit");
        assert_eq!(BlendShadowCast::of_object(0, 0), BlendShadowCast::default(), "casts both ways by default");
        assert_eq!(BlendShadowCast::of_object(1 << 9, 1 << 8), BlendShadowCast { display: false, ray: false });
        assert!(!BlendVisibility::default().any() && object.any());
    }
}
