//! USDA text serializer — converts a `UsdStage` back to the human-readable
//! `.usda` format. Mirrors the export path of Apple's USDKit framework.

pub mod mesh;
pub(crate) mod point_cloud;
pub mod prim;
pub mod scene;
pub mod script;
pub mod value;
mod writer;

pub use writer::UsdaWriter;

use crate::device::usd::stage::{UsdStage, UsdUpAxis};

pub use value::print_value;

/// Serializes a `UsdStage` to USDA text using a `String` buffer with
/// indentation tracking.
pub struct UsdaPrinter {
    indent: usize,
}

impl UsdaPrinter {
    /// Serialize a `UsdStage` into USDA text beginning with the `#usda 1.0`
    /// magic header.
    pub fn print(stage: &UsdStage) -> String {
        let mut printer = Self { indent: 0 };
        let mut out = String::new();
        out.push_str("#usda 1.0\n");
        printer.print_stage_metadata(stage, &mut out);
        for child in &stage.root.children {
            out.push('\n');
            prim::print_prim(child, &mut printer.indent, &mut out);
        }
        out
    }

    fn print_stage_metadata(&mut self, stage: &UsdStage, out: &mut String) {
        let mut entries: Vec<String> = Vec::new();
        if let Some(ref dp) = stage.default_prim {
            entries.push(format!("    defaultPrim = \"{}\"", dp));
        }
        if stage.meters_per_unit != 1.0 {
            entries.push(format!(
                "    metersPerUnit = {}",
                value::print_value(&super::attribute::UsdValue::Float(stage.meters_per_unit))
            ));
        }
        if stage.up_axis != UsdUpAxis::Y {
            let axis = match stage.up_axis {
                UsdUpAxis::Y => "Y",
                UsdUpAxis::Z => "Z",
            };
            entries.push(format!("    upAxis = \"{}\"", axis));
        }
        if !entries.is_empty() {
            out.push_str("(\n");
            for entry in &entries {
                out.push_str(entry);
                out.push('\n');
            }
            out.push_str(")\n");
        }
    }

    /// Serialize a `UsdValue` to its USDA text representation.
    /// Public convenience method delegating to `value::print_value`.
    pub fn print_value(value: &super::attribute::UsdValue) -> String {
        value::print_value(value)
    }
}
