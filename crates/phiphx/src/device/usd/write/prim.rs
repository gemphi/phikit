//! Prim serialization for USDA text output.

use crate::device::usd::prim::UsdPrim;
use crate::device::usd::write::value::print_attribute;

/// Serialize a `UsdPrim` as a `def <type_name> "<name>"` block with indented
/// attributes and recursively nested children.
pub fn print_prim(prim: &UsdPrim, indent: &mut usize, out: &mut String) {
    let indent_str = "    ".repeat(*indent);
    out.push_str(&indent_str);
    out.push_str(&format!("def {} \"{}\"\n", prim.type_name, prim.name()));
    out.push_str(&indent_str);
    out.push_str("{\n");
    *indent += 1;
    for attr in &prim.attributes {
        print_attribute(attr, *indent, out);
    }
    for child in &prim.children {
        if !prim.attributes.is_empty() || &prim.children[0] as *const _ != child as *const _ {
            out.push('\n');
        }
        print_prim(child, indent, out);
    }
    *indent -= 1;
    out.push_str(&indent_str);
    out.push_str("}\n");
}
