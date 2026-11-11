//! `Object.type` → Blender type name (`OB_*`, `DNA_object_types.h`, 5.2).

/// `Object.type` (`OB_*` in `DNA_object_types.h`, Blender 5.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectKind(pub i16);

impl ObjectKind {
    /// The name Blender's outliner / RNA uses ("MESH", "LIGHT", …).
    pub fn name(self) -> String {
    match self.0 {
        0 => "EMPTY",
        1 => "MESH",
        2 => "CURVE",
        3 => "SURFACE",
        4 => "FONT",
        5 => "META",
        10 => "LIGHT",
        11 => "CAMERA",
        12 => "SPEAKER",
        13 => "LIGHT_PROBE",
        22 => "LATTICE",
        25 => "ARMATURE",
        26 => "GPENCIL",
        27 => "CURVES",
        28 => "POINTCLOUD",
        29 => "VOLUME",
        30 => "GREASEPENCIL",
        other => return format!("TYPE_{other}"),
    }
    .to_string()
    }
}

