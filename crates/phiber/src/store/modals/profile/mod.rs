//! Profile modal spec - profile actions indexed by enum.

pub mod def;
pub mod set;

pub use def::BProfileType;

impl BProfileType {
    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn from_idx(idx: usize) -> Option<Self> {
        set::ALL.get(idx).copied()
    }

    pub fn all() -> &'static [BProfileType] {
        set::ALL
    }

    pub fn label(self) -> &'static str {
        set::LABELS[self as usize]
    }

    pub fn icon(self) -> &'static str {
        set::ICONS[self as usize]
    }

    pub fn items(self) -> &'static BProfileSpec {
        &set::ALL_SPECS[self as usize]
    }

    pub fn with_set(self) -> &'static BProfileSpec {
        &set::ALL_SPECS[self as usize]
    }
}

/// Static spec for a single profile action.
#[derive(Clone, Copy, Debug)]
pub struct BProfileSpec {
    pub idx: BProfileType,
    pub items: &'static [BProfileSpec],
}

impl BProfileSpec {
    pub const fn new(idx: BProfileType) -> Self {
        Self { idx, items: &[] }
    }

    pub fn with_set(&self) -> &'static [BProfileSpec] {
        self.items
    }
}

impl crate::store::spec::BSpec for BProfileSpec {
    fn idx(&self) -> crate::store::spec::BSpecType {
        crate::store::spec::BSpecType::BModal(crate::store::modals::BModalType::Profile)
    }
    fn items(&self) -> Vec<&dyn crate::store::spec::BSpec> {
        self.items
            .iter()
            .map(|s| s as &dyn crate::store::spec::BSpec)
            .collect()
    }
}
