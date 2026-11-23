//! A stage: a layer opened from a file with what it references
//! composed into it, as USD composes a scene. A prim that references a
//! text layer (`.usda`) takes that layer's prim (the one named after the
//! `@path@`, or its default prim) as what it is, and its own opinions
//! stand over it: its properties win, and a child of the same name is
//! composed into the referenced one. Any other layer (`.usdc`, `.usdz`,
//! a `.usd`, text or not) is the prim's asset, kept unopened for
//! whoever draws meshes to read (phiusd): those hold meshes, which the
//! world needs only to draw, and they are the largest by far.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::layer::{Layer, Prim, Spec};
use super::value::Value;

/// How deep references are followed: a reference to itself stops here.
const DEEP: usize = 16;

/// A composed scene.
#[derive(Debug, Clone, Default)]
pub struct Stage {
    /// The file it was opened from.
    pub file: PathBuf,
    /// Its root layer's metadata.
    pub meta: Vec<(String, Value)>,
    /// Its prims, each with what it references composed in.
    pub prim: Vec<Prim>,
}

/// Where a prim's unopened binary asset is, kept as a property of it.
pub const ASSET: &str = "phiecs:asset";

impl Stage {
    /// Open a stage from a text layer and compose it. `None` where the
    /// file cannot be read.
    pub fn open(file: &Path) -> Option<Stage> {
        let mut cache = HashMap::new();
        let layer = load(file, &mut cache)?;
        let base = file.parent().map(Path::to_path_buf).unwrap_or_default();
        let prim = layer.prim.iter().map(|p| compose(p, &base, &mut cache, 0)).collect();
        Some(Stage { file: file.to_path_buf(), meta: layer.meta.clone(), prim })
    }

    /// Its default prim, as its metadata says, or its first.
    pub fn root(&self) -> Option<&Prim> {
        let name = self.meta.iter().find(|(k, _)| k == "defaultPrim").map_or("", |(_, v)| v.text());
        self.prim.iter().find(|p| p.name == name).or(self.prim.first())
    }
}

/// A layer read from its file, once.
fn load(file: &Path, cache: &mut HashMap<PathBuf, Option<Layer>>) -> Option<Layer> {
    cache.entry(file.to_path_buf()).or_insert_with(|| text(file).map(|text| Layer::read(&text))).clone()
}

/// The text of a layer, where the file is a text layer named `.usda`.
fn text(file: &Path) -> Option<String> {
    let named = file.extension().is_some_and(|e| e.eq_ignore_ascii_case("usda"));
    let bytes = std::fs::read(file).ok().filter(|_| named)?;
    bytes.starts_with(b"#usda").then(|| String::from_utf8_lossy(&bytes).into_owned())
}

/// A prim with what it references composed into it, and its children
/// the same, down to `DEEP` references.
fn compose(prim: &Prim, base: &Path, cache: &mut HashMap<PathBuf, Option<Layer>>, depth: usize) -> Prim {
    let named = prim.meta("references").or(prim.meta("payload")).map(Value::assets).unwrap_or_default();
    let mut out = prim.clone();
    out.child = Vec::new();
    for (asset, path) in named.into_iter().filter(|_| depth < DEEP) {
        let file = base.join(&asset);
        let found = load(&file, cache).and_then(|layer| {
            let name = match path.is_empty() {
                true => format!("/{}", layer.default_prim()),
                false => path,
            };
            layer.find(&name).cloned()
        });
        match found {
            Some(under) => {
                let deeper = file.parent().map(Path::to_path_buf).unwrap_or_default();
                let under = compose(&under, &deeper, cache, depth + 1);
                out = over(&under, &out);
            }
            None => out.property.push(super::layer::Property { name: ASSET.to_string(), kind: "asset".to_string(), value: Value::Text(file.to_string_lossy().into_owned()) }),
        }
    }
    for child in &prim.child {
        let child = compose(child, base, cache, depth);
        match out.child.iter_mut().find(|c| c.name == child.name) {
            Some(there) => *there = over(there, &child),
            None => out.child.push(child),
        }
    }
    out
}

/// `strong` over `weak`: its name, its spec where it defines, its type
/// where it has one, its properties winning, its children composed in.
fn over(weak: &Prim, strong: &Prim) -> Prim {
    let mut out = weak.clone();
    out.name = strong.name.clone();
    out.spec = match strong.spec {
        Spec::Over => weak.spec,
        spec => spec,
    };
    if !strong.kind.is_empty() {
        out.kind = strong.kind.clone();
    }
    out.meta.retain(|(k, _)| strong.meta(k).is_none());
    out.meta.extend(strong.meta.iter().cloned());
    for p in &strong.property {
        match out.property.iter_mut().find(|q| q.name == p.name) {
            Some(q) => *q = p.clone(),
            None => out.property.push(p.clone()),
        }
    }
    for c in &strong.child {
        match out.child.iter_mut().find(|d| d.name == c.name) {
            Some(d) => *d = over(d, c),
            None => out.child.push(c.clone()),
        }
    }
    out
}
