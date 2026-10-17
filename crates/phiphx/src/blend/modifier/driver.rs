//! Drivers on a modifier's settings — `modifiers["Name"].property` on the
//! **object's** `AnimData`, not on the modifier and not on the mesh.
//!
//! Two things make this unlike the material, light and world passes.
//!
//! **The modifier's type is not in the path.** It has to be resolved from
//! the object's own stack by name, because the same property name means
//! different things on different modifiers: `strength` on Displace and on
//! Lattice, `merge_threshold` on Weld and on Mirror, `offset` on Solidify
//! and on Array. So the setting table is keyed by `(type, property)`, and
//! a driver on a modifier the stack has no entry for is read and skipped.
//!
//! **Some settings are integers.** A driver's value is always a float, and
//! an integer property takes it by **truncation toward zero** — a C `(int)`
//! cast, which is neither a round nor a floor. Measured: 0.7 → 0, 2.5 → 2,
//! 11.9 → 11, −0.7 → **0**, −1.5 → **−1**, −2.5 → **−2**. Floor would give
//! −1, −2 and −3 for those last three; rounding would give 1 and 3 for the
//! first two. The hard range clamps afterwards, and the minima are not all
//! zero: `Subsurf.quality` starts at 1 and `Triangulate.min_vertices` at 4.
//!
//! **What this does not do yet.** The values are read, typed, clamped and
//! measured; they are *not* yet written into the modifier stack, so an
//! evaluated mesh does not reflect them. That plumbing runs through
//! `modifier/mod.rs`'s `modifier_jobs` / `evaluate_modifiers`, which also
//! only evaluate a mesh once at open rather than per frame — a separate
//! change, and one in a file another agent is actively editing. The
//! semantics are settled here so that change is mechanical.

use crate::blend::anim::driver::{BlendDriver, BlendDriverScene};

/// How a setting takes a driver's value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BModKind {
    /// A float property, clamped to its RNA hard range.
    Float { range: (f32, f32) },
    /// An integer property: truncated toward zero, then clamped.
    Int { range: (i32, i32) },
}

/// A driven value, in the property's own type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BModValue {
    Float(f32),
    Int(i32),
}

impl BModValue {
    /// The value as a float, for a caller that does not care which it is.
    pub fn as_f32(self) -> f32 {
        match self {
            Self::Float(v) => v,
            Self::Int(v) => v as f32,
        }
    }
}

impl BModKind {
    /// `raw` as the write path leaves it.
    pub fn take(self, raw: f32) -> BModValue {
        match self {
            Self::Float { range: (lo, hi) } => BModValue::Float(raw.clamp(lo, hi)),
            // `(int)raw` truncates toward zero; a NaN casts to 0 in Rust as
            // it does in C, which is also what a broken driver should give.
            Self::Int { range: (lo, hi) } => BModValue::Int((raw as i32).clamp(lo, hi)),
        }
    }
}

/// Unbounded both ways — most modifier floats.
const FREE: BModKind = BModKind::Float { range: (f32::MIN, f32::MAX) };
/// A distance or threshold: zero or more.
const NON_NEGATIVE: BModKind = BModKind::Float { range: (0.0, f32::MAX) };

/// One drivable setting: the RNA property name as the path spells it, and
/// how it takes a value.
#[derive(Clone, Copy, Debug)]
pub struct BModSetting {
    pub prop: &'static str,
    pub kind: BModKind,
    /// How many components the property has (1 for a scalar).
    pub components: usize,
}

const fn scalar(prop: &'static str, kind: BModKind) -> BModSetting {
    BModSetting { prop, kind, components: 1 }
}
const fn vector(prop: &'static str, kind: BModKind) -> BModSetting {
    BModSetting { prop, kind, components: 3 }
}

/// The settings of each modifier type, by the type name `modifier::KINDS`
/// uses. Ranges are the RNA hard ranges, read off Blender itself.
const SETTINGS: &[(&str, &[BModSetting])] = &[
    (
        "SUBSURF",
        &[
            scalar("levels", BModKind::Int { range: (0, 11) }),
            scalar("render_levels", BModKind::Int { range: (0, 11) }),
            // Not zero: a subdivision needs at least one pass of quality.
            scalar("quality", BModKind::Int { range: (1, 10) }),
        ],
    ),
    (
        "TRIANGULATE",
        // Not zero either: a face needs four corners before it can be cut.
        &[scalar("min_vertices", BModKind::Int { range: (4, i32::MAX) })],
    ),
    ("DISPLACE", &[scalar("strength", FREE), scalar("mid_level", FREE)]),
    (
        "SOLIDIFY",
        &[
            scalar("thickness", FREE),
            scalar("offset", FREE),
            scalar("material_offset", BModKind::Int { range: (-32768, 32767) }),
            scalar("material_offset_rim", BModKind::Int { range: (-32768, 32767) }),
        ],
    ),
    ("WELD", &[scalar("merge_threshold", NON_NEGATIVE)]),
    ("MIRROR", &[scalar("merge_threshold", NON_NEGATIVE), scalar("bisect_threshold", NON_NEGATIVE)]),
    ("LATTICE", &[scalar("strength", FREE)]),
    (
        "ARRAY",
        &[
            scalar("count", BModKind::Int { range: (1, i32::MAX) }),
            vector("constant_offset_displace", FREE),
            vector("relative_offset_displace", FREE),
        ],
    ),
];

/// The drivable settings of a modifier type, empty for one with none here.
pub fn settings_of(kind: &str) -> &'static [BModSetting] {
    SETTINGS.iter().find(|(name, _)| *name == kind).map_or(&[], |(_, list)| *list)
}

/// The setting `prop` of modifier type `kind`, when it is one this models
/// and `component` is within it.
pub fn setting(kind: &str, prop: &str, component: usize) -> Option<BModSetting> {
    settings_of(kind).iter().copied().find(|s| s.prop == prop && component < s.components)
}

/// What one driver on a modifier writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BModWrite {
    /// The modifier's name, as the path spells it.
    pub modifier: String,
    /// Its type, resolved from the object's stack — `SUBSURF`, `DISPLACE`…
    pub kind: String,
    pub prop: String,
    pub component: usize,
}

/// One driver on a modifier setting.
#[derive(Clone, Debug, PartialEq)]
pub struct BModDriver {
    pub write: BModWrite,
    pub setting: BModSetting,
    pub driver: BlendDriver,
}

impl PartialEq for BModSetting {
    fn eq(&self, other: &Self) -> bool {
        self.prop == other.prop && self.kind == other.kind && self.components == other.components
    }
}

/// Every modifier driver one object carries.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BModifierDrivers {
    pub object: String,
    /// The object's saved address. The name is for reporting; this is what
    /// the stack reader matches on, the way a lamp's and the world's sets
    /// are matched (`anim/driver/wire.rs`).
    pub address: u64,
    pub drivers: Vec<BModDriver>,
}

impl BModifierDrivers {
    pub fn is_empty(&self) -> bool {
        self.drivers.is_empty()
    }

    /// Every driver's value at `frame`, in the property's own type and
    /// clamped as Blender clamps it. A muted driver is left out.
    pub fn values(&self, scene: &BlendDriverScene, frame: f32) -> Vec<(BModWrite, BModValue)> {
        self.drivers
            .iter()
            .filter(|d| !d.driver.muted())
            .map(|d| (d.write.clone(), d.setting.kind.take(d.driver.value(scene, frame))))
            .collect()
    }
}

#[cfg(test)]
#[path = "driver_tests.rs"]
mod driver_tests;
