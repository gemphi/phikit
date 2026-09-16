//! Help modal spec - help links indexed by enum.

pub mod def;
pub mod set;

pub use def::BHelpType;

impl BHelpType {
    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn from_idx(idx: usize) -> Option<Self> {
        set::ALL.get(idx).copied()
    }

    pub fn all() -> &'static [BHelpType] {
        set::ALL
    }

    pub fn label(self) -> &'static str {
        set::LABELS[self as usize]
    }

    pub fn icon(self) -> &'static str {
        set::ICONS[self as usize]
    }

    pub fn doc(self) -> &'static str {
        set::DOCS[self as usize]
    }

    pub fn items(self) -> &'static BHelpSpec {
        &set::ALL_SPECS[self as usize]
    }

    pub fn with_set(self) -> &'static BHelpSpec {
        &set::ALL_SPECS[self as usize]
    }
}

/// Static spec for a single help link.
#[derive(Clone, Copy, Debug)]
pub struct BHelpSpec {
    pub idx: BHelpType,
    pub items: &'static [BHelpSpec],
}

impl BHelpSpec {
    pub const fn new(idx: BHelpType) -> Self {
        Self { idx, items: &[] }
    }

    pub fn with_set(&self) -> &'static [BHelpSpec] {
        self.items
    }
}

impl crate::store::spec::BSpec for BHelpSpec {
    fn idx(&self) -> crate::store::spec::BSpecType {
        crate::store::spec::BSpecType::BModal(crate::store::modals::BModalType::Help)
    }
    fn items(&self) -> Vec<&dyn crate::store::spec::BSpec> {
        self.items
            .iter()
            .map(|s| s as &dyn crate::store::spec::BSpec)
            .collect()
    }
}
