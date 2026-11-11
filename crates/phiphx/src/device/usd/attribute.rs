//! USD attributes — typed named values authored on a prim.

/// A typed USD attribute value. Covers the scalar and array value types most
/// commonly authored on geometry and material prims.
#[derive(Debug, Clone, PartialEq)]
pub enum UsdValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Token(String),
    String(String),
    Float3([f32; 3]),
    Float3Array(Vec<[f32; 3]>),
    Float2Array(Vec<[f32; 2]>),
    IntArray(Vec<i32>),
    /// A token array (`token[]`), used for attributes like `xformOpOrder`.
    StringArray(Vec<String>),
    /// A matrix4d, 16 numbers in USD's text order (row by row, translation in the last row).
    Matrix([f64; 16]),
}

/// A named, typed attribute on a USD prim.
#[derive(Debug, Clone, PartialEq)]
pub struct UsdAttribute {
    pub name: String,
    pub value: UsdValue,
}

impl UsdAttribute {
    pub fn new(name: impl Into<String>, value: UsdValue) -> Self {
        Self { name: name.into(), value }
    }
}
