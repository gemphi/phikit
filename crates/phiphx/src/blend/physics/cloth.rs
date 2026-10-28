//! The Cloth modifier's settings (`ClothModifierData.sim_parms`, a
//! `ClothSimSettings`, and `coll_parms`, a `ClothCollSettings`). The RNA
//! names the Physics tab shows are given beside the DNA names they read.

use crate::blend::file::types::*;

/// `CLOTH_SIMSETTINGS_FLAG_PRESSURE`.
const SIM_FLAG_PRESSURE: u32 = 1 << 5;
/// `CLOTH_SIMSETTINGS_FLAG_PRESSURE_VOL`: a user-set target volume.
const SIM_FLAG_PRESSURE_VOL: u32 = 1 << 6;
/// `CLOTH_SIMSETTINGS_FLAG_INTERNAL_SPRINGS`.
const SIM_FLAG_INTERNAL_SPRINGS: u32 = 1 << 7;
/// `CLOTH_SIMSETTINGS_FLAG_DYNAMIC_BASEMESH`.
const SIM_FLAG_DYNAMIC_BASEMESH: u32 = 1 << 10;
/// `CLOTH_SIMSETTINGS_FLAG_SEW`.
const SIM_FLAG_SEW: u32 = 1 << 12;
/// `CLOTH_COLLSETTINGS_FLAG_ENABLED`.
const COLL_FLAG_ENABLED: u32 = 1 << 1;
/// `CLOTH_COLLSETTINGS_FLAG_SELF`.
const COLL_FLAG_SELF: u32 = 1 << 2;

/// Physics ▸ Cloth ▸ Collisions.
#[derive(Debug, Clone, PartialEq)]
pub struct BClothCollision {
    /// Object Collisions on (`use_collision`).
    pub enabled: bool,
    /// Quality (`loop_count`).
    pub quality: u32,
    /// Distance (`epsilon`).
    pub distance: f32,
    /// Impulse Clamping (`clamp`).
    pub impulse_clamp: f32,
    /// Friction (`friction`).
    pub friction: f32,
    /// The collection colliders are taken from; empty for every collider.
    pub collection: String,
    /// Self Collisions on.
    pub self_enabled: bool,
    /// `selfepsilon`.
    pub self_distance: f32,
    pub self_friction: f32,
    /// `self_clamp`.
    pub self_impulse_clamp: f32,
    /// The vertex group self collision leaves out (`vgroup_selfcol`).
    pub self_group: String,
}

impl Default for BClothCollision {
    fn default() -> Self {
        Self {
            enabled: true,
            quality: 2,
            distance: 0.015,
            impulse_clamp: 0.0,
            friction: 5.0,
            collection: String::new(),
            self_enabled: false,
            self_distance: 0.015,
            self_friction: 5.0,
            self_impulse_clamp: 0.0,
            self_group: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BCloth {
    pub object: String,
    /// The modifier's own name.
    pub modifier: String,
    /// Quality Steps (`stepsPerFrame`).
    pub quality: u32,
    /// Vertex Mass (`mass`).
    pub mass: f32,
    /// Air Viscosity (`Cvi`).
    pub air_damping: f32,
    /// Stiffness: Tension (`structural`), Compression, Shear, Bending.
    pub tension: f32,
    pub compression: f32,
    pub shear: f32,
    pub bending: f32,
    /// Damping: Tension (`tension_damp`), Compression, Shear, Bending.
    pub tension_damping: f32,
    pub compression_damping: f32,
    pub shear_damping: f32,
    pub bending_damping: f32,
    /// Bending Model: `true` for Angular (`bending_model == 1`), else Linear.
    pub angular_bending: bool,
    /// Shape ▸ Pin Group (`vgroup_mass`) and Stiffness (`goalspring`).
    pub pin_group: String,
    pub pin_stiffness: f32,
    /// Shape ▸ Sewing, Shrinking (min, max), Dynamic Mesh.
    pub sewing: bool,
    pub shrink: [f32; 2],
    pub dynamic_mesh: bool,
    /// Internal Springs on.
    pub internal_springs: bool,
    /// Pressure on, its force (`uniform_pressure_force`), the user volume
    /// when set (`target_volume`, else `None`), Pressure Scale and Fluid
    /// Density.
    pub pressure: bool,
    pub pressure_force: f32,
    pub target_volume: Option<f32>,
    pub pressure_factor: f32,
    pub fluid_density: f32,
    /// The vertex groups scaling the stiffnesses: Structural, Shear,
    /// Bending (`vgroup_struct`, `vgroup_shear`, `vgroup_bend`).
    pub stiffness_groups: [String; 3],
    pub collision: BClothCollision,
}

impl Default for BCloth {
    fn default() -> Self {
        Self {
            object: String::new(),
            modifier: String::new(),
            quality: 5,
            mass: 0.3,
            air_damping: 1.0,
            tension: 15.0,
            compression: 15.0,
            shear: 5.0,
            bending: 0.5,
            tension_damping: 5.0,
            compression_damping: 5.0,
            shear_damping: 5.0,
            bending_damping: 0.5,
            angular_bending: true,
            pin_group: String::new(),
            pin_stiffness: 1.0,
            sewing: false,
            shrink: [0.0, 0.0],
            dynamic_mesh: false,
            internal_springs: false,
            pressure: false,
            pressure_force: 0.0,
            target_volume: None,
            pressure_factor: 1.0,
            fluid_density: 0.0,
            stiffness_groups: Default::default(),
            collision: Default::default(),
        }
    }
}

impl BlendFile {
    /// The settings of the Cloth modifier `md` on `ob`.
    pub(crate) fn cloth(&self, md: &BlendBlock, ob: &BlendBlock, object: &str, dna: &DnaType) -> BCloth {
        let d = BCloth::default();
        let modifier = self.modifier_name(md, self.modifier_name_offset(dna)).unwrap_or_default();
        let Some(sim) = self.block_at(self.ptr(md, self.off("ClothModifierData", "*sim_parms", dna))) else {
            return BCloth { object: object.to_string(), modifier, ..d };
        };
        let flags = self.int_field(sim, "ClothSimSettings", "flags", dna);
        let f = |n: &str, dflt: f32| self.float_field(sim, "ClothSimSettings", n, dna, dflt);
        let group = |n: &str| self.vertex_group_name(ob, self.int_field(sim, "ClothSimSettings", n, dna), dna);
        BCloth {
            object: object.to_string(),
            modifier,
            quality: self.uint_field(sim, "ClothSimSettings", "stepsPerFrame", dna).unwrap_or(d.quality),
            mass: f("mass", d.mass),
            air_damping: f("Cvi", d.air_damping),
            // `tension` since 2.8; `structural` is the one field older
            // files have, and the one the RNA setter still writes beside it.
            tension: f("tension", f("structural", d.tension)),
            compression: f("compression", d.compression),
            shear: f("shear", d.shear),
            bending: f("bending", d.bending),
            tension_damping: f("tension_damp", d.tension_damping),
            compression_damping: f("compression_damp", d.compression_damping),
            shear_damping: f("shear_damp", d.shear_damping),
            bending_damping: f("bending_damping", d.bending_damping),
            angular_bending: self.int_field(sim, "ClothSimSettings", "bending_model", dna) == 1,
            pin_group: group("vgroup_mass"),
            pin_stiffness: f("goalspring", d.pin_stiffness),
            sewing: flags & SIM_FLAG_SEW != 0,
            shrink: [f("shrink_min", 0.0), f("shrink_max", 0.0)],
            dynamic_mesh: flags & SIM_FLAG_DYNAMIC_BASEMESH != 0,
            internal_springs: flags & SIM_FLAG_INTERNAL_SPRINGS != 0,
            pressure: flags & SIM_FLAG_PRESSURE != 0,
            pressure_force: f("uniform_pressure_force", d.pressure_force),
            target_volume: (flags & SIM_FLAG_PRESSURE_VOL != 0).then(|| f("target_volume", 0.0)),
            pressure_factor: f("pressure_factor", d.pressure_factor),
            fluid_density: f("fluid_density", d.fluid_density),
            stiffness_groups: [group("vgroup_struct"), group("vgroup_shear"), group("vgroup_bend")],
            collision: self.cloth_collision(md, ob, dna),
        }
    }

    fn cloth_collision(&self, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> BClothCollision {
        let d = BClothCollision::default();
        let Some(coll) = self.block_at(self.ptr(md, self.off("ClothModifierData", "*coll_parms", dna))) else { return d };
        let flags = self.int_field(coll, "ClothCollSettings", "flags", dna);
        let f = |n: &str, dflt: f32| self.float_field(coll, "ClothCollSettings", n, dna, dflt);
        BClothCollision {
            enabled: flags & COLL_FLAG_ENABLED != 0,
            quality: self.uint_field(coll, "ClothCollSettings", "loop_count", dna).unwrap_or(d.quality),
            distance: f("epsilon", d.distance),
            impulse_clamp: f("clamp", d.impulse_clamp),
            friction: f("friction", d.friction),
            collection: self.collection_name_at(self.ptr(coll, self.off("ClothCollSettings", "*group", dna))),
            self_enabled: flags & COLL_FLAG_SELF != 0,
            self_distance: f("selfepsilon", d.self_distance),
            self_friction: f("self_friction", d.self_friction),
            self_impulse_clamp: f("self_clamp", d.self_impulse_clamp),
            self_group: self.vertex_group_name(ob, self.int_field(coll, "ClothCollSettings", "vgroup_selfcol", dna), dna),
        }
    }
}
