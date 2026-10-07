//! Reading lights' and the world's drivers out of the file.
//!
//! A light keeps all of its drivers on the `Light` ID (`LA` blocks); it has
//! no node tree in Blender 5.x — the `Lamp` DNA struct still carries a
//! `*nodetree` pointer, but nothing drives through it, so it is not read.
//! Note the DNA name: RNA's `Light` is DNA's **`Lamp`** even in 5.0.1
//! ([`BlendFile::lamp_struct`]). A world keeps `color` on the `World` ID
//! (`WO` blocks) and its Background node's sockets on `World.nodetree`, the
//! same split a material has — so the socket half is the shared reader in
//! [`anim::driver::socket`].
//!
//! [`anim::driver::socket`]: crate::blend::anim::driver::socket

use super::light_driver::{BLightDriver, BLightDrivers, BLightProp};
use crate::blend::anim::driver::socket_path;
use crate::blend::file::types::*;
use crate::blend::shade::world_driver::{BWorldDriver, BWorldDrivers, BWorldWrite};

impl BlendFile {
    /// Every light in the file that carries a driver, by `Light` name.
    pub(crate) fn light_drivers(&self, dna: &DnaType) -> Vec<BLightDrivers> {
        self.blocks
            .iter()
            .filter(|block| block.code_str == "LA")
            .filter_map(|la| {
                self.scoped(la, || {
                    let mut set = BLightDrivers { light: self.id_name(la, dna), drivers: Vec::new() };
                    for fc in self.adt_driver_curves(la, Self::lamp_struct(dna), dna) {
                        let Some(curve) = self.fcurve(fc, dna) else { continue };
                        let Some(prop) = BLightProp::of(&curve.path, curve.index as usize) else { continue };
                        let Some(driver) = self.channel_driver(fc, curve, dna) else { continue };
                        set.drivers.push(BLightDriver { prop, driver });
                    }
                    (!set.is_empty()).then_some(set)
                })
            })
            .collect()
    }

    /// Every world in the file that carries a driver, over both its IDs.
    pub(crate) fn world_drivers(&self, dna: &DnaType) -> Vec<BWorldDrivers> {
        self.blocks
            .iter()
            .filter(|block| block.code_str == "WO")
            .filter_map(|wo| {
                self.scoped(wo, || {
                    let mut set = BWorldDrivers { world: self.id_name(wo, dna), ..Default::default() };
                    // `World.color` on the ID itself.
                    for fc in self.adt_driver_curves(wo, "World", dna) {
                        let Some(curve) = self.fcurve(fc, dna) else { continue };
                        let index = curve.index as usize;
                        if curve.path != "color" || index >= 3 {
                            continue;
                        }
                        let Some(driver) = self.channel_driver(fc, curve, dna) else { continue };
                        set.drivers.push(BWorldDriver { write: BWorldWrite::Color(index), driver });
                    }
                    // The tree's sockets.
                    let tree = self.block_at(self.ptr(wo, self.off("World", "*nodetree", dna)));
                    for fc in tree.map(|t| self.adt_driver_curves(t, "bNodeTree", dna)).unwrap_or_default() {
                        let Some(curve) = self.fcurve(fc, dna) else { continue };
                        let component = curve.index as usize;
                        let Some((node, input)) = socket_path(&curve.path) else { continue };
                        let Some(driver) = self.channel_driver(fc, curve, dna) else { continue };
                        set.drivers.push(BWorldDriver { write: BWorldWrite::Socket { node, input, component }, driver });
                    }
                    let mut wanted: Vec<String> = set
                        .drivers
                        .iter()
                        .filter_map(|d| match &d.write {
                            BWorldWrite::Socket { node, .. } => Some(node.clone()),
                            BWorldWrite::Color(_) => None,
                        })
                        .collect();
                    wanted.sort();
                    wanted.dedup();
                    set.inputs = self.driver_sockets_of(wo, "World", &wanted, dna);
                    (!set.is_empty()).then_some(set)
                })
            })
            .collect()
    }

    /// `AnimData.drivers` of `id`, as the F-curve blocks — the
    /// `ChannelDriver` under each one is read from the block, which
    /// `id_drivers` does not hand back.
    pub(crate) fn adt_driver_curves(&self, id: &BlendBlock, owner: &str, dna: &DnaType) -> Vec<&BlendBlock> {
        let Some(adt) = self.block_at(self.ptr(id, self.off(owner, "*adt", dna))) else { return Vec::new() };
        self.list(self.ptr(adt, self.off("AnimData", "drivers", dna)), self.off("FCurve", "*next", dna))
    }
}
