//! Switcher part spec — editor picker dropdown and modes.
//! Follows standard phiber store parts specification.

pub mod def;
pub mod set;

use crate::store::hosts::BHostType;
use crate::store::parts::{BPart, BPartSpec, BPartType};

/// A switcher mode capability item for a host.
#[derive(Clone, Copy, Debug)]
pub struct ModeEntry {
    pub label: &'static str,
    pub icon: &'static str,
}

/// A category group of hosts in the switcher.
#[derive(Clone, Copy, Debug)]
pub struct SwitcherGroup {
    pub name: &'static str,
    pub hosts: &'static [BHostType],
}

/// Spec for a Switcher part.
#[derive(Clone, Copy, Debug)]
pub struct SwitcherPartSpec {
    pub idx: BPartType,
    pub items: &'static [BPart],
}

impl SwitcherPartSpec {
    pub const fn new() -> Self {
        Self {
            idx: BPartType::Switcher,
            items: &[],
        }
    }
}

impl BPartSpec for SwitcherPartSpec {
    fn idx(&self) -> BPartType {
        self.idx
    }
    fn items(&self) -> &'static [BPart] {
        self.items
    }
}
