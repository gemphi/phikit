//! Collections and objects as the Xform tree.
use crate::device::usd::write::scene::camera::UsdCamera;
use crate::device::usd::write::scene::head::UsdHead;
use crate::device::usd::write::scene::light::UsdLight;
use crate::device::usd::write::scene::mesh::UsdMesh;
use crate::device::usd::write::scene::name::UsdName;
use crate::device::usd::write::scene::node::NodeKind;
use crate::device::usd::write::scene::text::UsdText;
use crate::device::usd::write::scene::UsdSceneWriter;
use crate::scene::SceneCollection;

impl UsdSceneWriter<'_> {
    /// A collection as a `Scope` and what it holds. The scene's top
    /// collection (`top`) is /World itself, not a Scope under it.
    ///
    /// An object sits where its parent puts it, so a collection opens only
    /// the objects it holds that have no parent in the scene; the rest are
    /// reached through their parents. An object linked into several
    /// collections is written in the first.
    pub(super) fn collection(&mut self, c: &SceneCollection, parent: &str, top: bool) {
        let mut path = parent.to_string();
        if !top {
            let id = self.names.claim(parent, &c.name);
            let mut meta = UsdName::meta(&c.name, &id);
            if c.exclude {
                meta.push("active = false".to_string());
            }
            self.out.open("Scope", &id, &meta);
            path = format!("{parent}/{id}");
        }
        // Mesh nodes come first in the list, so an object index is its node index.
        let held = c.object_indices.iter().copied().chain(c.extra_names.iter().filter_map(|n| self.node.find(n)));
        for i in held.collect::<Vec<_>>() {
            if self.node.node.get(i).is_some_and(|n| n.parent.is_none()) {
                self.node(i, &path);
            }
        }
        for child in &c.children {
            self.collection(child, &path, false);
        }
        if !top {
            self.out.close();
        }
    }

    /// Node `i` under `parent`, its children inside it.
    pub(super) fn node(&mut self, i: usize, parent: &str) {
        if std::mem::replace(&mut self.emitted[i], true) {
            return;
        }
        let n = &self.node.node[i];
        let id = self.names.claim(parent, &n.name);
        let path = format!("{parent}/{id}");
        let head = UsdHead { raw: n.name.clone(), matrix: n.local(&self.node), hidden: self.hidden.contains(n.name.as_str()), id };
        let (kind, child) = (n.kind, n.child.clone());
        let scene = self.scene;
        match kind {
            NodeKind::Mesh(k) => {
                let object = &scene.objects[k];
                head.open(&mut self.out, "Xform", &[]);
                // The geometry prim is a child of the object, so no child object may take its name.
                self.names.claim(&path, "Mesh");
                if !object.mesh.vertices.is_empty() {
                    match self.proto.path_of(k) {
                        Some(proto) => {
                            self.out.open("Xform", "Mesh", &["instanceable = true".to_string(), format!("prepend references = <{proto}>")]);
                            self.out.close();
                        }
                        None => UsdMesh::write(&mut self.out, object, &self.material),
                    }
                }
            }
            NodeKind::Camera(k) => {
                let r = &scene.render;
                UsdCamera::open(&mut self.out, &head, &scene.cameras[k], r.resolution_x.max(1) as f32 / r.resolution_y.max(1) as f32);
            }
            NodeKind::Light(k) => UsdLight::open(&mut self.out, &head, &scene.lights[k]),
            NodeKind::Empty(k) => {
                let kind = UsdText::quote(&scene.empties[k].display_type);
                head.open(&mut self.out, "Xform", &[format!("customData = {{string emptyType = {kind}}}")]);
            }
        }
        for c in child {
            self.node(c, &path);
        }
        self.out.close();
    }
}
