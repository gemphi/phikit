//! A mesh's active UV map as the UV editor draws it: every face's corners'
//! UVs, face after face, in the mesh's own corner order (`MLoopUV` /
//! the `CD_PROP_FLOAT2` corner attribute) — the base mesh, as Blender's UV
//! editor shows it, not the evaluated one. Faces stay faces: a seam keeps
//! its two sides, which the tracer's per-vertex UVs cannot.

/// One object's UV map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneUvMap {
    /// `starts[f] .. starts[f + 1]` is face `f`'s run of `uvs`.
    pub starts: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
}

impl SceneUvMap {
    /// The map of `obj`, or an empty one when it carries no UVs (or not
    /// one per corner).
    pub fn of(obj: &crate::blend::BlendSceneObject) -> Self {
        if obj.loop_uvs.is_empty() || obj.loop_uvs.len() != obj.faces.corners() {
            return Self::default();
        }
        let mut starts = Vec::with_capacity(obj.faces.len() + 1);
        let mut at = 0u32;
        starts.push(0);
        for face in obj.faces.iter() {
            at += face.len() as u32;
            starts.push(at);
        }
        Self { starts, uvs: obj.loop_uvs.clone() }
    }

    pub fn is_empty(&self) -> bool {
        self.uvs.is_empty()
    }

    /// Every face's UVs, in order.
    pub fn faces(&self) -> impl Iterator<Item = &[[f32; 2]]> + '_ {
        self.starts.windows(2).map(|w| self.uvs.get(w[0] as usize..w[1] as usize).unwrap_or(&[]))
    }
}
