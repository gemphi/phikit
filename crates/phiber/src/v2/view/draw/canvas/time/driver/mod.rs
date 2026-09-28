//! The Drivers editor's data — the driver F-curves of the shown objects
//! (`ANIM_animdata_filter` with `ADS_FILTER_ONLYDRIVERS`, Only Show
//! Selected as `graph_create` sets it) and the names the channel region
//! prints for them (`getname_anim_fcurve`). Structure here; the cursor
//! lines the editor paints over the curves are `set.rs`.

pub mod set;

use super::curve::{BAxisColors, BCurveSet, BDrawnCurve};
use super::row::{BRowKind, BTimeRow};
use crate::phiphx::blend::BFCurve;
use crate::phiphx::scene::SceneDrivers;
use crate::v2::state::{BState, CtxKey};

/// `RNA_property_ui_name` of the properties drivers reach, and the axis
/// letters `RNA_property_array_item_char` gives their items.
const PROPERTY_NAMES: &[(&str, &str, &str)] = &[
    ("influence", "Influence", ""),
    ("mute", "Disable", ""),
    ("hide", "Hide", ""),
    ("weight", "Weight", ""),
    ("location", "Location", "XYZ"),
    ("rotation_euler", "Euler Rotation", "XYZ"),
    ("rotation_quaternion", "Quaternion Rotation", "WXYZ"),
    ("scale", "Scale", "XYZ"),
    ("value", "Value", ""),
];
/// `ConstraintTargetBone.weight`: the `targets[…]` of an Armature constraint.
const TARGET_WEIGHT: &str = "Blend Weight";

/// `BLI_str_quoted_substr(path, prefix, …)`: the quoted name after `prefix`.
fn quoted_after<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = &path[path.find(prefix)? + prefix.len()..];
    let rest = rest.strip_prefix('"')?;
    rest.split_once('"').map(|(name, _)| name)
}

impl BTimeRow {
    /// `getname_anim_fcurve`: `<axis> <property> (<struct>)` — the struct a
    /// bone and its constraint (`bone : constraint`), the bone alone, an
    /// object constraint alone, or nothing for the ID's own properties.
    pub fn driver_name(curve: &BFCurve) -> String {
        let property = curve.property();
        let (label, axes) = PROPERTY_NAMES
            .iter()
            .find(|(name, _, _)| *name == property)
            .map_or_else(|| (title_case(property), ""), |(_, label, axes)| ((*label).to_string(), *axes));
        let label = if property == "weight" && curve.path.contains(".targets[") { TARGET_WEIGHT.to_string() } else { label };
        let axis = axes.chars().nth(curve.index as usize).map_or(String::new(), |letter| format!("{letter} "));
        let bone = quoted_after(&curve.path, "bones[");
        let constraint = quoted_after(&curve.path, "constraints[");
        let owner = match (bone, constraint) {
            (Some(bone), Some(constraint)) => Some(format!("{bone} : {constraint}")),
            (Some(bone), None) => Some(bone.to_string()),
            (None, Some(constraint)) => Some(constraint.to_string()),
            (None, None) => quoted_after(&curve.path, "modifiers[").map(str::to_string),
        };
        match owner {
            Some(owner) => format!("{axis}{label} ({owner})"),
            None => format!("{axis}{label}"),
        }
    }
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

/// The shown objects' drivers, in object order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BDriverList {
    pub objects: Vec<SceneDrivers>,
}

impl BDriverList {
    /// Only Show Selected, plus the active object.
    pub fn now(s: &BState) -> Self {
        let active = s.get(CtxKey::Object).map(str::to_string);
        crate::v2::scene::source::with_loaded(|loaded| {
            let Some(model) = loaded else { return Self::default() };
            let mut shown: Vec<&str> = model.selection.selected.iter().map(String::as_str).collect();
            shown.extend(active.as_deref());
            Self { objects: model.drivers.iter().filter(|drivers| shown.contains(&drivers.object.as_str())).cloned().collect() }
        })
    }

    /// Every driver curve, coloured over the whole list as
    /// `graph_refresh_fcurve_colors` colours it.
    pub fn curves(&self, axis: BAxisColors) -> Vec<BDrawnCurve> {
        BCurveSet::of(self.objects.iter().flat_map(|drivers| drivers.curves.iter().cloned()).collect(), axis).list
    }

    /// The channel rows: each object, then a row per driver under it, each
    /// carrying its curve's colour for the swatch `ANIM_channel_draw` paints.
    pub fn rows(&self, axis: BAxisColors) -> Vec<BTimeRow> {
        let mut drawn = self.curves(axis).into_iter();
        self.objects
            .iter()
            .flat_map(|drivers| {
                let object = std::iter::once(BTimeRow { kind: BRowKind::Object(drivers.object.clone()), keys: Vec::new() });
                let curves: Vec<BTimeRow> = drivers
                    .curves
                    .iter()
                    .map(|curve| BTimeRow { kind: BRowKind::Driver { name: BTimeRow::driver_name(curve), color: drawn.next().map_or([0.0; 3], |d| d.color) }, keys: Vec::new() })
                    .collect();
                object.chain(curves)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve(path: &str, index: u32) -> BFCurve {
        BFCurve::new(path, index, Vec::new())
    }

    #[test]
    fn channel_names_are_blenders() {
        assert_eq!(BTimeRow::driver_name(&curve("pose.bones[\"MCH-upper_arm_parent.L\"].constraints[\"Copy Rotation\"].influence", 0)), "Influence (MCH-upper_arm_parent.L : Copy Rotation)");
        assert_eq!(BTimeRow::driver_name(&curve("pose.bones[\"MCH-forearm_ik.L\"].constraints[\"IK\"].mute", 0)), "Disable (MCH-forearm_ik.L : IK)");
        assert_eq!(BTimeRow::driver_name(&curve("pose.bones[\"MCH-torso.parent\"].constraints[\"SWITCH_PARENT\"].targets[0].weight", 0)), "Blend Weight (MCH-torso.parent : SWITCH_PARENT)");
        assert_eq!(BTimeRow::driver_name(&curve("pose.bones[\"MCH-thumb.02_drv.L\"].rotation_euler", 0)), "X Euler Rotation (MCH-thumb.02_drv.L)");
        assert_eq!(BTimeRow::driver_name(&curve("pose.bones[\"thigh_ik.R\"].hide", 0)), "Hide (thigh_ik.R)");
        assert_eq!(BTimeRow::driver_name(&curve("rotation_quaternion", 0)), "W Quaternion Rotation", "the ID's own property names no struct");
        assert_eq!(BTimeRow::driver_name(&curve("modifiers[\"Wave\"].height", 0)), "Height (Wave)", "an unknown property is title-cased");
        assert_eq!(BTimeRow::driver_name(&curve("constraints[\"Track To\"].influence", 0)), "Influence (Track To)");
    }

    #[test]
    fn rows_stack_each_object_over_its_drivers() {
        let list = BDriverList {
            objects: vec![SceneDrivers { object: "rig".into(), curves: vec![curve("pose.bones[\"a\"].hide", 0), curve("location", 2)] }],
        };
        let axis = BAxisColors { x: [1.0; 3], y: [1.0; 3], z: [1.0; 3], w: [1.0; 3], other: [1.0; 3] };
        let rows = list.rows(axis);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].kind, BRowKind::Object("rig".into()));
        assert_eq!(rows[1].name(), "Hide (a)");
        assert_eq!(rows[2].name(), "Z Location");
        let curves = list.curves(axis);
        assert_eq!(curves.len(), 2);
        assert_eq!(rows[2].kind, BRowKind::Driver { name: "Z Location".into(), color: curves[1].color }, "the row wears its curve's colour");
    }
}
