pub mod prelude {
    pub use glam::Vec3;
    pub use rust_reality_kit::prelude::*;
}

pub use prelude::*;

use std::sync::atomic::{AtomicU64, Ordering};

pub struct Scene {
    pub world: World,
    pub root_entities: Vec<Entity>,
    pub meta: std::collections::HashMap<u64, EntityMeta>,
    /// Version counter for dirty tracking - incremented on any scene modification
    version: AtomicU64,
}

#[derive(Clone)]
pub struct EntityMeta {
    pub name: String,
    pub children: Vec<Entity>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            root_entities: Vec::new(),
            meta: std::collections::HashMap::new(),
            version: AtomicU64::new(1), // Start at 1 so 0 means "never synced"
        }
    }

    /// Get current scene version for dirty tracking
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    /// Mark scene as modified (increments version)
    pub fn mark_dirty(&self) {
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_entity_name(&self, entity: Entity) -> &str {
        self.meta
            .get(&entity.id())
            .map(|m| m.name.as_str())
            .unwrap_or("Untitled Entity")
    }

    pub fn ray_cast(&self, origin: Vec3, direction: Vec3) -> Option<Entity> {
        let mut closest_entity_id = None;
        let mut min_t = f32::MAX;

        // Iterate all entities with a Transform
        let mut query = self.world.query::<&Transform>();
        for (entity, transform) in query.iter() {
            // For now, treat everything as a 2x2x2 box at the transform position
            let center = transform.translation;
            let half_extents = transform.scale;

            // Slab-based AABB intersection
            let min = center - half_extents;
            let max = center + half_extents;

            let mut t_min = (min.x - origin.x) / direction.x;
            let mut t_max = (max.x - origin.x) / direction.x;

            if t_min > t_max {
                std::mem::swap(&mut t_min, &mut t_max);
            }

            let mut ty_min = (min.y - origin.y) / direction.y;
            let mut ty_max = (max.y - origin.y) / direction.y;

            if ty_min > ty_max {
                std::mem::swap(&mut ty_min, &mut ty_max);
            }

            if (t_min > ty_max) || (ty_min > t_max) {
                continue;
            }

            if ty_min > t_min {
                t_min = ty_min;
            }
            if ty_max < t_max {
                t_max = ty_max;
            }

            let mut tz_min = (min.z - origin.z) / direction.z;
            let mut tz_max = (max.z - origin.z) / direction.z;

            if tz_min > tz_max {
                std::mem::swap(&mut tz_min, &mut tz_max);
            }

            if (t_min > tz_max) || (tz_min > t_max) {
                continue;
            }

            if tz_min > t_min {
                t_min = tz_min;
            }

            // Intersection! Check if it's the closest one and in front of camera
            if t_min > 0.0 && t_min < min_t {
                min_t = t_min;
                closest_entity_id = Some(entity.id());
            }
        }

        closest_entity_id.map(Entity::from_id)
    }
}

pub struct NSViewComponent {
    pub view_id: u64,
}
