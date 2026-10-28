//! The Rigid Body World (`RigidBodyWorld`, Scene ▸ Rigid Body World): its
//! collections, solver settings, effector weights and point cache.

use super::string_at;
use crate::blend::file::types::*;

/// `RBW_FLAG_MUTED`: the world is paused.
const RBW_FLAG_MUTED: u32 = 1 << 0;
/// `RBW_FLAG_USE_SPLIT_IMPULSE`: Split Impulse.
const RBW_FLAG_USE_SPLIT_IMPULSE: u32 = 1 << 2;
/// `PTCACHE_BAKED`.
const PTCACHE_BAKED: u32 = 1 << 0;

/// `EffectorWeights`: how much of each force field kind, and of gravity,
/// a simulation feels (`weight[14]` in `ePFieldType` order).
#[derive(Debug, Clone, PartialEq)]
pub struct BEffectorWeights {
    /// The collection the effectors are taken from; empty for all.
    pub collection: String,
    /// `weight[0]`, "All".
    pub all: f32,
    /// `weight[1..14]`: Force, Vortex, Magnetic, Wind, Curve Guide,
    /// Texture, Harmonic, Charge, Lennard-Jones, Boid, Turbulence, Drag,
    /// Fluid Flow.
    pub weights: [f32; 13],
    pub gravity: f32,
}

impl Default for BEffectorWeights {
    fn default() -> Self {
        Self { collection: String::new(), all: 1.0, weights: [1.0; 13], gravity: 1.0 }
    }
}

/// `PointCache`: the frames a simulation is cached over, and whether it
/// has been baked.
#[derive(Debug, Clone, PartialEq)]
pub struct BPointCache {
    pub frame_start: i32,
    pub frame_end: i32,
    /// Cache Step.
    pub step: i32,
    pub baked: bool,
}

impl Default for BPointCache {
    fn default() -> Self {
        Self { frame_start: 1, frame_end: 250, step: 1, baked: false }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BRigidWorld {
    /// The collection whose objects are the bodies (`group`).
    pub collection: String,
    /// The collection whose objects carry the constraints.
    pub constraints: String,
    pub enabled: bool,
    pub substeps_per_frame: u32,
    pub solver_iterations: u32,
    pub split_impulse: bool,
    pub time_scale: f32,
    pub effector_weights: BEffectorWeights,
    pub cache: BPointCache,
}

impl Default for BRigidWorld {
    fn default() -> Self {
        Self {
            collection: String::new(),
            constraints: String::new(),
            enabled: true,
            substeps_per_frame: 10,
            solver_iterations: 10,
            split_impulse: false,
            time_scale: 1.0,
            effector_weights: Default::default(),
            cache: Default::default(),
        }
    }
}

impl BlendFile {
    /// `Scene.rigidbody_world`, or `None` when the scene has no world.
    pub(crate) fn rigid_world(&self, sc: &BlendBlock, dna: &DnaType) -> Option<BRigidWorld> {
        let rbw = self.block_at(self.ptr(sc, self.off("Scene", "*rigidbody_world", dna)))?;
        let d = BRigidWorld::default();
        let flag = self.int_field(rbw, "RigidBodyWorld", "flag", dna);
        let f = |n: &str, dflt: f32| self.float_field(rbw, "RigidBodyWorld", n, dna, dflt);
        let i = |n: &str, dflt: u32| self.uint_field(rbw, "RigidBodyWorld", n, dna).unwrap_or(dflt);
        Some(BRigidWorld {
            collection: self.collection_name_at(self.ptr(rbw, self.off("RigidBodyWorld", "*group", dna))),
            constraints: self.collection_name_at(self.ptr(rbw, self.off("RigidBodyWorld", "*constraints", dna))),
            enabled: flag & RBW_FLAG_MUTED == 0,
            // `substeps_per_frame` in Blender 2.8-4.x; `steps_per_second`
            // in 2.7 and again in the files 5.x writes.
            substeps_per_frame: self
                .uint_field(rbw, "RigidBodyWorld", "substeps_per_frame", dna)
                .or_else(|| self.uint_field(rbw, "RigidBodyWorld", "steps_per_second", dna))
                .unwrap_or(d.substeps_per_frame),
            solver_iterations: i("num_solver_iterations", d.solver_iterations),
            split_impulse: flag & RBW_FLAG_USE_SPLIT_IMPULSE != 0,
            time_scale: f("time_scale", d.time_scale),
            effector_weights: self.effector_weights(self.ptr(rbw, self.off("RigidBodyWorld", "*effector_weights", dna)), dna),
            cache: self.point_cache(self.ptr(rbw, self.off("RigidBodyWorld", "*pointcache", dna)), dna),
        })
    }

    /// `ID.name` of the collection at `addr`, less its prefix.
    pub(crate) fn collection_name_at(&self, addr: u64) -> String {
        self.block_at(addr).and_then(|b| self.extract_name_from_block(b, "name").ok()).unwrap_or_default()
    }

    /// The `EffectorWeights` at `addr`, or the defaults.
    pub(crate) fn effector_weights(&self, addr: u64, dna: &DnaType) -> BEffectorWeights {
        let Some(ew) = self.block_at(addr) else { return BEffectorWeights::default() };
        let ps = self.header.pointer_size as usize;
        let weights = Self::struct_field_offset_exact("EffectorWeights", "weight[14]", dna, ps)
            .filter(|o| o + 14 * 4 <= ew.data.len())
            .map(|o| std::array::from_fn::<f32, 14, _>(|k| self.read_f32(&ew.data[o + 4 * k..o + 4 * k + 4])))
            .unwrap_or([1.0; 14]);
        BEffectorWeights {
            collection: self.collection_name_at(self.ptr(ew, self.off("EffectorWeights", "*group", dna))),
            all: weights[0],
            weights: std::array::from_fn(|k| weights[k + 1]),
            gravity: self.float_field(ew, "EffectorWeights", "global_gravity", dna, 1.0),
        }
    }

    /// The `PointCache` at `addr`, or the defaults.
    pub(crate) fn point_cache(&self, addr: u64, dna: &DnaType) -> BPointCache {
        let Some(pc) = self.block_at(addr) else { return BPointCache::default() };
        let d = BPointCache::default();
        let i = |n: &str, dflt: i32| self.uint_field(pc, "PointCache", n, dna).map(|v| v as i32).unwrap_or(dflt);
        let _ = string_at(&pc.data, self.off("PointCache", "name[64]", dna), 64);
        BPointCache {
            frame_start: i("startframe", d.frame_start),
            frame_end: i("endframe", d.frame_end),
            step: i("step", d.step).max(1),
            baked: self.int_field(pc, "PointCache", "flag", dna) & PTCACHE_BAKED != 0,
        }
    }
}
