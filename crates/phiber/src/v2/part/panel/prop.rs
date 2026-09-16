//! `BPropSpec` — one RNA property. Blender ref: `source/blender/makesrna/intern`.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPropSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: BPropKind,
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
    }
}
pub const fn i(k: &'static str, l: &'static str, min: i64, max: i64) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Int { min, max },
    }
}
pub const fn f(k: &'static str, l: &'static str, u: BUnit) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Float { unit: u },
    }
}
pub const fn v3(k: &'static str, l: &'static str, u: BUnit) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Vec3 { unit: u },
    }
}
pub const fn e(k: &'static str, l: &'static str, it: &'static [BEnumItem]) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Enum(it),
    }
}
pub const fn s(k: &'static str, l: &'static str) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Str,
    }
}
pub const fn p(k: &'static str, l: &'static str, t: &'static str) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Pointer { target: t },
    }
}
pub const fn col(k: &'static str, l: &'static str, a: bool) -> BPropSpec {
    BPropSpec {
        key: k,
        label: l,
        kind: BPropKind::Color { alpha: a },
    }
}
pub const fn en(k: &'static str, l: &'static str) -> BEnumItem {
    BEnumItem { key: k, label: l }
}
