//! Reading modifier drivers out of the file.
//!
//! They sit on the object's `AnimData`, beside its transform and
//! custom-property drivers, so this walks the same list
//! `anim::driver::read` walks and picks up the paths that one skips.
//!
//! The modifier's **type** is what the setting table needs and the path
//! does not carry, so the object's whole modifier list is read first —
//! every modifier, not only the enabled and supported ones the evaluation
//! stack keeps, because Blender writes a driven value whether or not the
//! modifier is switched on.

use super::driver::{BModDriver, BModWrite, BModifierDrivers, setting};
use crate::blend::file::types::*;
use crate::blend::modifier::KINDS;
use crate::blend::scene::types::BlendSceneObject;

const MODIFIERS_PREFIX: &str = "modifiers[\"";
const NAME_END: &str = "\"].";

/// `modifiers["Subsurf"].levels` as `("Subsurf", "levels")`. The name runs
/// to its own closing quote, so a modifier called `a"].x` cannot shift
/// which property is read.
pub(crate) fn modifier_path(path: &str) -> Option<(String, String)> {
    let rest = path.strip_prefix(MODIFIERS_PREFIX)?;
    let quote = rest.find('"')?;
    let (name, tail) = (&rest[..quote], &rest[quote..]);
    let prop = tail.strip_prefix(NAME_END)?;
    // A nested path (`modifiers["M"].texture.name`) is not a setting.
    if name.is_empty() || prop.is_empty() || prop.contains('.') || prop.contains('[') {
        return None;
    }
    Some((name.to_string(), prop.to_string()))
}

impl BlendFile {
    /// Every object that carries a modifier driver, in object order.
    pub(crate) fn modifier_drivers(&self, objects: &[BlendSceneObject], dna: &DnaType) -> Vec<BModifierDrivers> {
        objects
            .iter()
            .filter_map(|object| {
                let ob = self.block_at(object.block_address)?;
                let set = self.object_modifier_drivers(object.name.clone(), ob, dna);
                (!set.is_empty()).then_some(set)
            })
            .collect()
    }

    /// One object's modifier drivers, with each modifier's type resolved
    /// from its own stack.
    fn object_modifier_drivers(&self, name: String, ob: &BlendBlock, dna: &DnaType) -> BModifierDrivers {
        let mut set = BModifierDrivers { object: name, drivers: Vec::new() };
        let stack = self.modifier_kinds(ob, dna);
        if stack.is_empty() {
            return set;
        }
        for fc in self.adt_driver_curves(ob, "Object", dna) {
            let Some(curve) = self.fcurve(fc, dna) else { continue };
            let component = curve.index as usize;
            let Some((modifier, prop)) = modifier_path(&curve.path) else { continue };
            // A driver on a modifier this object has not got, or on a
            // setting this does not model, is read and dropped.
            let Some(kind) = stack.iter().find(|(n, _)| *n == modifier).map(|(_, k)| *k) else { continue };
            let Some(setting) = setting(kind, &prop, component) else { continue };
            let Some(driver) = self.channel_driver(fc, curve, dna) else { continue };
            let write = BModWrite { modifier, kind: kind.to_string(), prop, component };
            set.drivers.push(BModDriver { write, setting, driver });
        }
        set
    }

    /// `(name, type)` of every modifier on `ob`, in stack order —
    /// including the ones the evaluation stack filters out.
    fn modifier_kinds(&self, ob: &BlendBlock, dna: &DnaType) -> Vec<(String, &'static str)> {
        let first = self.ptr(ob, self.off("Object", "modifiers", dna));
        let next = self.off("ModifierData", "*next", dna);
        let name_o = self.off("ModifierData", "name[64]", dna).or_else(|| self.off("ModifierData", "name", dna));
        self.scoped(ob, || self.list(first, next))
            .into_iter()
            .filter_map(|md| {
                let ty = self.int_field(md, "ModifierData", "type", dna);
                let kind = KINDS.iter().find(|k| k.dna_type == ty)?;
                let o = name_o?;
                let bytes = md.data.get(o..(o + 64).min(md.data.len()))?;
                let end = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
                Some((String::from_utf8_lossy(&bytes[..end]).to_string(), kind.name))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::modifier_path;

    #[test]
    fn a_modifier_path_is_a_name_and_a_property() {
        assert_eq!(modifier_path("modifiers[\"Subsurf\"].levels"), Some(("Subsurf".into(), "levels".into())));
        assert_eq!(
            modifier_path("modifiers[\"My Array\"].constant_offset_displace"),
            Some(("My Array".into(), "constant_offset_displace".into()))
        );
    }

    #[test]
    fn a_path_that_is_not_a_modifier_setting_is_not_read_as_one() {
        assert_eq!(modifier_path("location"), None);
        assert_eq!(modifier_path("[\"gain\"]"), None);
        assert_eq!(modifier_path("modifiers[\"M\"]."), None, "no property");
        assert_eq!(modifier_path("modifiers[\"\"].levels"), None, "no name");
        // A nested path reaches through the modifier to another datablock,
        // which is not a setting on it.
        assert_eq!(modifier_path("modifiers[\"Disp\"].texture.contrast"), None);
        assert_eq!(modifier_path("modifiers[\"Disp\"].vertex_group[0]"), None);
    }

    #[test]
    fn the_modifier_name_ends_at_its_own_quote() {
        // The name is `a`, so the property would be `x"].levels` — which
        // holds a `.` and is therefore not a setting on it. A name chosen
        // to look like the rest of the path cannot redirect the read.
        assert_eq!(modifier_path("modifiers[\"a\"].x\"].levels"), None);
        // And a plain name with a quote after it reads as itself.
        assert_eq!(modifier_path("modifiers[\"a b\"].offset"), Some(("a b".into(), "offset".into())));
    }
}
