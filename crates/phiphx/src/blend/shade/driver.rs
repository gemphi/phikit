//! Drivers on a material — what they write, and the clamp each write
//! takes. Reading them out of the file is [`driver_read`](super::driver_read);
//! this is what one means.
//!
//! Blender keeps a material's drivers on **two** IDs. A driver on the
//! material's own property (`roughness`, `diffuse_color`) lives on
//! `Material.adt`; a driver on a shader socket lives on
//! `Material.nodetree->adt`, pathed `nodes["<name>"].inputs[<i>].default_value`
//! with the component in the curve's `array_index`. Both land here.
//!
//! The clamps are the part worth stating, because the three disagree and
//! Blender's own evaluated values are the only way to know it:
//!
//! | written through | below range | above range |
//! |---|---|---|
//! | a float or vector socket | **kept** | **kept** |
//! | an RGBA socket's component | clamped to 0 | kept |
//! | the Material's own float property | clamped to its RNA hard min | clamped to its hard max |
//!
//! A socket's declared RNA range is therefore *not* what a driver obeys:
//! `Roughness` says 0..1 and takes -1 from a driver without complaint,
//! while `Material.roughness` says the same and clamps. The reason is the
//! write path — `rna_NodeSocketStandard_value_set` writes a float socket
//! straight through, where a plain float property goes through
//! `RNA_property_float_set`'s clamp.

use crate::blend::anim::driver::{BlendDriver, BlendDriverScene, socket_clamp};
use crate::blend::scene::types::{BlendBsdf, BlendMaterial};

/// A property of the `Material` ID itself that a driver may write, with the
/// RNA hard range its write is clamped to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BMatProp {
    /// `Material.roughness` — what Solid mode shades with, hard 0..1.
    Roughness,
    /// `Material.metallic`, hard 0..1.
    Metallic,
    /// `Material.specular_intensity`, hard 0..1. Read for the Drivers
    /// editor and clamped like the rest; [`BlendMaterial`] has no field for
    /// it, so [`BMaterialDrivers::apply`] writes nothing for it.
    SpecularIntensity,
    /// `Material.diffuse_color[c]` — the `r/g/b/a` fields, hard 0..∞.
    DiffuseColor(usize),
}

impl BMatProp {
    /// The RNA path and component a driver's curve names, as `None` for a
    /// path this does not model.
    pub fn of(path: &str, index: usize) -> Option<Self> {
        match path {
            "roughness" => Some(Self::Roughness),
            "metallic" => Some(Self::Metallic),
            "specular_intensity" => Some(Self::SpecularIntensity),
            "diffuse_color" if index < 4 => Some(Self::DiffuseColor(index)),
            _ => None,
        }
    }

    /// `(hard_min, hard_max)` — what `RNA_property_float_set` clamps to.
    pub fn range(self) -> (f32, f32) {
        match self {
            Self::DiffuseColor(_) => (0.0, f32::MAX),
            _ => (0.0, 1.0),
        }
    }
}

/// What one driver on a material writes.
#[derive(Clone, Debug, PartialEq)]
pub enum BMatWrite {
    /// A property of the `Material` ID.
    Own(BMatProp),
    /// A node socket's `default_value`: the node's name as the path spells
    /// it, the socket's index in that node's input list, and the component.
    Socket { node: String, input: usize, component: usize },
}

/// One input socket as the file stores it. The material's drivers and
/// every other datablock's read this the same way, so the type is the
/// shared one.
pub use crate::blend::anim::driver::BDriverSocket as BMatSocket;

/// One driver on a material, with what it writes.
#[derive(Clone, Debug, PartialEq)]
pub struct BMaterialDriver {
    pub write: BMatWrite,
    pub driver: BlendDriver,
}

/// Every driver one material carries, over both its IDs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BMaterialDrivers {
    pub material: String,
    pub drivers: Vec<BMaterialDriver>,
    /// The input sockets of each node a driver writes, in storage order —
    /// what turns `inputs[2]` into a name the BSDF knows.
    pub inputs: Vec<(String, Vec<BMatSocket>)>,
    /// The name of the node the material's surface reaches, when it is a
    /// Principled BSDF: the only node whose sockets [`Self::apply`] folds
    /// into [`BlendMaterial::bsdf`].
    pub surface: Option<String>,
}

impl BMaterialDrivers {
    pub fn is_empty(&self) -> bool {
        self.drivers.is_empty()
    }

    /// This material's socket list for `node`.
    fn sockets(&self, node: &str) -> &[BMatSocket] {
        self.inputs.iter().find(|(name, _)| name == node).map_or(&[], |(_, list)| list.as_slice())
    }

    /// Every driver's value at `frame`, clamped as Blender clamps it on the
    /// way in — the driven values themselves, before any of them is folded
    /// into a surface. A muted driver is left out, as it writes nothing.
    pub fn values(&self, scene: &BlendDriverScene, frame: f32) -> Vec<(BMatWrite, f32)> {
        self.drivers
            .iter()
            .filter(|d| !d.driver.muted())
            .map(|d| {
                let raw = d.driver.value(scene, frame);
                (d.write.clone(), self.clamp_for(&d.write, raw))
            })
            .collect()
    }

    /// `raw` as the write path leaves it — the table in this module's own
    /// documentation, which is Blender's own evaluated behaviour.
    pub fn clamp_for(&self, write: &BMatWrite, raw: f32) -> f32 {
        match write {
            BMatWrite::Own(prop) => {
                let (lo, hi) = prop.range();
                raw.clamp(lo, hi)
            }
            // A float or vector socket takes the value as it comes; an
            // RGBA socket's component clamps at 0 and is free above 1.
            BMatWrite::Socket { node, input, .. } => socket_clamp(self.sockets(node), *input, raw),
        }
    }

    /// Write this frame's driven values over `material`.
    ///
    /// The material-level properties land where the reader puts them —
    /// `Material.roughness`/`metallic` in `viewport_pbr` and `r/g/b/a` in
    /// `viewport_color`, which is what Solid mode shades with, not the
    /// Principled's own values. A socket's value lands in `bsdf` only when
    /// its node is the surface the Material Output reaches and nothing is
    /// linked into it, because a linked socket's `default_value` never
    /// reaches a pixel.
    pub fn apply(&self, material: &mut BlendMaterial, scene: &BlendDriverScene, frame: f32) {
        for (write, value) in self.values(scene, frame) {
            match write {
                BMatWrite::Own(BMatProp::Roughness) => material.viewport_pbr[0] = value,
                BMatWrite::Own(BMatProp::Metallic) => material.viewport_pbr[1] = value,
                BMatWrite::Own(BMatProp::DiffuseColor(c)) => material.viewport_color[c] = value,
                BMatWrite::Own(BMatProp::SpecularIntensity) => {}
                write @ BMatWrite::Socket { .. } => self.fold(material, &write, value),
            }
        }
        // `base_color`, `roughness` and `metallic` are the surface's, read
        // off the Principled — so a driven socket has to move them too.
        material.base_color = material.bsdf.base;
        material.roughness = material.bsdf.roughness.clamp(0.0, 1.0);
        material.metallic = material.bsdf.metallic.clamp(0.0, 1.0);
    }
}

impl BMaterialDrivers {
    /// `value` folded into `material`'s surface, when `write` names a
    /// socket of the node the Material Output reaches and nothing is
    /// linked into it. A linked socket's `default_value` never reaches a
    /// pixel, so Blender's write to it changes no shading — and neither
    /// does this.
    pub fn fold(&self, material: &mut BlendMaterial, write: &BMatWrite, value: f32) {
        let BMatWrite::Socket { node, input, component } = write else { return };
        if self.surface.as_deref() != Some(node.as_str()) {
            return;
        }
        let Some(sock) = self.sockets(node).get(*input).filter(|sock| !sock.linked) else { return };
        write_bsdf(&mut material.bsdf, &sock.name, *component, value);
    }
}

/// One Principled input socket, by the name the node gives it, written into
/// the BSDF the same way `shade::principled`'s table reads it. Names follow
/// Blender 4.x with the 2.8–3.x spellings beside them.
fn write_bsdf(bsdf: &mut BlendBsdf, socket: &str, component: usize, value: f32) {
    let rgb = |slot: &mut [f32; 3]| {
        if component < 3 {
            slot[component] = value;
        }
    };
    match socket {
        "Base Color" if component < 4 => bsdf.base[component] = value,
        "Roughness" => bsdf.roughness = value,
        "Metallic" => bsdf.metallic = value,
        "Alpha" => bsdf.alpha = value,
        "IOR" => bsdf.ior = value,
        "Transmission Weight" | "Transmission" => bsdf.transmission = value,
        "Emission Color" | "Emission" => rgb(&mut bsdf.emission),
        "Emission Strength" => bsdf.emission_strength = value,
        "Subsurface Weight" | "Subsurface" => bsdf.subsurface = value,
        "Subsurface Radius" => rgb(&mut bsdf.subsurface_radius),
        "Subsurface Scale" => bsdf.subsurface_scale = value,
        "Sheen Weight" | "Sheen" => bsdf.sheen = value,
        "Sheen Roughness" => bsdf.sheen_roughness = value,
        "Sheen Tint" => rgb(&mut bsdf.sheen_tint),
        "Coat Weight" | "Clearcoat" => bsdf.coat = value,
        "Coat Roughness" | "Clearcoat Roughness" => bsdf.coat_roughness = value,
        "Anisotropic" => bsdf.anisotropic = value,
        "Anisotropic Rotation" => bsdf.anisotropic_rotation = value,
        _ => {}
    }
}

#[cfg(test)]
#[path = "driver_tests.rs"]
mod driver_tests;
