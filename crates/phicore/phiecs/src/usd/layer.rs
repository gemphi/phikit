//! A USD text layer (`.usda`) read into its prims: each with how it is
//! specified (`def`, `over`, `class`), its type, its name, its metadata,
//! its attributes and relationships, and its children. What the reader
//! does not know (a variant set's body, say) is stepped over whole.

use super::read::Reader;
use super::token::split;
use super::value::Value;

/// How a prim is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Spec {
    #[default]
    Def,
    Over,
    Class,
}

/// A property: an attribute or a relationship, its type, its value and
/// whether it connects to another (`.connect`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Property {
    pub name: String,
    pub kind: String,
    pub value: Value,
}

/// A prim as one layer writes it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Prim {
    pub spec: Spec,
    pub kind: String,
    pub name: String,
    pub meta: Vec<(String, Value)>,
    pub property: Vec<Property>,
    pub child: Vec<Prim>,
}

impl Prim {
    /// The value of a piece of its metadata.
    pub fn meta(&self, key: &str) -> Option<&Value> {
        self.meta.iter().find(|(name, _)| name == key).map(|(_, value)| value)
    }

    /// The value of one of its properties.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.property.iter().find(|p| p.name == name).map(|p| &p.value)
    }

    /// Its child of that name.
    pub fn child(&self, name: &str) -> Option<&Prim> {
        self.child.iter().find(|c| c.name == name)
    }
}

/// A whole layer: its own metadata and its top prims.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layer {
    pub meta: Vec<(String, Value)>,
    pub prim: Vec<Prim>,
}

impl Layer {
    /// Read a layer from its text.
    pub fn read(text: &str) -> Layer {
        let token = split(text);
        let mut read = Reader::new(token);
        // The header, `#usda 1.0`, is a comment to the splitter.
        let meta = read.meta();
        let mut prim = Vec::new();
        while !read.done() {
            match read.prim() {
                Some(one) => prim.push(one),
                None => read.step(),
            }
        }
        Layer { meta, prim }
    }

    /// Its default prim's name, as its metadata says.
    pub fn default_prim(&self) -> &str {
        self.meta.iter().find(|(key, _)| key == "defaultPrim").map_or("", |(_, value)| value.text())
    }

    /// The prim at a path such as `/Root/child`.
    pub fn find(&self, path: &str) -> Option<&Prim> {
        let mut step = path.trim_start_matches('/').split('/').filter(|s| !s.is_empty());
        let first = self.prim.iter().find(|p| Some(p.name.as_str()) == step.clone().next())?;
        step.next();
        step.try_fold(first, |prim, name| prim.child(name))
    }
}
