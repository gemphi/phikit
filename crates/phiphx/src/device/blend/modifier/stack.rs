//! A mesh object's modifier stack, read from the file in stack order.
use super::*;

impl BlendFile {
    /// Enabled viewport modifiers of `ob`, in stack order (supported ones).
    pub(crate) fn modifier_stack(&self, ob: &BlendBlock, dna: &DnaType) -> Vec<Box<dyn BModifier>> {
        let first = self.ptr(ob, self.off("Object", "modifiers", dna));
        let next = self.off("ModifierData", "*next", dna);
        let mds = self.scoped(ob, || self.list(first, next));
        // This frame's driven settings for THIS object, read before the
        // geometry was evaluated (`scene/read.rs`). Empty for almost every
        // object, so the name lookup below is paid for only where a driver
        // actually exists.
        let driven = {
            let held = self.driven_modifiers.lock().unwrap_or_else(|e| e.into_inner());
            held.get(&ob.old_address).cloned().unwrap_or_default()
        };
        let name_at = (!driven.is_empty()).then(|| self.modifier_name_offset(dna)).flatten();
        let mut stack: Vec<Box<dyn BModifier>> = mds
            .into_iter()
            .filter(|md| self.int_field(md, "ModifierData", "mode", dna) & MODE_REALTIME != 0)
            .filter_map(|md| {
                let ty = self.int_field(md, "ModifierData", "type", dna);
                let kind = KINDS.iter().find(|k| k.dna_type == ty)?;
                let mut built = self.scoped(ob, || (kind.read)(self, md, ob, dna))?;
                if let Some(name) = name_at.and_then(|at| self.modifier_name(md, Some(at))) {
                    // A driver's path carries the modifier's NAME only, and
                    // the same name means different properties on different
                    // types — so the type has to match too (`driver.rs`).
                    for (write, value) in driven.iter().filter(|(w, _)| w.modifier == name && w.kind == kind.name) {
                        built.set_driven(&write.prop, write.component, *value);
                    }
                }
                // `is_disabled`, on the settings as the drivers left them.
                (!built.disabled()).then_some(built)
            })
            .collect();
        // Blender ADDS a Geometry Nodes "Auto Smooth" modifier at the tail
        // while loading a file older than 4.2.22 whose mesh carries the
        // legacy auto-smooth flag (`nodes_legacy.rs`,
        // `BKE_main_mesh_legacy_convert_auto_smooth`), so the stack a file
        // describes is not the stack Blender runs.
        if let Some(m) = self.scoped(ob, || self.legacy_auto_smooth(ob, dna)) {
            stack.push(Box::new(m));
        }
        stack
    }

    /// Linked-list walk from a `ListBase.first` address.
    pub(crate) fn list(&self, first: u64, next: Option<usize>) -> Vec<&BlendBlock> {
        let mut out = Vec::new();
        let mut addr = first;
        while let Some(b) = self.block_at(addr) {
            if out.len() > 100_000 { break; }
            out.push(b);
            addr = self.ptr(b, next);
        }
        out
    }
}
