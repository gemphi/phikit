//! What a USD attribute or a piece of metadata holds, as the text layer
//! writes it.

/// A value of a USD text layer.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    /// `None`, or an attribute declared with no value.
    #[default]
    Nothing,
    Bool(bool),
    Number(f64),
    /// A string or a token.
    Text(String),
    /// An asset path, `@path@`, and the prim path after it where it has one.
    Asset(String, String),
    /// A prim or property path, `</Root/child>`.
    Path(String),
    /// A tuple, `(1, 2, 3)`.
    Tuple(Vec<Value>),
    /// A list, `[a, b]`.
    List(Vec<Value>),
    /// A dictionary, `{ key = value }` or time samples, `{ 0: v }`.
    Dict(Vec<(String, Value)>),
}

impl Value {
    /// The numbers of a tuple or a list of numbers, or the one number.
    pub fn numbers(&self) -> Vec<f32> {
        match self {
            Value::Number(n) => vec![*n as f32],
            Value::Bool(b) => vec![*b as u8 as f32],
            Value::Tuple(items) | Value::List(items) => items.iter().flat_map(Value::numbers).collect(),
            _ => Vec::new(),
        }
    }

    /// The text of a string or token; nothing for any other value.
    pub fn text(&self) -> &str {
        match self {
            Value::Text(text) => text,
            _ => "",
        }
    }

    /// Whether it is true: a true bool or a number other than nothing.
    pub fn truth(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            _ => false,
        }
    }

    /// Every asset a value names, with the prim path after each: one
    /// for `@a@</p>`, each of a list's.
    pub fn assets(&self) -> Vec<(String, String)> {
        match self {
            Value::Asset(path, prim) => vec![(path.clone(), prim.clone())],
            Value::List(items) => items.iter().flat_map(Value::assets).collect(),
            _ => Vec::new(),
        }
    }
}
