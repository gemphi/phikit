//! Meshes the scene shares, written once as prototypes.
//!
//! Collection instances and linked duplicates hold the same
//! `Arc<SceneMesh>` (`scene/share.rs`); here that sharing becomes USD native
//! instancing: one `Xform` under `/World/Prototype` that every user
//! references with `instanceable = true`.
use super::material::MaterialBook;
use super::mesh::UsdMesh;
use super::name::{UsdName, UsdNameBook};
use super::out::UsdOut;
use crate::scene::SceneModel;
use std::collections::HashMap;
use std::sync::Arc;

/// One shared mesh: its prim name and the object whose slots it is written with.
struct Proto {
    id: String,
    first: usize,
}

/// The shared meshes of a scene.
#[derive(Default)]
pub(super) struct ProtoList {
    item: Vec<Proto>,
    /// Object index to the `item` it instances.
    of_object: HashMap<usize, usize>,
}

impl ProtoList {
    pub const ROOT: &'static str = "/World/Prototype";

    /// Group the objects by mesh; a mesh two or more objects hold is shared.
    pub fn of(scene: &SceneModel, names: &mut UsdNameBook) -> Self {
        let mut at: HashMap<*const crate::scene::SceneMesh, Vec<usize>> = HashMap::new();
        let mut order = Vec::new();
        for (i, o) in scene.objects.iter().enumerate() {
            if o.mesh.vertices.is_empty() {
                continue;
            }
            let group = at.entry(Arc::as_ptr(&o.mesh)).or_default();
            if group.is_empty() {
                order.push(Arc::as_ptr(&o.mesh));
            }
            group.push(i);
        }
        let mut list = Self::default();
        for key in order {
            let group = &at[&key];
            if group.len() < 2 {
                continue;
            }
            let id = names.claim(Self::ROOT, &scene.objects[group[0]].name);
            for &i in group {
                list.of_object.insert(i, list.item.len());
            }
            list.item.push(Proto { id, first: group[0] });
        }
        list
    }

    /// The prototype path object `object` instances, if its mesh is shared.
    pub fn path_of(&self, object: usize) -> Option<String> {
        self.of_object.get(&object).map(|&p| format!("{}/{}", Self::ROOT, self.item[p].id))
    }

    /// The `Scope` of prototypes; nothing when no mesh is shared.
    pub fn write(&self, out: &mut UsdOut, scene: &SceneModel, book: &MaterialBook) {
        if self.item.is_empty() {
            return;
        }
        out.open("Scope", "Prototype", &[]);
        for p in &self.item {
            let first = &scene.objects[p.first];
            out.open("Xform", &p.id, &UsdName::meta(&first.name, &p.id));
            UsdMesh::write(out, first, book);
            out.close();
        }
        out.close();
    }
}
