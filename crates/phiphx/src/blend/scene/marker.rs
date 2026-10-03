//! Timeline markers — `Scene.markers`, a list of `TimeMarker { frame,
//! name[64], flag, camera }` (`DNA_scene_types.h`), the ones the Timeline
//! draws along its foot and the Marker menu jumps between.

use crate::blend::file::types::*;
use crate::blend::shade::nodes::read_cstr;

/// One marker: the frame it stands at and its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlendMarker {
    pub frame: i32,
    pub name: String,
}

impl BlendFile {
    /// The scene's markers, in list order.
    pub(crate) fn markers(&self, sc: &BlendBlock, dna: &DnaType) -> Vec<BlendMarker> {
        let first = self.ptr(sc, self.off("Scene", "markers", dna));
        let (frame_o, name_o) = (self.off("TimeMarker", "frame", dna), self.off("TimeMarker", "name[64]", dna));
        self.list(first, self.off("TimeMarker", "*next", dna))
            .into_iter()
            .map(|marker| BlendMarker {
                frame: self.int_field(marker, "TimeMarker", "frame", dna) as i32,
                name: name_o.map(|o| read_cstr(&marker.data, o, 64)).unwrap_or_default(),
            })
            .filter(|_| frame_o.is_some())
            .collect()
    }
}
