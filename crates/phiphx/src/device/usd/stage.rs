//! USD stage — the root container that composes layers into a prim hierarchy.

use crate::device::usd::prim::UsdPrim;
use crate::device::usd::write::UsdaPrinter;

/// A USD stage: the composed scene rooted at a default prim, with stage
/// metadata (up axis, meters-per-unit).
#[derive(Debug, Clone, PartialEq)]
pub struct UsdStage {
    pub default_prim: Option<String>,
    pub up_axis: UsdUpAxis,
    pub meters_per_unit: f64,
    pub root: UsdPrim,
}

/// The stage's up axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UsdUpAxis {
    #[default]
    Y,
    Z,
}

impl UsdStage {
    /// Create an empty stage with a pseudo-root prim `/`.
    pub fn new() -> Self {
        Self {
            default_prim: None,
            up_axis: UsdUpAxis::Y,
            meters_per_unit: 1.0,
            root: UsdPrim::new("/", "Scope"),
        }
    }

    /// Depth-first traversal of every prim under the root (excluding the
    /// pseudo-root). Mirrors `Stage::Traverse`.
    pub fn traverse(&self) -> Vec<&UsdPrim> {
        let mut out = Vec::new();
        fn walk<'a>(p: &'a UsdPrim, out: &mut Vec<&'a UsdPrim>) {
            for c in &p.children {
                out.push(c);
                walk(c, out);
            }
        }
        walk(&self.root, &mut out);
        out
    }

    /// Find a prim by absolute path.
    pub fn prim_at_path(&self, path: &str) -> Option<&UsdPrim> {
        self.traverse().into_iter().find(|p| p.path == path)
    }

    /// All mesh prims in the stage.
    pub fn meshes(&self) -> Vec<&UsdPrim> {
        self.traverse().into_iter().filter(|p| p.is_mesh()).collect()
    }

    /// Serialize this stage to USDA text via `UsdaPrinter`.
    pub fn export(&self) -> String {
        UsdaPrinter::print(self)
    }
}

impl Default for UsdStage {
    fn default() -> Self {
        Self::new()
    }
}
