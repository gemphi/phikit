//! Object-type → outliner icon table (Blender `tree_element_get_icon`,
//! `OB_*` → `ICON_OUTLINER_OB_*`). Static data only; keyed by the type
//! names `phiphx::blend::ObjectKind::name` produces.

pub struct BOutlineIcon;

static OBJECT_TYPE_ICONS: &[(&str, &str)] = &[
    ("MESH", "mesh"),
    ("CAMERA", "camera"),
    ("LIGHT", "light"),
    ("EMPTY", "empty"),
    ("CURVE", "curve"),
    ("SURFACE", "curve"),
    ("FONT", "text"),
    ("CURVES", "curve"),
    ("SPEAKER", "speaker"),
    ("ARMATURE", "bone"),
    ("LATTICE", "grid_3x3"),
    ("LIGHT_PROBE", "light"),
    ("GREASEPENCIL", "pencil"),
    ("GPENCIL", "pencil"),
    ("VOLUME", "volume"),
    ("META", "mesh"),
];

impl BOutlineIcon {
    /// Icon for an object type; unknown types use the generic empty glyph.
    pub fn for_object_type(ty: &str) -> &'static str {
        OBJECT_TYPE_ICONS
            .iter()
            .find(|(t, _)| *t == ty)
            .map(|(_, icon)| *icon)
            .unwrap_or("empty")
    }
}
