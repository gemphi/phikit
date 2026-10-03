//! Reading the rig object's drivers: each `AnimData.drivers` F-curve with
//! its `ChannelDriver` — type, expression, and `DriverVar`s whose first
//! `DriverTarget` names what they read.

use super::expr::BExpr;
use super::{BDriven, BDriverSource, BDriverVar, BRigDriver, BRnaPath};
use crate::blend::file::types::*;
use crate::blend::rig::read::BFields;
use crate::blend::rig::BRig;

/// `DVAR_TYPE_SINGLE_PROP`.
const DVAR_SINGLE_PROP: u32 = 0;

impl BlendFile {
    /// The drivers on armature object `ob`, resolved against its bones.
    pub(crate) fn rig_drivers(&self, ob: &BlendBlock, dna: &DnaType) -> Vec<BRigDriver> {
        let Some(adt) = self.block_at(self.ptr(ob, self.off("Object", "*adt", dna))) else { return Vec::new() };
        let first = self.ptr(adt, self.off("AnimData", "drivers", dna));
        let fcurves = self.list(first, self.off("FCurve", "*next", dna));
        fcurves.into_iter().filter_map(|fc| self.rig_driver_raw(fc, dna, ob.old_address)).collect()
    }

    /// One driver, its paths kept as text until the rig resolves them.
    fn rig_driver_raw(&self, fc: &BlendBlock, dna: &DnaType, rig: u64) -> Option<BRigDriver> {
        let curve = self.fcurve(fc, dna)?;
        let driver = self.block_at(self.ptr(fc, self.off("FCurve", "*driver", dna)))?;
        let d = BFields::new(self, dna, driver, "ChannelDriver");
        let expression = d.text("expression[256]", 256);
        let vars = self
            .list(d.ptr("variables"), self.off("DriverVar", "*next", dna))
            .into_iter()
            .map(|var| self.driver_var_raw(var, dna, rig))
            .collect();
        Some(BRigDriver { driven: BDriven::Other(curve.path.clone()), curve, kind: d.int("type"), expression, vars, parsed: None })
    }

    /// A variable: its name and what it reads — a single property of the
    /// rig is kept as its path (in `Prop`, bone `usize::MAX`) until the rig
    /// resolves it; anything else is unsupported.
    fn driver_var_raw(&self, var: &BlendBlock, dna: &DnaType, rig: u64) -> BDriverVar {
        let v = BFields::new(self, dna, var, "DriverVar");
        let name = v.name("name[64]");
        let target = self.off("DriverVar", "targets[8]", dna);
        let (Some(target), Some(id_o), Some(path_o)) = (target, self.off("DriverTarget", "*id", dna), self.off("DriverTarget", "*rna_path", dna)) else {
            return BDriverVar { name, source: BDriverSource::Unsupported };
        };
        let ps = self.header.pointer_size as usize;
        let read = |o: usize| Self::read_ptr(&var.data, o, ps, self.header.little_endian);
        let id = read(target + id_o);
        let path = self.block_at(read(target + path_o)).map(|b| {
            let z = b.data.iter().position(|&c| c == 0).unwrap_or(b.data.len());
            String::from_utf8_lossy(&b.data[..z]).to_string()
        });
        let source = match (v.int("type"), path) {
            (DVAR_SINGLE_PROP, Some(path)) if id == rig => BDriverSource::Prop { bone: usize::MAX, name: path },
            _ => BDriverSource::Unsupported,
        };
        BDriverVar { name, source }
    }
}

impl BRig {
    /// Resolve the drivers' paths now the bones and constraints are known:
    /// what each writes, what each variable reads, the expression parsed.
    pub(crate) fn resolve_drivers(&mut self) {
        let drivers = std::mem::take(&mut self.drivers);
        self.drivers = drivers
            .into_iter()
            .map(|mut d| {
                let path = BRnaPath::parse(&d.curve.path);
                d.driven = path.as_ref().and_then(|p| p.driven(self)).unwrap_or_else(|| BDriven::Other(d.curve.path.clone()));
                for v in &mut d.vars {
                    if let BDriverSource::Prop { bone: usize::MAX, name } = &v.source {
                        v.source = BRnaPath::parse(name).map_or(BDriverSource::Unsupported, |p| p.source(self));
                    }
                }
                let names: Vec<String> = std::iter::once("frame".to_string()).chain(d.vars.iter().map(|v| v.name.clone())).collect();
                d.parsed = BExpr::parse(&d.expression, &names);
                d
            })
            .collect();
    }
}
