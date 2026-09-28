//! `BPropSpec` — one RNA property. Blender ref: `source/blender/makesrna/intern`.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPropSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: BPropKind,
    /// The property's RNA default, where Blender's differs from the kind's
    /// factory value — `cycles.samples` is 4096, not 0. `None` keeps the
    /// kind's own (`ctrl_for` in `prop_view.rs`).
    pub default: Option<BDefault>,
}

/// An RNA default, in the value category the property edits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BDefault {
    Bool(bool),
    Int(i64),
    Float(f32),
    Enum(&'static str),
}

impl BPropSpec {
    pub const fn on(mut self) -> Self {
        self.default = Some(BDefault::Bool(true));
        self
    }
    pub const fn at(mut self, n: i64) -> Self {
        self.default = Some(BDefault::Int(n));
        self
    }
    pub const fn about(mut self, v: f32) -> Self {
        self.default = Some(BDefault::Float(v));
        self
    }
    pub const fn picked(mut self, key: &'static str) -> Self {
        self.default = Some(BDefault::Enum(key));
        self
    }
}

impl BSpec for BPropSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Panel)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BPropKind {
    Bool,
    Int { min: i64, max: i64 },
    Float { unit: BUnit },
    Vec3 { unit: BUnit },
    Vec4 { unit: BUnit },
    Enum(&'static [BEnumItem]),
    Str,
    Color { alpha: bool },
    Pointer { target: &'static str },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BUnit {
    None,
    Length,
    Angle,
    Time,
    Mass,
    Velocity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BEnumItem {
    pub key: &'static str,
    pub label: &'static str,
}

pub const fn b(k: &'static str, l: &'static str) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Bool,
        default: None,
    }
}
pub const fn i(k: &'static str, l: &'static str, min: i64, max: i64) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Int { min, max },
        default: None,
    }
}
pub const fn f(k: &'static str, l: &'static str, u: BUnit) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Float { unit: u },
        default: None,
    }
}
pub const fn v3(k: &'static str, l: &'static str, u: BUnit) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Vec3 { unit: u },
        default: None,
    }
}
pub const fn e(k: &'static str, l: &'static str, it: &'static [BEnumItem]) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Enum(it),
        default: None,
    }
}
pub const fn s(k: &'static str, l: &'static str) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Str,
        default: None,
    }
}
pub const fn p(k: &'static str, l: &'static str, t: &'static str) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Pointer { target: t },
        default: None,
    }
}
pub const fn col(k: &'static str, l: &'static str, a: bool) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Color { alpha: a },
        default: None,
    }
}
pub const fn en(k: &'static str, l: &'static str) -> BEnumItem {
    BEnumItem { key: k, label: l }
}
