//! A material that is a volume and nothing else.
//!
//! Cycles draws no surface where the Material Output's Surface is not
//! linked: a mesh carrying only a Volume shader is the boundary of its
//! fog, invisible itself and shadowing nothing (`shader.cpp` leaves
//! `has_surface` false; the object is then a volume object only). The
//! renderer draws no volumes yet, and without this rule such a mesh came
//! out with the first Principled in the tree, or the legacy grey — opaque.
//! CRYSTAL-CAVES-CITY wraps the whole city in one, a 275 m "Fog" box, and
//! rendered black inside it.
//!
//! Until volumes are drawn the right surface for it is none: alpha zero,
//! which the tracer lets camera and shadow rays through.

use crate::blend::file::types::*;
use crate::blend::scene::types::BlendBsdf;

impl BlendFile {
    /// Whether `ma`'s active output has a Volume linked and no Surface.
    pub(crate) fn volume_alone(&self, ma: &BlendBlock, dna: &DnaType) -> bool {
        let nodes = self.shader_nodes(ma, dna);
        let links = self.links(ma, dna);
        let Some(out) = self.material_output(&nodes, dna) else { return false };
        self.linked_named(&nodes, &links, out, "Surface", dna).is_none() && self.linked_named(&nodes, &links, out, "Volume", dna).is_some()
    }
}

/// The surface of a volume-only material: none.
pub(crate) fn no_surface() -> BlendBsdf {
    BlendBsdf { base: [0.0; 4], alpha: 0.0, roughness: 1.0, metallic: 0.0, ..Default::default() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_surface_lets_every_ray_through() {
        let b = no_surface();
        assert_eq!(b.alpha, 0.0);
        assert_eq!(b.base[3], 0.0);
        assert_eq!(b.emitted(), [0.0; 3], "and gives off nothing");
    }
}
