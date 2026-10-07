//! Reading an object's drivers out of the file: every `AnimData.drivers`
//! F-curve whose path is one of the object's own transform channels or one
//! of its custom properties, with its `ChannelDriver` — type, expression —
//! and each `DriverVar`'s targets, which name the IDs and RNA paths it
//! reads.
//!
//! The paths stay as the file spells them; what they resolve to is decided
//! here, by the same `channel_mut` table the animation system writes
//! through, so a driver can only write a channel the renderer can read.

use super::target::{BDriverTarget, BDriverTargets};
use super::{BlendDriver, BlendDriverEnd, BlendDriverRead, BlendDriverSet, BlendDriverVar, BlendObjectDriver, BlendPropDriver};
use crate::blend::file::types::*;
use crate::blend::rig::driver::expr::BExpr;

/// `eDriverVar_Types`.
const DVAR_SINGLE_PROP: u32 = 0;
const DVAR_ROT_DIFF: u32 = 1;
const DVAR_LOC_DIFF: u32 = 2;
const DVAR_TRANSFORM_CHAN: u32 = 3;

/// The channels a driver may write, as `anim::channels::channel_mut` reads
/// them — the property's name and how many indices it has.
const CHANNELS: [(&str, usize); 5] =
    [("location", 3), ("rotation_euler", 3), ("rotation_quaternion", 4), ("rotation_axis_angle", 4), ("scale", 3)];

/// `property` and `index` when the path names one of those channels. A path
/// with anything before it (`pose.bones[…].location`) is the rig's, not the
/// object's, and is not one.
pub(super) fn own_channel(path: &str, index: usize) -> Option<(String, usize)> {
    CHANNELS.iter().find(|(name, count)| *name == path && index < *count).map(|(name, _)| ((*name).to_string(), index))
}

/// `["name"]` — the whole path, and nothing before or after it: a custom
/// property of the ID itself.
pub(super) fn own_prop(path: &str) -> Option<String> {
    let inner = path.strip_prefix("[\"")?.strip_suffix("\"]")?;
    (!inner.is_empty() && !inner.contains('[')).then(|| inner.replace("\\\"", "\""))
}

impl BlendFile {
    /// Everything object `ob` brings to a frame's driver pass: the drivers
    /// on its own transform channels, the drivers on its own custom
    /// properties, and those properties as saved. A driver on anything else
    /// — a material, a modifier, a shape key — is read by `anim::drivers`
    /// for the Drivers editor to list and is not evaluated here.
    pub(crate) fn object_driver_set(&self, ob: &BlendBlock, dna: &DnaType) -> BlendDriverSet {
        let saved = self.id_properties(ob, dna);
        let mut set = BlendDriverSet { address: ob.old_address, saved, ..Default::default() };
        let Some(adt) = self.block_at(self.ptr(ob, self.off("Object", "*adt", dna))) else { return set };
        let first = self.ptr(adt, self.off("AnimData", "drivers", dna));
        for fc in self.list(first, self.off("FCurve", "*next", dna)) {
            let Some(curve) = self.fcurve(fc, dna) else { continue };
            let Some(driver) = self.channel_driver(fc, curve.clone(), dna) else { continue };
            if let Some((property, index)) = own_channel(&curve.path, curve.index as usize) {
                set.channels.push(BlendObjectDriver { property, index, driver });
            } else if let Some(name) = own_prop(&curve.path) {
                set.props.push(BlendPropDriver { name, driver });
            }
        }
        set
    }

    /// `FCurve.driver`: the `ChannelDriver` under a curve, with its
    /// variables resolved and its expression parsed.
    pub(crate) fn channel_driver(&self, fc: &BlendBlock, curve: crate::blend::anim::fcurve::BFCurve, dna: &DnaType) -> Option<BlendDriver> {
        let driver = self.block_at(self.ptr(fc, self.off("FCurve", "*driver", dna)))?;
        let expression = self.driver_expression(driver, dna);
        let kind = self.int_field(driver, "ChannelDriver", "type", dna);
        let vars: Vec<BlendDriverVar> = self
            .list(self.ptr(driver, self.off("ChannelDriver", "variables", dna)), self.off("DriverVar", "*next", dna))
            .into_iter()
            .map(|var| self.object_driver_var(var, dna))
            .collect();
        let names: Vec<String> = std::iter::once("frame".to_string()).chain(vars.iter().map(|v| v.name.clone())).collect();
        let parsed = BExpr::parse(&expression, &names);
        Some(BlendDriver { kind, expression, vars, parsed, curve })
    }

    /// `ChannelDriver.expression`, a fixed 256-byte string.
    fn driver_expression(&self, driver: &BlendBlock, dna: &DnaType) -> String {
        let Some(o) = self.off("ChannelDriver", "expression[256]", dna) else { return String::new() };
        let Some(bytes) = driver.data.get(o..(o + 256).min(driver.data.len())) else { return String::new() };
        let end = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).to_string()
    }

    /// One variable: its name, its kind, and what its targets name.
    fn object_driver_var(&self, var: &BlendBlock, dna: &DnaType) -> BlendDriverVar {
        let name = self
            .off("DriverVar", "name[64]", dna)
            .and_then(|o| var.data.get(o..(o + 64).min(var.data.len())))
            .map(|b| {
                let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
                String::from_utf8_lossy(&b[..end]).to_string()
            })
            .unwrap_or_default();
        // `DriverVar.type` reads `u32::MAX` when the file has no such
        // field, so an unknown layout is Unsupported rather than Single
        // Property by accident.
        let kind = self.off("DriverVar", "type", dna).map_or(u32::MAX, |_| self.int_field(var, "DriverVar", "type", dna));
        BlendDriverVar { name, read: self.driver_read(var, kind, dna) }
    }

    /// What a variable of this kind reads from its targets.
    fn driver_read(&self, var: &BlendBlock, kind: u32, dna: &DnaType) -> BlendDriverRead {
        let target = |slot: usize| self.driver_target(var, slot, dna);
        let Some(first) = target(0) else { return BlendDriverRead::Unsupported };
        // A target naming a bone is the rig's business (`rig/driver`), not
        // this pass's.
        if !first.bone.is_empty() {
            return BlendDriverRead::Unsupported;
        }
        match kind {
            DVAR_SINGLE_PROP => Self::single_prop(&first),
            DVAR_TRANSFORM_CHAN => {
                BlendDriverRead::Transform { object: first.id, chan: first.chan, space: first.space, rot_mode: first.rot_mode }
            }
            DVAR_LOC_DIFF if self.driver_target_count(var, dna) >= 2 => match target(1).filter(|t| t.bone.is_empty() && t.id != 0) {
                Some(second) if first.id != 0 => BlendDriverRead::LocDiff { a: Self::end(&first), b: Self::end(&second) },
                _ => BlendDriverRead::Unsupported,
            },
            DVAR_ROT_DIFF => match target(1).filter(|t| t.bone.is_empty() && t.id != 0) {
                Some(second) if first.id != 0 => BlendDriverRead::RotDiff { a: first.id, b: second.id },
                _ => BlendDriverRead::Unsupported,
            },
            _ => BlendDriverRead::Unsupported,
        }
    }

    /// `dvar_eval_singleProp`'s target: a transform channel the renderer
    /// reads, or a custom property of the object.
    fn single_prop(target: &BDriverTarget) -> BlendDriverRead {
        let (name, index) = split_index(&target.path);
        if let Some((property, index)) = own_channel(&name, index) {
            return BlendDriverRead::Channel { object: target.id, property, index };
        }
        match own_prop(&target.path) {
            Some(name) => BlendDriverRead::Prop { object: target.id, name },
            None => BlendDriverRead::Unsupported,
        }
    }

    fn end(target: &BDriverTarget) -> BlendDriverEnd {
        BlendDriverEnd { object: target.id, space: target.space }
    }
}

/// `location[1]` as `("location", 1)`; a path with no subscript is index 0.
pub(super) fn split_index(path: &str) -> (String, usize) {
    match path.split_once('[') {
        Some((name, rest)) => (name.to_string(), rest.trim_end_matches(']').parse().unwrap_or(0)),
        None => (path.to_string(), 0),
    }
}

#[cfg(test)]
#[path = "read_tests.rs"]
mod read_tests;
