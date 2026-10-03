//! Image Texture nodes linked to Principled inputs other than Base Color:
//! Roughness, Metallic and Emission Color. A linked image replaces the
//! socket value (Blender ignores the slider once something is linked).

use crate::blend::file::types::*;

/// Image keys (see `image.rs`) per textured Principled input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendMaps {
    pub rough: Option<String>,
    pub metal: Option<String>,
    pub emit: Option<String>,
}

/// The Principled socket each map reads, in [`BlendMaps`] field order.
pub const MAPPED: [&str; 3] = ["Roughness", "Metallic", "Emission Color"];

impl BlendMaps {
    /// The three keys in field order.
    pub fn each(&self) -> [&Option<String>; 3] {
        [&self.rough, &self.metal, &self.emit]
    }

    fn each_mut(&mut self) -> [&mut Option<String>; 3] {
        [&mut self.rough, &mut self.metal, &mut self.emit]
    }

    pub fn is_empty(&self) -> bool {
        self.each().iter().all(|k| k.is_none())
    }
}

impl BlendFile {
    /// Every Image Texture linked to a mapped Principled input.
    pub(crate) fn shade_maps(&self, ma: &BlendBlock, dna: &DnaType) -> BlendMaps {
        let mut maps = BlendMaps::default();
        for (slot, socket) in maps.each_mut().into_iter().zip(MAPPED) {
            *slot = self.socket_image(ma, dna, socket).flatten();
        }
        maps
    }
}
