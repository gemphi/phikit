//! What every placed prim opens with: its name, its transform, its visibility.
use super::name::UsdName;
use super::out::UsdOut;
use super::text::UsdText;

/// The common start of an Xform, camera or light prim.
pub(crate) struct UsdHead {
    /// Unique prim name.
    pub id: String,
    /// The Blender name, kept as `displayName` when `id` is not it.
    pub raw: String,
    /// Placement under the parent prim, USD row layout (`UsdText::matrix`).
    pub matrix: [[f32; 4]; 4],
    /// Disabled in renders or viewports in the Blender file.
    pub hidden: bool,
}

impl UsdHead {
    /// Open `kind` with this head, `meta` after the display name, then the
    /// transform and the visibility.
    pub fn open(&self, out: &mut UsdOut, kind: &str, meta: &[String]) {
        let mut all = UsdName::meta(&self.raw, &self.id);
        all.extend_from_slice(meta);
        out.open(kind, &self.id, &all);
        out.line(&format!("matrix4d xformOp:transform = {}", UsdText::matrix(&self.matrix)));
        out.line("uniform token[] xformOpOrder = [\"xformOp:transform\"]");
        if self.hidden {
            out.line("token visibility = \"invisible\"");
        }
    }
}
