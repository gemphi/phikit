//! Descriptor-backed UI-style preference declarations.
//!
//! Sections and rows are projections of `UI_STYLE_DESCRIPTORS`; this module
//! intentionally owns no preference keys, parsers, bounds, or defaults.

use crate::v2::state::BPath;
use crate::v2::theme::{
    BPreferenceMetadata, BStyleDescriptor, BStyleParseError, BStyleParser, BStyleValue,
    BUiStyleDefaults, UI_STYLE_DESCRIPTORS,
};

#[derive(Clone, Copy)]
pub struct BPreferenceRow(&'static BStyleDescriptor);

impl BPreferenceRow {
    pub const fn from_descriptor(descriptor: &'static BStyleDescriptor) -> Self {
        Self(descriptor)
    }

    pub const fn descriptor(self) -> &'static BStyleDescriptor {
        self.0
    }

    pub fn key(self) -> &'static BPath {
        self.0.key
    }

    pub fn target(self) -> &'static str {
        self.0.target
    }

    pub fn parser(self) -> BStyleParser {
        self.0.parser
    }

    pub fn metadata(self) -> &'static BPreferenceMetadata {
        &self.0.preference
    }

    pub fn default_value(self, defaults: &BUiStyleDefaults) -> BStyleValue {
        self.0.default_value(defaults)
    }

    pub fn parse_and_validate(self, raw: &str) -> Result<BStyleValue, BStyleParseError> {
        self.0.parse_and_validate(raw)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BPreferenceSection {
    title: &'static str,
}

impl BPreferenceSection {
    pub const fn title(self) -> &'static str {
        self.title
    }

    pub fn rows(self) -> impl Iterator<Item = BPreferenceRow> {
        UI_STYLE_DESCRIPTORS
            .iter()
            .filter(move |descriptor| descriptor.preference.section == self.title)
            .map(BPreferenceRow::from_descriptor)
    }

    pub fn len(self) -> usize {
        self.rows().count()
    }

    pub fn is_empty(self) -> bool {
        self.rows().next().is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BPreferenceDeclarationSet;

pub static SECTIONS: BPreferenceDeclarationSet = BPreferenceDeclarationSet;

impl BPreferenceDeclarationSet {
    pub fn iter(self) -> impl Iterator<Item = BPreferenceSection> {
        sections()
    }

    pub fn len(self) -> usize {
        self.iter().count()
    }

    pub fn is_empty(self) -> bool {
        UI_STYLE_DESCRIPTORS.is_empty()
    }
}

pub fn sections() -> impl Iterator<Item = BPreferenceSection> {
    UI_STYLE_DESCRIPTORS
        .iter()
        .enumerate()
        .filter(|(index, descriptor)| {
            !UI_STYLE_DESCRIPTORS[..*index]
                .iter()
                .any(|prior| prior.preference.section == descriptor.preference.section)
        })
        .map(|(_, descriptor)| BPreferenceSection {
            title: descriptor.preference.section,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declarations_expose_every_style_domain_in_descriptor_order() {
        let titles: Vec<_> = SECTIONS.iter().map(BPreferenceSection::title).collect();
        assert_eq!(
            titles,
            [
                "Colors",
                "Radii",
                "Typography",
                "Spacing",
                "Dimensions",
                "Strokes",
                "Resize",
                "Effects",
            ]
        );
        assert!(SECTIONS.iter().all(|section| !section.is_empty()));
    }

    #[test]
    fn rows_are_the_canonical_descriptors_without_parallel_metadata() {
        let rows: Vec<_> = SECTIONS.iter().flat_map(BPreferenceSection::rows).collect();
        assert_eq!(rows.len(), UI_STYLE_DESCRIPTORS.len());
        for (row, descriptor) in rows.into_iter().zip(UI_STYLE_DESCRIPTORS) {
            assert!(std::ptr::eq(row.descriptor(), descriptor));
            assert!(std::ptr::eq(row.key(), descriptor.key));
            assert_eq!(row.target(), descriptor.target);
            assert_eq!(row.parser(), descriptor.parser);
            assert_eq!(row.metadata(), &descriptor.preference);
        }
    }
}
