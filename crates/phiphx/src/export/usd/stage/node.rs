//! The scene's placed things as one list with a parent for each.
//!
//! `SceneModel` keeps meshes, cameras, lights and empties apart; USD wants
//! one Xform tree. The parent chain is `SceneModel::parents` (by name).
use crate::scene::SceneModel;
use glam::Mat4;
use std::collections::HashMap;

/// Which list of the scene a node came from, and where in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum NodeKind {
    Mesh(usize),
    Camera(usize),
    Light(usize),
    Empty(usize),
}

/// One placed thing.
#[derive(Debug)]
pub(super) struct Node {
    pub kind: NodeKind,
    pub name: String,
    /// `matrix_world`, Blender's `m[col][row]` read as a `Mat4`.
    pub world: Mat4,
    pub parent: Option<usize>,
    pub child: Vec<usize>,
}

impl Node {
    /// Placement under the parent: `parent⁻¹ · world`, or the world matrix
    /// for a root. Blender's `parentinv` is already folded into `world`.
    pub fn local(&self, list: &NodeList) -> [[f32; 4]; 4] {
        let m = match self.parent {
            Some(p) => list.node[p].world.inverse() * self.world,
            None => self.world,
        };
        m.to_cols_array_2d()
    }
}

/// Every node of a scene, in mesh, camera, light, empty order.
#[derive(Debug, Default)]
pub(super) struct NodeList {
    pub node: Vec<Node>,
    by_name: HashMap<String, usize>,
}

impl NodeList {
    pub fn of(scene: &SceneModel) -> Self {
        let mut list = Self::default();
        for (i, o) in scene.objects.iter().enumerate() {
            list.push(NodeKind::Mesh(i), &o.name, o.matrix_world);
        }
        for (i, c) in scene.cameras.iter().enumerate() {
            list.push(NodeKind::Camera(i), &c.name, c.matrix_world);
        }
        for (i, l) in scene.lights.iter().enumerate() {
            list.push(NodeKind::Light(i), &l.name, l.matrix_world);
        }
        for (i, e) in scene.empties.iter().enumerate() {
            list.push(NodeKind::Empty(i), &e.name, e.matrix_world);
        }
        list.link(scene);
        list
    }

    fn push(&mut self, kind: NodeKind, name: &str, world: [[f32; 4]; 4]) {
        self.by_name.entry(name.to_string()).or_insert(self.node.len());
        self.node.push(Node { kind, name: name.to_string(), world: Mat4::from_cols_array_2d(&world), parent: None, child: Vec::new() });
    }

    /// Set `parent` / `child` from the scene's `(child, parent)` pairs. A
    /// parent that is not a node (an armature the scene drops) leaves its
    /// child a root; a loop in the pairs is broken where it closes.
    fn link(&mut self, scene: &SceneModel) {
        for i in 0..self.node.len() {
            let Some(p) = scene.parent_of(&self.node[i].name).and_then(|n| self.by_name.get(n)).copied() else { continue };
            if p != i && !self.is_below(p, i) {
                self.node[i].parent = Some(p);
                self.node[p].child.push(i);
            }
        }
    }

    /// Whether `a` hangs under `b` (or is `b`) so far.
    fn is_below(&self, a: usize, b: usize) -> bool {
        let mut at = Some(a);
        for _ in 0..=self.node.len() {
            match at {
                Some(x) if x == b => return true,
                Some(x) => at = self.node[x].parent,
                None => return false,
            }
        }
        true
    }

    /// The node called `name`, if any.
    pub fn find(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }
}
