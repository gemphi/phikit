//! `Object.pd`, a `PartDeflect`: one struct that is both the Collision
//! modifier's settings (Physics ▸ Collision, its `pdef_*` members) and a
//! force field (Physics ▸ Force Field, its `f_*` members).

use crate::blend::file::types::*;

/// `PDEFLE_KILL_PART`: Kill Particles.
const PDEFLE_KILL_PART: u32 = 1 << 4;
/// `PFIELD_CLOTH_USE_CULLING`: Single Sided.
const PFIELD_CLOTH_USE_CULLING: u32 = 1 << 19;
/// `PFIELD_CLOTH_USE_NORMAL`: Override Normals.
const PFIELD_CLOTH_USE_NORMAL: u32 = 1 << 20;
/// `PFIELD_USEMAX`, `PFIELD_USEMIN`: the distance limits are on.
const PFIELD_USEMAX: u32 = 1 << 0;
const PFIELD_USEMIN: u32 = 1 << 8;

#[derive(Debug, Clone, PartialEq)]
pub struct BCollider {
    pub object: String,
    /// Particle ▸ Permeability, Stickiness, Kill Particles.
    pub permeability: f32,
    pub stickiness: f32,
    pub kill_particles: bool,
    /// Particle ▸ Damping and its Randomize, Friction and its Randomize.
    pub damping_factor: f32,
    pub damping_random: f32,
    pub friction_factor: f32,
    pub friction_random: f32,
    /// Softbody & Cloth ▸ Damping (`pdef_sbdamp`), Thickness Outer
    /// (`pdef_sboft`) and Inner (`pdef_sbift`), Friction (`pdef_cfrict`).
    pub damping: f32,
    pub thickness_outer: f32,
    pub thickness_inner: f32,
    pub cloth_friction: f32,
    /// Single Sided, Override Normals.
    pub single_sided: bool,
    pub override_normals: bool,
}

impl Default for BCollider {
    fn default() -> Self {
        Self {
            object: String::new(),
            permeability: 0.0,
            stickiness: 0.0,
            kill_particles: false,
            damping_factor: 0.0,
            damping_random: 0.0,
            friction_factor: 0.0,
            friction_random: 0.0,
            damping: 0.1,
            thickness_outer: 0.02,
            thickness_inner: 0.2,
            cloth_friction: 5.0,
            single_sided: true,
            override_normals: false,
        }
    }
}

/// `ePFieldType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BFieldKind {
    #[default]
    None,
    Force,
    Vortex,
    Magnetic,
    Wind,
    CurveGuide,
    Texture,
    Harmonic,
    Charge,
    LennardJones,
    Boid,
    Turbulence,
    Drag,
    FluidFlow,
}

impl BFieldKind {
    fn of(v: u32) -> Self {
        match v {
            1 => Self::Force,
            2 => Self::Vortex,
            3 => Self::Magnetic,
            4 => Self::Wind,
            5 => Self::CurveGuide,
            6 => Self::Texture,
            7 => Self::Harmonic,
            8 => Self::Charge,
            9 => Self::LennardJones,
            10 => Self::Boid,
            11 => Self::Turbulence,
            12 => Self::Drag,
            13 => Self::FluidFlow,
            _ => Self::None,
        }
    }

    pub fn blender(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Force => "FORCE",
            Self::Vortex => "VORTEX",
            Self::Magnetic => "MAGNET",
            Self::Wind => "WIND",
            Self::CurveGuide => "GUIDE",
            Self::Texture => "TEXTURE",
            Self::Harmonic => "HARMONIC",
            Self::Charge => "CHARGE",
            Self::LennardJones => "LENNARDJ",
            Self::Boid => "BOID",
            Self::Turbulence => "TURBULENCE",
            Self::Drag => "DRAG",
            Self::FluidFlow => "FLUID_FLOW",
        }
    }
}

/// `ePFieldShape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BFieldShape {
    #[default]
    Point,
    Plane,
    Surface,
    Points,
    Line,
}

impl BFieldShape {
    fn of(v: u32) -> Self {
        match v {
            1 => Self::Plane,
            2 => Self::Surface,
            3 => Self::Points,
            4 => Self::Line,
            _ => Self::Point,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BForceField {
    pub object: String,
    pub kind: BFieldKind,
    pub shape: BFieldShape,
    /// Strength (`f_strength`), Flow (`f_flow`), Noise (`f_noise`), Seed.
    pub strength: f32,
    pub flow: f32,
    pub noise: f32,
    pub seed: u32,
    /// Falloff ▸ Power (`f_power`), and the distance limits when on.
    pub power: f32,
    pub max_distance: Option<f32>,
    pub min_distance: Option<f32>,
}

impl BlendFile {
    pub(crate) fn collider(&self, pd: &BlendBlock, object: &str, dna: &DnaType) -> BCollider {
        let d = BCollider::default();
        let flag = self.int_field(pd, "PartDeflect", "flag", dna);
        let f = |n: &str, dflt: f32| self.float_field(pd, "PartDeflect", n, dna, dflt);
        BCollider {
            object: object.to_string(),
            permeability: f("pdef_perm", d.permeability),
            stickiness: f("pdef_stickness", d.stickiness),
            kill_particles: flag & PDEFLE_KILL_PART != 0,
            damping_factor: f("pdef_damp", d.damping_factor),
            damping_random: f("pdef_rdamp", d.damping_random),
            friction_factor: f("pdef_frict", d.friction_factor),
            friction_random: f("pdef_rfrict", d.friction_random),
            damping: f("pdef_sbdamp", d.damping),
            thickness_outer: f("pdef_sboft", d.thickness_outer),
            thickness_inner: f("pdef_sbift", d.thickness_inner),
            cloth_friction: f("pdef_cfrict", d.cloth_friction),
            single_sided: flag & PFIELD_CLOTH_USE_CULLING != 0,
            override_normals: flag & PFIELD_CLOTH_USE_NORMAL != 0,
        }
    }

    pub(crate) fn force_field(&self, pd: &BlendBlock, object: &str, dna: &DnaType) -> BForceField {
        let flag = self.int_field(pd, "PartDeflect", "flag", dna);
        let f = |n: &str, dflt: f32| self.float_field(pd, "PartDeflect", n, dna, dflt);
        BForceField {
            object: object.to_string(),
            kind: BFieldKind::of(self.int_field(pd, "PartDeflect", "forcefield", dna)),
            shape: BFieldShape::of(self.int_field(pd, "PartDeflect", "shape", dna)),
            strength: f("f_strength", 0.0),
            flow: f("f_flow", 0.0),
            noise: f("f_noise", 0.0),
            seed: self.uint_field(pd, "PartDeflect", "seed", dna).unwrap_or(0),
            power: f("f_power", 0.0),
            max_distance: (flag & PFIELD_USEMAX != 0).then(|| f("maxdist", 0.0)),
            min_distance: (flag & PFIELD_USEMIN != 0).then(|| f("mindist", 0.0)),
        }
    }
}
