//! What an F-curve is called in the channel list — `getname_anim_fcurve`
//! (`editors/animation/anim_ipo_utils.cc:48`).
//!
//! Blender builds the name from three pieces: the array item's letter, the
//! property's **RNA UI name**, and the struct that owns it —
//! `"%s%s (%s)"`, or `"%s%s"` when nothing owns it. So
//! `pose.bones["Chest"].location` index 0 is `X Location (Chest)`. Its own
//! comment calls this "option 2" and admits it "does have problems with
//! looking rather odd though"; it is what ships, so it is what this does.
//!
//! The UI name is the piece that cannot be derived. Blender resolves the
//! path to the real property and asks `RNA_property_ui_name`, which returns
//! a **declared** string: `energy` is "Power", `spot_size` is "Beam Angle",
//! `mid_level` is "Midlevel", `merge_threshold` is "Merge Distance". A
//! title-cased identifier matches it only sometimes, and this file used to
//! fall back to one for everything outside an eighteen-entry hand table —
//! so most of the channel list read in names Blender never shows.
//!
//! The tables are now generated from Blender itself
//! (`tests/truth/blender_channel_name.py` → `truth/channel_name.txt`, 389
//! identifiers across objects, pose bones, lights, materials, worlds, every
//! constraint type, shape keys and the modifiers phiber evaluates), and
//! `tests/channel_name_truth.rs` holds this function to every row of it.
//!
//! The title-cased fallback remains for a property no table covers, because
//! that is also what RNA does for a property declaring no UI name.

pub mod data;
pub mod modifier;
pub mod rig;
pub mod set;

use crate::phiphx::blend::BFCurve;

/// An identifier whose UI name depends on **which** struct carries it, and
/// the path fragment that says which.
///
/// Blender never has this problem: it resolves the path to the property and
/// reads the name off that one. A table keyed by identifier alone has to be
/// told apart some other way, and for these three the path itself says it.
pub struct BOwnedName {
    /// The property identifier.
    pub property: &'static str,
    /// The fragment the path carries when [`Self::named`] applies.
    pub when: &'static str,
    /// `(UI name, letters)` when `when` is in the path.
    pub named: (&'static str, &'static str),
    /// `(UI name, letters)` when it is not.
    pub other: (&'static str, &'static str),
}

/// The three the path answers. Each row is a real difference in Blender:
/// an object's `matrix_basis` is "Input Matrix" and a pose bone's is "Basis
/// Matrix"; a constraint's `mute` is "Disable" and a shape key's is "Mute";
/// an IK constraint's `weight` is "Weight" and an Armature constraint
/// target's is "Blend Weight".
pub static OWNED: &[BOwnedName] = &[
    BOwnedName { property: "matrix_basis", when: "pose.bones[", named: ("Basis Matrix", ""), other: ("Input Matrix", "") },
    BOwnedName { property: "mute", when: "key_blocks[", named: ("Mute", ""), other: ("Disable", "") },
    BOwnedName { property: "weight", when: ".targets[", named: ("Blend Weight", ""), other: ("Weight", "") },
];

/// An identifier whose UI name depends on a struct the path does **not**
/// name, and the spelling phiber prints for it.
///
/// A path carries a modifier's and a constraint's *name* but never its
/// *type*, so Blender — which resolves the path to the property itself —
/// can tell a Solidify's `invert_vertex_group` from a Displace's and this
/// cannot. Rather than let such a property fall through to the title-cased
/// fallback (which gave "Invert Vertex Group", a spelling Blender shows for
/// no owner at all), the majority spelling is **declared** here and the
/// minority is recorded beside it.
pub struct BAmbiguousName {
    /// The property identifier.
    pub property: &'static str,
    /// `(UI name, letters)` phiber prints — the spelling most owners show.
    pub printed: (&'static str, &'static str),
    /// The one owner it is wrong for.
    pub odd: &'static str,
    /// `(UI name, letters)` that owner shows. `offset` is the row where the
    /// *letters* are what differ and the name does not, which is why both
    /// halves are recorded rather than the name alone.
    pub odd_shows: (&'static str, &'static str),
}

/// The four. Closing them would mean resolving the path against the loaded
/// model the way Blender does, which [`fcurve_name`] deliberately does not
/// do — it is handed a curve and nothing else, which is what makes it
/// testable without a file. `tests/channel_name_truth.rs` holds each row to
/// a spelling Blender really shows.
pub static AMBIGUOUS: &[BAmbiguousName] = &[
    BAmbiguousName { property: "invert_vertex_group", printed: ("Invert", ""), odd: "Solidify", odd_shows: ("Vertex Group Invert", "") },
    BAmbiguousName { property: "use_curve_radius", printed: ("Use Curve Radius", ""), odd: "Follow Path", odd_shows: ("Curve Radius", "") },
    BAmbiguousName { property: "use_rotation", printed: ("Use Rotation", ""), odd: "IK", odd_shows: ("Rotation", "") },
    // Pivot's offset is a three component vector and shows an axis letter;
    // Floor's, Follow Path's and Solidify's are scalars, and win. Here the
    // name agrees across owners and only the letters part company.
    BAmbiguousName { property: "offset", printed: ("Offset", ""), odd: "Pivot", odd_shows: ("Offset", "XYZW") },
];

/// `BLI_str_quoted_substr(path, prefix, …)`: the quoted name after `prefix`.
fn quoted_after<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = &path[path.find(prefix)? + prefix.len()..];
    let rest = rest.strip_prefix('"')?;
    rest.split_once('"').map(|(name, _)| name)
}

/// `RNA_def_property`'s default UI name: the identifier, spaced and capitalised.
fn title_case(identifier: &str) -> String {
    identifier
        .split('_')
        .map(|word| {
            let mut letters = word.chars();
            letters.next().map_or(String::new(), |first| first.to_uppercase().chain(letters).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `arrayname`: the item's letter and a space, or Blender's bracketed index
/// when the subtype has no letter for it (`if (c)` fails past index 3).
fn array_prefix(letters: &str, index: u32) -> String {
    if letters.is_empty() {
        return String::new();
    }
    match letters.chars().nth(index as usize) {
        Some(letter) => format!("{letter} "),
        None => format!("[{index}]"),
    }
}

/// The `structname` half: the bone, the constraint, or both as
/// `bone : constraint`.
///
/// Blender's first branch is exactly that pair; its remaining branches ask
/// RNA for a struct's name property, which has no equivalent here. A
/// modifier and a node are kept because a driver on either is common and
/// reads as nothing otherwise.
fn owner(path: &str) -> Option<String> {
    let bone = quoted_after(path, "bones[");
    let constraint = quoted_after(path, "constraints[");
    match (bone, constraint) {
        (Some(bone), Some(constraint)) => Some(format!("{bone} : {constraint}")),
        (Some(bone), None) => Some(bone.to_string()),
        (None, Some(constraint)) => Some(constraint.to_string()),
        (None, None) => quoted_after(path, "modifiers[")
            .or_else(|| quoted_after(path, "nodes["))
            .or_else(|| quoted_after(path, "key_blocks["))
            .map(str::to_string),
    }
}

/// `RNA_property_ui_name(prop)` and `RNA_property_array_item_char`'s letters
/// for `property` — the generated tables, then RNA's own fallback.
fn ui_name(property: &str, path: &str) -> (String, &'static str) {
    if let Some(row) = OWNED.iter().find(|row| row.property == property) {
        let (name, letters) = if path.contains(row.when) { row.named } else { row.other };
        return (name.to_string(), letters);
    }
    if let Some(row) = AMBIGUOUS.iter().find(|row| row.property == property) {
        return (row.printed.0.to_string(), row.printed.1);
    }
    set::OBJECT
        .iter()
        .chain(data::DATABLOCK)
        .chain(rig::RIG)
        .chain(modifier::MODIFIER)
        .find(|(name, _, _)| *name == property)
        .map_or_else(|| (title_case(property), ""), |(_, label, letters)| ((*label).to_string(), *letters))
}

/// `getname_anim_fcurve`: `<axis> <property> (<struct>)`.
pub fn fcurve_name(curve: &BFCurve) -> String {
    // `if (fcu->rna_path == nullptr)` — a curve the file gave no path.
    if curve.path.is_empty() {
        return "<no path>".to_string();
    }
    let property = curve.property();
    let (label, letters) = ui_name(property, &curve.path);
    let axis = array_prefix(letters, curve.index);
    match owner(&curve.path) {
        Some(owner) => format!("{axis}{label} ({owner})"),
        None => format!("{axis}{label}"),
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;
