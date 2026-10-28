//! `Object.soft`, a `SoftBody` (Physics ▸ Soft Body). The on/off switches
//! live in `Object.softflag`; the numbers in the `SoftBody` block.

use super::string_at;
use crate::blend::file::types::*;

/// `Object.softflag` bits.
const OB_SB_GOAL: u32 = 1 << 1;
const OB_SB_EDGES: u32 = 1 << 2;
const OB_SB_QUADS: u32 = 1 << 3;
const OB_SB_SELF: u32 = 1 << 9;
const OB_SB_FACECOLL: u32 = 1 << 10;
const OB_SB_EDGECOLL: u32 = 1 << 11;
/// `SoftBody.solverflags`: `SBSO_OLDERR` is Auto-Step (`SBSO_MONITOR` is
/// bit 0, Print Performance Diagnostics).
const SBSO_OLDERR: u32 = 1 << 1;

/// `sbc_mode`: how a vertex's collision ball is sized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BSoftCollision {
    Manual,
    #[default]
    Average,
    Minimal,
    Maximal,
    AverageMinMax,
}

impl BSoftCollision {
    fn of(v: u32) -> Self {
        match v {
            0 => Self::Manual,
            2 => Self::Minimal,
            3 => Self::Maximal,
            4 => Self::AverageMinMax,
            _ => Self::Average,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BSoftBody {
    pub object: String,
    /// Object ▸ Friction (`mediafrict`), Mass (`nodemass`), its vertex
    /// group, Speed (`physics_speed`).
    pub friction: f32,
    pub mass: f32,
    pub mass_group: String,
    pub speed: f32,
    /// Goal on, Default / Minimum / Maximum, Stiffness (`goalspring`),
    /// Damping (`goalfrict`), its vertex group.
    pub goal: bool,
    pub goal_default: f32,
    pub goal_min: f32,
    pub goal_max: f32,
    pub goal_spring: f32,
    pub goal_friction: f32,
    pub goal_group: String,
    /// Edges on; Pull (`inspring`), Push (`inpush`), Damp (`infrict`),
    /// Plasticity, Bending (`secondspring`), Length (`springpreload`),
    /// Shear (`shearstiff`), Stiff Quads, their vertex group.
    pub edges: bool,
    pub pull: f32,
    pub push: f32,
    pub damping: f32,
    pub plastic: u32,
    pub bending: f32,
    pub spring_length: u32,
    pub shear: f32,
    pub stiff_quads: bool,
    pub spring_group: String,
    /// Self Collision on, its ball sizing, Ball Size (`colball`), Stiffness
    /// (`ballstiff`), Dampening (`balldamp`).
    pub self_collision: bool,
    pub collision: BSoftCollision,
    pub ball_size: f32,
    pub ball_stiffness: f32,
    pub ball_damping: f32,
    /// Collision with faces and edges of colliders.
    pub face_collision: bool,
    pub edge_collision: bool,
    /// Solver ▸ Step Size Min / Max (`minloops`, `maxloops`), Auto-Step,
    /// Error Limit (`rklimit`), Choke, Fuzzy (`fuzzyness`).
    pub step_min: u32,
    pub step_max: u32,
    pub auto_step: bool,
    pub error_limit: f32,
    pub choke: u32,
    pub fuzzy: u32,
    pub cache: super::BPointCache,
}

impl Default for BSoftBody {
    fn default() -> Self {
        Self {
            object: String::new(),
            friction: 0.5,
            mass: 1.0,
            mass_group: String::new(),
            speed: 1.0,
            goal: true,
            goal_default: 0.7,
            goal_min: 0.0,
            goal_max: 1.0,
            goal_spring: 0.5,
            goal_friction: 0.0,
            goal_group: String::new(),
            edges: true,
            pull: 0.5,
            push: 0.5,
            damping: 0.5,
            plastic: 0,
            bending: 0.1,
            spring_length: 0,
            shear: 1.0,
            stiff_quads: false,
            spring_group: String::new(),
            self_collision: false,
            collision: Default::default(),
            ball_size: 0.49,
            ball_stiffness: 1.0,
            ball_damping: 0.5,
            // `sbNew` sets Goal and Edges alone; both collision kinds are off.
            face_collision: false,
            edge_collision: false,
            step_min: 10,
            step_max: 300,
            auto_step: true,
            error_limit: 0.1,
            choke: 3,
            fuzzy: 1,
            cache: Default::default(),
        }
    }
}

impl BlendFile {
    /// The name of the object's vertex group number `index` (1-based, 0 for
    /// none), as `rna_object_vgroup_name_index_get` resolves it.
    fn vertex_group_at(&self, ob: &BlendBlock, index: usize, dna: &DnaType) -> String {
        let Some(me) = self.mesh_block_of(ob, dna).filter(|_| index > 0) else { return String::new() };
        self.vertex_group_names(ob, me, dna).get(index - 1).cloned().unwrap_or_default()
    }

    pub(crate) fn soft_body(&self, sb: &BlendBlock, ob: &BlendBlock, object: &str, dna: &DnaType) -> BSoftBody {
        let d = BSoftBody::default();
        let softflag = self.int_field(ob, "Object", "softflag", dna);
        let f = |n: &str, dflt: f32| self.float_field(sb, "SoftBody", n, dna, dflt);
        let i = |n: &str, dflt: u32| self.uint_field(sb, "SoftBody", n, dna).unwrap_or(dflt);
        let name = |n: &str| string_at(&sb.data, self.off("SoftBody", n, dna), 64);
        BSoftBody {
            object: object.to_string(),
            friction: f("mediafrict", d.friction),
            mass: f("nodemass", d.mass),
            mass_group: name("namedVG_Mass[64]"),
            speed: f("physics_speed", d.speed),
            goal: softflag & OB_SB_GOAL != 0,
            goal_default: f("defgoal", d.goal_default),
            goal_min: f("mingoal", d.goal_min),
            goal_max: f("maxgoal", d.goal_max),
            goal_spring: f("goalspring", d.goal_spring),
            goal_friction: f("goalfrict", d.goal_friction),
            // RNA `vertex_group_goal` is `SoftBody.vertgroup`, the group's
            // 1-based index (`rna_SoftBodySettings_goal_vgroup_set`); the
            // `namedVG_Softgoal` name beside it is never written.
            goal_group: self.vertex_group_at(ob, i("vertgroup", 0) as usize, dna),
            edges: softflag & OB_SB_EDGES != 0,
            pull: f("inspring", d.pull),
            push: f("inpush", d.push),
            damping: f("infrict", d.damping),
            plastic: i("plastic", d.plastic),
            bending: f("secondspring", d.bending),
            spring_length: i("springpreload", d.spring_length),
            shear: f("shearstiff", d.shear),
            stiff_quads: softflag & OB_SB_QUADS != 0,
            spring_group: name("namedVG_Spring_K[64]"),
            self_collision: softflag & OB_SB_SELF != 0,
            collision: BSoftCollision::of(i("sbc_mode", 1)),
            ball_size: f("colball", d.ball_size),
            ball_stiffness: f("ballstiff", d.ball_stiffness),
            ball_damping: f("balldamp", d.ball_damping),
            face_collision: softflag & OB_SB_FACECOLL != 0,
            edge_collision: softflag & OB_SB_EDGECOLL != 0,
            step_min: i("minloops", d.step_min),
            step_max: i("maxloops", d.step_max),
            auto_step: i("solverflags", SBSO_OLDERR) & SBSO_OLDERR != 0,
            error_limit: f("rklimit", d.error_limit),
            choke: i("choke", d.choke),
            fuzzy: i("fuzzyness", d.fuzzy),
            cache: self.point_cache(self.ptr(sb, self.off("SoftBody", "*pointcache", dna)), dna),
        }
    }
}
