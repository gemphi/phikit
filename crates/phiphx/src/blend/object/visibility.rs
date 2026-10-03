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
        assert!(!BlendVisibility::default().any() && object.any());
    }
}
