//! `SceneModel` to a USD text layer: the one scene writer.
//!
//! One walk over what the scene holds, so what the viewport draws and what
//! a USD tool opens are the same list of things:
//!
//! ```text
//! /World                       Xform, the layer's defaultPrim
//!   /<Collection>...           Scope per collection, objects inside
//!     /<Object>                Xform; children nest by Blender's parent chain
//!       /Mesh                  UsdGeomMesh, or an instance of a prototype
//!     /<Camera>, /<Light>      UsdGeomCamera, UsdLux lights
//!   /Material/<Name>           UsdPreviewSurface materials
//!   /Prototype/<Name>          meshes the scene shares
//!   /Environment               DomeLight of the world
//! ```
//!
//! ## Frame and units
//!
//! `upAxis = "Z"` and `metersPerUnit = 1`: Blender is Z-up and counts one
//! unit as a metre (its own USD exporter writes the same), and the scene's
//! matrices are already in that frame, so no axis fix-up is applied.
//! Lens and film stay in millimetres (`camera.rs`); light units are
//! explained in `light.rs`.
//!
//! Not written: UsdSkel, animation (`timeSamples`), modifiers' inputs
//! (meshes are the evaluated result), packed image bytes, node graphs.
pub(crate) mod camera;
pub mod check;
pub(crate) mod head;
pub(crate) mod light;
pub(crate) mod material;
pub(crate) mod mesh;
pub(crate) mod name;
pub(crate) mod node;
pub(crate) mod out;
pub(crate) mod proto;
pub(crate) mod text;
pub(crate) mod texture;
pub(crate) mod tree;
pub(crate) mod world;

use crate::scene::SceneModel;
use material::MaterialBook;
use name::UsdNameBook;
use node::NodeList;
use out::UsdOut;
use proto::ProtoList;
use std::collections::HashSet;
use text::UsdText;

/// A scene being written. Build with [`UsdSceneWriter::write`].
pub struct UsdSceneWriter<'a> {
    scene: &'a SceneModel,
    out: UsdOut,
    names: UsdNameBook,
    node: NodeList,
    material: MaterialBook<'a>,
    proto: ProtoList,
    emitted: Vec<bool>,
    /// Names of objects the file disables in renders or viewports.
    hidden: HashSet<&'a str>,
}

impl<'a> UsdSceneWriter<'a> {
    /// The USDA text of `scene`.
    pub fn write(scene: &'a SceneModel) -> String {
        let mut names = UsdNameBook::default();
        // The fixed children of /World claim their names before any object does.
        for fixed in ["Material", "Prototype", "Environment"] {
            names.claim("/World", fixed);
        }
        let node = NodeList::of(scene);
        let sel = &scene.selection;
        let mut stage = Self {
            scene,
            out: UsdOut::default(),
            material: MaterialBook::of(scene, &mut names),
            proto: ProtoList::of(scene, &mut names),
            emitted: vec![false; node.node.len()],
            hidden: sel.render_disabled.iter().chain(&sel.viewport_disabled).map(String::as_str).collect(),
            names,
            node,
        };
        stage.header();
        stage.out.open("Xform", "World", &[]);
        stage.body();
        stage.out.close();
        stage.out.finish()
    }

    /// Layer metadata: the stage's frame and units, the clock, and where the
    /// file came from.
    fn header(&mut self) {
        let s = self.scene;
        let out = &mut self.out;
        out.raw("#usda 1.0");
        out.raw("(");
        out.raw("    defaultPrim = \"World\"");
        out.raw("    upAxis = \"Z\"");
        out.raw("    metersPerUnit = 1");
        out.raw(&format!("    framesPerSecond = {}", UsdText::float(s.timeline.fps)));
        out.raw(&format!("    timeCodesPerSecond = {}", UsdText::float(s.timeline.fps)));
        out.raw("    doc = \"Written by phiphx from a SceneModel; modifiers are evaluated.\"");
        out.raw("    customLayerData = {");
        out.raw(&format!("        string scene = {}", UsdText::quote(&s.report.scene)));
        out.raw(&format!("        string renderCamera = {}", UsdText::quote(&s.render.camera)));
        out.raw("    }");
        out.raw(")");
        out.raw("");
    }

    /// Everything under /World.
    fn body(&mut self) {
        let scene = self.scene;
        for root in &scene.collections {
            self.collection(root, "/World", true);
        }
        for i in 0..self.node.node.len() {
            if !self.emitted[i] && self.node.node[i].parent.is_none() {
                self.node(i, "/World");
            }
        }
        self.material.write(&mut self.out);
        self.proto.write(&mut self.out, scene, &self.material);
        world::UsdWorld::write(&mut self.out, "Environment", &scene.world);
    }
}

impl SceneModel {
    /// This scene as a USD ASCII layer (see [`UsdSceneWriter`]).
    pub fn to_usda(&self) -> String {
        UsdSceneWriter::write(self)
    }
}
