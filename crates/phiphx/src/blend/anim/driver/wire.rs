//! Driven values written into the scene — the step that was missing.
//!
//! Drivers on things that are **not** objects were read, typed, clamped and
//! measured against Blender over batches 172–174: a material's own
//! properties and its shader sockets (`shade/driver.rs`), a lamp's and the
//! world's (`object/light_driver.rs`, `shade/world_driver.rs`), a modifier's
//! settings (`modifier/driver.rs`). Each of those modules ends in an `apply`
//! that writes this frame's values over its target, each verified to six
//! decimals against Blender's own evaluated datablocks.
//!
//! **Nothing called any of them.** The sets were read into
//! `BlendScene::props` and left there, so a file whose material roughness is
//! driven to 0.9 opened showing the 0.2 saved in the file. The values were
//! right and invisible. [`BDriverState::drive`] is the call, and
//! `scene/read.rs` makes it once per open.
//!
//! ## Why this is where it happens
//!
//! A non-object driver has to read the scene **after** the object drivers
//! have run, because a socket driven off a custom property that is itself
//! driven only resolves in that order — which is the order Blender's
//! depsgraph gives, and what `BDriverState` was built to preserve
//! (`state.rs`). So the state comes from
//! [`BlendMotionList::driver_state_at`][super::super::motion], after the
//! actions, the NLA and the object pass, and every non-object driver reads
//! that one state.
//!
//! ## Matching a set to its target
//!
//! By the datablock's **saved address**, never by name. A material is the one
//! kind where a name would do (`BlendMaterial` carries its own), and even
//! there the address would be safer; for the other two a name is actively
//! wrong. A light object called `Lamp` may use light data called
//! `Light.001`, so an object's name says nothing about which set applies;
//! and `world_drivers` returns a set per world datablock in the file while
//! the scene uses one of them, so "apply the one set" is right until a file
//! has two worlds and then silently wrong.
//!
//! So `BlendLightData` and `BlendWorld` each carry the address they were read
//! from, and each driver set carries the address it was read off. A set whose
//! address matches nothing in the scene is left alone — it belongs to a
//! datablock this scene does not use.
//!
//! ## What is still not wired
//!
//! A **modifier**'s values, and the reason is ordering rather than identity:
//! `scene/read.rs` evaluates geometry (0.90) *before* it reads the animation
//! (0.94+), so at the moment the modifier stack is applied the driven
//! settings do not exist yet. Blender's depsgraph runs animation first and
//! geometry after, precisely because geometry depends on driven settings.
//! Fixing it the Blender way means moving the modifier evaluation after the
//! motion read — not threading values backwards.

use super::state::BDriverState;
use crate::blend::object::data::BlendObjectData;
use crate::blend::scene::types::BlendScene;

impl BDriverState {
    /// Write this frame's driven values over everything in `scene` that is
    /// not an object: its materials, its lamps and its world.
    ///
    /// Returns how many targets were written, so a test can assert that
    /// "wired" has not quietly become "wired and matching nothing".
    pub(crate) fn drive(&self, scene: &mut BlendScene) -> usize {
        // The sets are a handful per file, and cloning them frees the borrow
        // on `props` so the three targets below can be reached mutably
        // without splitting this into one call per kind.
        let materials = scene.props.material_drivers.clone();
        let lights = scene.props.light_drivers.clone();
        let worlds = scene.props.world_drivers.clone();
        if materials.iter().all(|set| set.is_empty())
            && lights.iter().all(|set| set.is_empty())
            && worlds.iter().all(|set| set.is_empty())
        {
            return 0;
        }
        let at = self.scene();
        let mut driven = 0;

        for object in scene.objects.iter_mut() {
            for slot in object.materials.iter_mut() {
                let Some(set) = materials.iter().find(|set| set.material == slot.name && !set.is_empty()) else {
                    continue;
                };
                set.apply(slot, &at, self.frame);
                driven += 1;
            }
            // A lamp's set belongs to the `Light` datablock, which several
            // light objects may share — each copy is driven to the same
            // value, which is what one driven datablock means.
            if let BlendObjectData::Light(light) = &mut object.data {
                if let Some(set) = lights.iter().find(|set| set.address == light.address && !set.is_empty()) {
                    set.apply(light, &at, self.frame);
                    driven += 1;
                }
            }
        }

        // The object's own `base_color` / `roughness` / `metallic` are slot
        // zero's, copied when the object was read — the viewport draws from
        // them, so a driven first slot has to move them too, or the values
        // are right in the slot and stale on screen.
        for object in scene.objects.iter_mut() {
            if let Some(first) = object.materials.first() {
                object.base_color = first.base_color;
                object.roughness = first.roughness;
                object.metallic = first.metallic;
            }
        }

        let world = scene.props.world.address;
        if let Some(set) = worlds.iter().find(|set| set.address == world && !set.is_empty()) {
            set.apply(&mut scene.props.world, &at, self.frame);
            driven += 1;
        }

        driven
    }
}
