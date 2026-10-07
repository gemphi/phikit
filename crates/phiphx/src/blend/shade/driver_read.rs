//! Reading a material's drivers out of the file: `Material.adt->drivers`
//! for the material's own properties, and `Material.nodetree->adt->drivers`
//! for the sockets of its shader nodes.
//!
//! The socket path Blender writes is `nodes["<name>"].inputs[<i>].default_value`
//! — the node by **name** and the socket by its **index** in that node's
//! input list, which is the `bNode.inputs` ListBase order the file stores.
//! The component is the curve's `array_index`, not part of the path.

use super::driver::{BMatProp, BMatWrite, BMaterialDriver, BMaterialDrivers};
use crate::blend::anim::driver::socket_path;
use crate::blend::file::types::*;

impl BlendFile {
    /// Every material in the file that carries a driver, by block address —
    /// what a frame's pass needs to write the materials it reads.
    pub(crate) fn material_drivers(&self, dna: &DnaType) -> Vec<(u64, BMaterialDrivers)> {
        self.blocks
            .iter()
            .filter(|block| block.code_str == "MA")
            .filter_map(|ma| {
                let drivers = self.material_driver_set(ma, dna);
                (!drivers.is_empty()).then_some((ma.old_address, drivers))
            })
            .collect()
    }

    /// One material's drivers over both its IDs, with the socket lists a
    /// socket index is resolved through.
    pub(crate) fn material_driver_set(&self, ma: &BlendBlock, dna: &DnaType) -> BMaterialDrivers {
        self.scoped(ma, || {
            let mut set = BMaterialDrivers { material: self.id_name(ma, dna), ..Default::default() };
            for (block, owner) in [(Some(ma), "Material"), (self.node_tree(ma, "Material", dna), "bNodeTree")] {
                let Some(block) = block else { continue };
                for fc in self.adt_driver_curves(block, owner, dna) {
                    let Some(curve) = self.fcurve(fc, dna) else { continue };
                    let index = curve.index as usize;
                    let Some(write) = self.material_write(&curve.path, index, owner) else { continue };
                    let Some(driver) = self.channel_driver(fc, curve, dna) else { continue };
                    set.drivers.push(BMaterialDriver { write, driver });
                }
            }
            self.fill_sockets(&mut set, ma, dna);
            set
        })
    }


    /// What a curve on `owner` writes, as `None` for a path this does not
    /// model — a driver on a texture node's own property, say, which the
    /// Drivers editor lists and the surface does not read.
    fn material_write(&self, path: &str, index: usize, owner: &str) -> Option<BMatWrite> {
        match owner {
            "Material" => BMatProp::of(path, index).map(BMatWrite::Own),
            _ => socket_path(path).map(|(node, input)| BMatWrite::Socket { node, input, component: index }),
        }
    }

    /// `Material.nodetree`, when the material has one.
    fn node_tree(&self, id: &BlendBlock, owner: &str, dna: &DnaType) -> Option<&BlendBlock> {
        self.block_at(self.ptr(id, self.off(owner, "*nodetree", dna)))
    }

    /// The input sockets of every node a driver on `set` writes, and the
    /// name of the surface's Principled node.
    fn fill_sockets(&self, set: &mut BMaterialDrivers, ma: &BlendBlock, dna: &DnaType) {
        if set.is_empty() {
            return;
        }
        set.surface = self.surface_principled(ma, dna).and_then(|node| self.node_name(node, dna));
        let mut wanted: Vec<String> = set
            .drivers
            .iter()
            .filter_map(|d| match &d.write {
                BMatWrite::Socket { node, .. } => Some(node.clone()),
                BMatWrite::Own(_) => None,
            })
            .collect();
        wanted.sort();
        wanted.dedup();
        set.inputs = self.driver_sockets_of(ma, "Material", &wanted, dna);
    }
}
