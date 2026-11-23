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

/// What fills a mesh whose material has a Volume: how much light a unit
/// of length scatters and how much it stops, per channel, and which way
/// the scattered light leans.
///
/// Cycles' own sums (`kernel/svm/closure.h`). Volume Scatter
/// (`svm_node_closure_volume`): scattering is Color times Density and
/// nothing is absorbed. Volume Absorption: nothing scatters, and what is
/// stopped is one minus Color, times Density. Principled Volume
/// (`svm_node_principled_volume`): scattering is Color times Density, and
/// to it is added `max(1 - Color, 0) * max(1 - sqrt(Absorption Color), 0)`
/// times Density. Emission, the Blackbody inputs and a density read from
/// a grid attribute are not here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendVolume {
    /// Scattering coefficient per unit length (`sigma_s`).
    pub scatter: [f32; 3],
    /// Extinction per unit length: scattering plus absorption (`sigma_t`).
    pub extinction: [f32; 3],
    /// The Henyey-Greenstein lean, -1 back to 1 forward (`g`).
    pub anisotropy: f32,
}

impl BlendVolume {
    /// From a node's Color, Density, Anisotropy and Absorption Color by its
    /// kind; `None` for a node that is no volume closure, or one of no
    /// density.
    pub fn of(kind: &str, color: [f32; 3], density: f32, anisotropy: f32, absorption_color: [f32; 3]) -> Option<Self> {
        let density = density.max(0.0);
        let scatter = color.map(|c| c * density);
        let (scatter, extinction) = match kind {
            "VolumeScatter" => (scatter, scatter),
            "VolumeAbsorption" => ([0.0; 3], color.map(|c| (1.0 - c) * density)),
            "VolumePrincipled" => {
                let stopped: [f32; 3] = std::array::from_fn(|k| (1.0 - color[k]).max(0.0) * (1.0 - absorption_color[k].max(0.0).sqrt()).max(0.0) * density);
                (scatter, std::array::from_fn(|k| scatter[k] + stopped[k]))
            }
            _ => return None,
        };
        (density > 0.0).then_some(Self { scatter, extinction, anisotropy: anisotropy.clamp(-0.99, 0.99) })
    }
}

impl BlendFile {
    /// The volume `ma`'s active output is given, when one closure node is
    /// plugged straight into it. A Mix or Add of volume shaders is not
    /// read: `None`, and the mesh stays empty of fog.
    pub(crate) fn volume_of(&self, ma: &BlendBlock, dna: &DnaType) -> Option<BlendVolume> {
        let nodes = self.shader_nodes(ma, dna);
        let links = self.links(ma, dna);
        let out = self.material_output(&nodes, dna)?;
        let node = self.linked_named(&nodes, &links, out, "Volume", dna)?;
        let kind = self.kind(node, dna);
        let graph = super::eval::BGraph::new(self, ma, dna);
        let value = |name: &str| self.socket_named(node, name, dna).and_then(|s| graph.input_or_default(s, 0));
        let rgb = |name: &str, default: [f32; 3]| value(name).map_or(default, |v| [v.rgba[0], v.rgba[1], v.rgba[2]]);
        let float = |name: &str, default: f32| value(name).map_or(default, |v| v.value());
        // Blender's own defaults for a socket the file predates: Color
        // 0.5 grey (0.8 on the two plain closures), Density 1, no lean,
        // black Absorption Color.
        let plain = kind.strip_prefix("ShaderNode").unwrap_or(&kind) != "VolumePrincipled";
        let grey = if plain { 0.8 } else { 0.5 };
        // A Density a texture drives is that texture times these sums at
        // a density of one (`BlendProcedural::density`, which the tracer
        // reads at each point): its fold to one number would be the
        // texture's middle, which a ramp may well send to nothing.
        let density = match self.surface_procedural(ma, dna).density {
            Some(_) => 1.0,
            None => float("Density", 1.0),
        };
        BlendVolume::of(kind.strip_prefix("ShaderNode").unwrap_or(&kind), rgb("Color", [grey; 3]), density, float("Anisotropy", 0.0), rgb("Absorption Color", [0.0; 3]))
    }
}

/// The surface of a volume-only material: none.
pub(crate) fn no_surface() -> BlendBsdf {
    BlendBsdf { base: [0.0; 4], alpha: 0.0, roughness: 1.0, metallic: 0.0, ..Default::default() }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cycles' three sums, on numbers small enough to check by hand.
    #[test]
    fn the_three_closures_follow_cycles_sums() {
        let c = [0.5, 0.25, 1.0];
        let s = BlendVolume::of("VolumeScatter", c, 2.0, 0.3, [0.0; 3]).expect("scatter");
        assert_eq!((s.scatter, s.extinction, s.anisotropy), ([1.0, 0.5, 2.0], [1.0, 0.5, 2.0], 0.3));
        let a = BlendVolume::of("VolumeAbsorption", c, 2.0, 0.0, [0.0; 3]).expect("absorption");
        assert_eq!((a.scatter, a.extinction), ([0.0; 3], [1.0, 1.5, 0.0]));
        // Black Absorption Color stops all of what the colour does not scatter.
        let p = BlendVolume::of("VolumePrincipled", c, 2.0, 0.0, [0.0; 3]).expect("principled");
        assert_eq!((p.scatter, p.extinction), ([1.0, 0.5, 2.0], [2.0, 2.0, 2.0]));
        // White stops none of it.
        let w = BlendVolume::of("VolumePrincipled", c, 2.0, 0.0, [1.0; 3]).expect("principled");
        assert_eq!(w.extinction, w.scatter);
        assert!(BlendVolume::of("VolumeScatter", c, 0.0, 0.0, [0.0; 3]).is_none(), "no density, no volume");
        assert!(BlendVolume::of("BsdfDiffuse", c, 1.0, 0.0, [0.0; 3]).is_none(), "not a volume closure");
    }

    #[test]
    fn no_surface_lets_every_ray_through() {
        let b = no_surface();
        assert_eq!(b.alpha, 0.0);
        assert_eq!(b.base[3], 0.0);
        assert_eq!(b.emitted(), [0.0; 3], "and gives off nothing");
    }
}
