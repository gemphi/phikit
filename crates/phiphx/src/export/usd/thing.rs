//! A built thing as USDA: one Xform named after the thing (the stage's
//! default prim); under it a Mesh for each part and an Xform for each
//! thing inside it, with its transform; and each look worn, once, under
//! the materials scope. A thing inside that wears its own looks refers
//! to its own file (`<name>.usda` beside this one, as the `thing_usd`
//! example writes every thing); one whose looks are swapped is written
//! out in full, as deep as its layers go.

use super::UsdaWriter;
use super::name::{UsdNameSet, usd_word};
use super::stage::UsdMesh;
use crate::format::PreviewSurfaceMaterial;
use crate::geom::shape::ShapePart;
use crate::geom::thing::{ThingNode, ThingTree};

impl ThingTree {
    /// The thing as a USDA layer, Y up, in metres.
    pub fn to_usda(&self) -> String {
        let root = UsdNameSet::clean(&self.name);
        let look_scope = usd_word("look");
        let scope = format!("/{root}/{look_scope}");
        let mut w = UsdaWriter::stage(&root, usd_word("up"));
        w.push_scope(&root, "Xform");
        let mut name = UsdNameSet::default();
        name.take(look_scope);
        layers(&mut w, &self.node, &mut name, &scope);
        w.push_scope(look_scope, "Scope");
        for look in self.looks() {
            let surface = PreviewSurfaceMaterial {
                name: look.name.clone(),
                diffuse_color: look.linear(),
                roughness: look.roughness,
                metallic: look.metallic,
                ..Default::default()
            };
            w.look(&scope, &UsdNameSet::clean(&look.name), &surface);
        }
        w.pop_scope();
        w.pop_scope();
        w.get_text()
    }
}

/// One level's layers, each named apart from its siblings.
fn layers(w: &mut UsdaWriter, node: &[ThingNode], name: &mut UsdNameSet, scope: &str) {
    for n in node {
        match n {
            ThingNode::Mesh { part, look } => {
                let prim = name.take(&part.name);
                let by_point = by_point(part);
                w.mesh(&UsdMesh {
                    name: &prim,
                    point: &part.point,
                    count: &part.count,
                    index: &part.index,
                    normal: by_point.as_deref().unwrap_or(&part.normal),
                    normal_by_point: by_point.is_some(),
                    colour: part.colour,
                    look: Some(format!("{scope}/{}", UsdNameSet::clean(&look.name))),
                    ..Default::default()
                });
            }
            ThingNode::Group { name: group, matrix, node, from } => {
                let prim = name.take(group);
                match from {
                    Some(from) => w.push_scope_with(&prim, "Xform", &[&format!("prepend references = @./{from}.usda@")]),
                    None => w.push_scope(&prim, "Xform"),
                }
                w.transform(matrix);
                if from.is_none() {
                    layers(w, node, &mut UsdNameSet::default(), scope);
                }
                w.pop_scope();
            }
        }
    }
}

/// One normal for each point, where every corner at a point has the same
/// one (a smooth part: a ball, a pill, a rounded box); none where a part
/// has an edge (a can's rim, a letter's faces).
fn by_point(part: &ShapePart) -> Option<Vec<[f32; 3]>> {
    let mut out: Vec<Option<[f32; 3]>> = vec![None; part.point.len()];
    for (&index, &normal) in part.index.iter().zip(&part.normal) {
        let seen = out.get_mut(index as usize)?;
        match seen {
            Some(n) if *n != normal => return None,
            _ => *seen = Some(normal),
        }
    }
    out.into_iter().collect()
}
