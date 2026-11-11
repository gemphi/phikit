//! Value and attribute serialization for USDA text output.

use crate::device::usd::attribute::{UsdAttribute, UsdValue};

/// Serialize a `UsdValue` to its USDA text representation.
pub fn print_value(value: &UsdValue) -> String {
    match value {
        UsdValue::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        UsdValue::Int(i) => i.to_string(),
        UsdValue::Float(f) => format_f64(*f),
        UsdValue::Token(t) => format!("\"{}\"", t),
        UsdValue::String(s) => format!("\"{}\"", s),
        UsdValue::Float3(v) => {
            format!("({}, {}, {})", format_f32(v[0]), format_f32(v[1]), format_f32(v[2]))
        }
        UsdValue::Float3Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let items: Vec<String> = arr
                .iter()
                .map(|v| format!("({}, {}, {})", format_f32(v[0]), format_f32(v[1]), format_f32(v[2])))
                .collect();
            format!("[{}]", items.join(", "))
        }
        UsdValue::Float2Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let items: Vec<String> = arr
                .iter()
                .map(|v| format!("({}, {})", format_f32(v[0]), format_f32(v[1])))
                .collect();
            format!("[{}]", items.join(", "))
        }
        UsdValue::IntArray(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let items: Vec<String> = arr.iter().map(|i| i.to_string()).collect();
            format!("[{}]", items.join(", "))
        }
        UsdValue::Matrix(m) => {
            let row = |r: usize| format!("({}, {}, {}, {})", format_f64(m[r * 4]), format_f64(m[r * 4 + 1]), format_f64(m[r * 4 + 2]), format_f64(m[r * 4 + 3]));
            format!("({}, {}, {}, {})", row(0), row(1), row(2), row(3))
        }
        UsdValue::StringArray(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let items: Vec<String> = arr.iter().map(|s| format!("\"{}\"", s)).collect();
            format!("[{}]", items.join(", "))
        }
    }
}

/// Return the USDA type prefix for a given value variant.
pub fn type_prefix(value: &UsdValue) -> &'static str {
    match value {
        UsdValue::Bool(_) => "bool",
        UsdValue::Int(_) => "int",
        UsdValue::Float(_) => "double",
        UsdValue::Token(_) => "token",
        UsdValue::String(_) => "string",
        UsdValue::Float3(_) => "float3",
        UsdValue::Float3Array(_) => "point3f[]",
        UsdValue::Float2Array(_) => "texCoord2f[]",
        UsdValue::IntArray(_) => "int[]",
        UsdValue::StringArray(_) => "token[]",
        UsdValue::Matrix(_) => "matrix4d",
    }
}

/// Serialize an attribute (type prefix + name + value) at the given indent level.
pub fn print_attribute(attr: &UsdAttribute, indent: usize, out: &mut String) {
    let indent_str = "    ".repeat(indent);
    let tp = type_prefix(&attr.value);
    let value_str = print_value(&attr.value);
    out.push_str(&indent_str);
    out.push_str(&format!("{} {} = {}\n", tp, attr.name, value_str));
}

/// Format an f64 value, ensuring it always has a decimal point.
fn format_f64(v: f64) -> String {
    let s = v.to_string();
    if s.contains('.') { s } else { format!("{}.0", s) }
}

/// Format an f32 value, ensuring it always has a decimal point.
fn format_f32(v: f32) -> String {
    let s = v.to_string();
    if s.contains('.') { s } else { format!("{}.0", s) }
}
