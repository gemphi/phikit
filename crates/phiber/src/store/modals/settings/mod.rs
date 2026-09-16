//! Settings modal spec - settings categories indexed by enum.

pub mod def;
pub mod set;

pub use def::BSettingsType;

impl BSettingsType {
    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn from_idx(idx: usize) -> Option<Self> {
        set::ALL.get(idx).copied()
    }

    pub fn all() -> &'static [BSettingsType] {
        set::ALL
    }

    pub fn label(self) -> &'static str {
        set::LABELS[self as usize]
    }

    pub fn rows(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Interface => set::INTERFACE_ROWS,
            Self::Editing => set::EDITING_ROWS,
            Self::Navigation => set::NAVIGATION_ROWS,
            Self::AddOns => set::ADDONS_ROWS,
            Self::Themes => set::THEMES_ROWS,
            Self::System => set::SYSTEM_ROWS,
        }
    }

    pub fn items(self) -> &'static BSettingsSpec {
        &set::ALL_SPECS[self as usize]
    }

    pub fn with_set(self) -> &'static BSettingsSpec {
        &set::ALL_SPECS[self as usize]
    }
}

/// Static spec for a single settings category.
#[derive(Clone, Copy, Debug)]
pub struct BSettingsSpec {
    pub idx: BSettingsType,
    pub items: &'static [BSettingsSpec],
}

impl BSettingsSpec {
    pub const fn new(idx: BSettingsType) -> Self {
        Self { idx, items: &[] }
    }

    pub fn with_set(&self) -> &'static [BSettingsSpec] {
        self.items
    }
}

impl crate::store::spec::BSpec for BSettingsSpec {
    fn idx(&self) -> crate::store::spec::BSpecType {
        crate::store::spec::BSpecType::BModal(crate::store::modals::BModalType::Settings)
    }
    fn items(&self) -> Vec<&dyn crate::store::spec::BSpec> {
        self.items
            .iter()
            .map(|s| s as &dyn crate::store::spec::BSpec)
            .collect()
    }
}
