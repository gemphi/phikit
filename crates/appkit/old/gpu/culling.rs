//! Culling — BVH-based frustum culling for the viewport.

use crate::math::{SCNBoundingBox, SCNVector3};

impl SCNBoundingBox {
    /// Checks if this bounding box intersects the 6 frustum planes.
    /// Each plane is stored as `[ax, ay, az, d]` coefficients.
    pub fn intersects_frustum(&self, planes: &[[f32; 4]; 6]) -> bool {
        for plane in planes {
            let px = if plane[0] >= 0.0 {
                self.max.x
            } else {
                self.min.x
            };
            let py = if plane[1] >= 0.0 {
                self.max.y
            } else {
                self.min.y
            };
            let pz = if plane[2] >= 0.0 {
                self.max.z
            } else {
                self.min.z
            };

            let dot = plane[0] * px + plane[1] * py + plane[2] * pz + plane[3];
            if dot < 0.0 {
                // Completely outside this plane
                return false;
            }
        }
        true
    }
}

/// Bounding Volume Hierarchy node.
#[derive(Debug, Clone)]
pub struct BVHNode {
    pub bounds: SCNBoundingBox,
    pub entity_id: Option<u64>,
    pub children: Vec<BVHNode>,
}

impl BVHNode {
    pub fn new_leaf(bounds: SCNBoundingBox, entity_id: u64) -> Self {
        Self {
            bounds,
            entity_id: Some(entity_id),
            children: Vec::new(),
        }
    }

    pub fn new_branch(bounds: SCNBoundingBox, children: Vec<BVHNode>) -> Self {
        Self {
            bounds,
            entity_id: None,
            children,
        }
    }

    pub fn cull_visible(&self, planes: &[[f32; 4]; 6], visible: &mut Vec<u64>) {
        if !self.bounds.intersects_frustum(planes) {
            return;
        }

        if let Some(id) = self.entity_id {
            visible.push(id);
        } else {
            for child in &self.children {
                child.cull_visible(planes, visible);
            }
        }
    }
}

/// Bounding Volume Hierarchy for spatial culling.
#[derive(Debug, Clone)]
pub struct BVH {
    pub root: Option<BVHNode>,
}

impl BVH {
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Build a BVH tree from a list of entities and their bounding boxes.
    pub fn build(&mut self, entities: &[(u64, SCNBoundingBox)]) {
        if entities.is_empty() {
            self.root = None;
            return;
        }

        let mut nodes: Vec<BVHNode> = entities
            .iter()
            .map(|(id, box_val)| BVHNode::new_leaf(*box_val, *id))
            .collect();

        while nodes.len() > 1 {
            let mut next_level = Vec::new();
            let mut i = 0;
            while i < nodes.len() {
                if i + 1 < nodes.len() {
                    let left = nodes[i].clone();
                    let right = nodes[i + 1].clone();

                    let parent_bounds = left.bounds.union(&right.bounds);
                    next_level.push(BVHNode::new_branch(parent_bounds, vec![left, right]));
                    i += 2;
                } else {
                    next_level.push(nodes[i].clone());
                    i += 1;
                }
            }
            nodes = next_level;
        }

        self.root = nodes.into_iter().next();
    }

    /// Return list of visible entities within frustum planes.
    pub fn get_visible_entities(&self, planes: &[[f32; 4]; 6]) -> Vec<u64> {
        let mut visible = Vec::new();
        if let Some(ref root_node) = self.root {
            root_node.cull_visible(planes, &mut visible);
        }
        visible
    }
}

impl Default for BVH {
    fn default() -> Self {
        Self::new()
    }
}
