//! A shape from `data/shape.tsv` as USDA: one Xform named after the shape
//! (the stage's default prim), a Mesh per part, and each part's material
//! under the materials scope inside it.

use super::UsdaWriter;
use super::name::{UsdNameSet, usd_word};
use super::stage::UsdMesh;
use crate::format::PreviewSurfaceMaterial;
use crate::geom::shape::Shape;

impl Shape {
    /// The shape as a USDA layer, Y up, in metres.
    pub fn to_usda(&self) -> String {
        let root = UsdNameSet::clean(&self.name);
        let look_scope = usd_word("look");
        let scope = format!("/{root}/{look_scope}");
        let mut w = UsdaWriter::stage(&root, usd_word("up"));
        w.push_scope(&root, "Xform");
        let part = self.part();
        let mut name = UsdNameSet::default();
        name.take(look_scope);
        let prim: Vec<String> = part.iter().map(|p| name.take(&p.name)).collect();
        for (p, prim) in part.iter().zip(&prim) {
            w.mesh(&UsdMesh {
                name: prim,
                point: &p.point,
                count: &p.count,
                index: &p.index,
                normal: &p.normal,
                colour: p.colour,
                look: Some(format!("{scope}/{prim}")),
                ..Default::default()
            });
        }
        w.push_scope(look_scope, "Scope");
        for (p, prim) in part.iter().zip(&prim) {
            let look = PreviewSurfaceMaterial {
                name: p.name.clone(),
                diffuse_color: p.colour,
                roughness: p.roughness,
                metallic: p.metallic,
                ..Default::default()
            };
            w.look(&scope, prim, &look);
        }
        w.pop_scope();
        w.pop_scope();
        w.get_text()
    }
}
