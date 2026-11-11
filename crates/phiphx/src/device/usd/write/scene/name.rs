//! Blender names as USD prim names.
use super::text::UsdText;
use std::collections::HashSet;

/// Blender allows any text in a name ("Lot/Car", "Cube.001", "Wall é");
/// a USD prim name is `[A-Za-z_][A-Za-z0-9_]*`.
pub(crate) struct UsdName;

impl UsdName {
    /// The identifier for `raw`: every character outside `[A-Za-z0-9_]`
    /// becomes `_`, and a leading digit (or an empty name) gets a `_` in
    /// front. Pure, so the same name always gives the same identifier;
    /// [`UsdNameBook`] separates the ones that land on the same text.
    pub fn id(raw: &str) -> String {
        let mut id: String = raw.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
        if id.chars().next().is_none_or(|c| c.is_ascii_digit()) {
            id.insert(0, '_');
        }
        id
    }

    /// Metadata that keeps the Blender spelling when the identifier is not it.
    pub fn meta(raw: &str, id: &str) -> Vec<String> {
        if raw == id { Vec::new() } else { vec![format!("displayName = {}", UsdText::quote(raw))] }
    }
}

/// Every prim path handed out, so two siblings never share a name.
#[derive(Debug, Default)]
pub(crate) struct UsdNameBook {
    taken: HashSet<String>,
}

impl UsdNameBook {
    /// The unique child name under `parent` for `raw`: its identifier, then
    /// `_1`, `_2`, ... when a sibling already holds it.
    pub fn claim(&mut self, parent: &str, raw: &str) -> String {
        let base = UsdName::id(raw);
        let mut name = base.clone();
        let mut n = 0;
        while !self.taken.insert(format!("{parent}/{name}")) {
            n += 1;
            name = format!("{base}_{n}");
        }
        name
    }
}
