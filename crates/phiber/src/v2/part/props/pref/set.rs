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

    /// The section's rows in groups of at most [`MAX_CHILDREN`], or `None`
    /// when it is short enough to stand as one list. A long section is never
    /// one run of thirty rows: it folds into named groups, each a nested
    /// accordion under it ([`GROUPS`]).
    pub fn groups(self) -> Option<Vec<(&'static str, Vec<BPreferenceRow>)>> {
        if self.len() <= MAX_CHILDREN {
            return None;
        }
        let mut out: Vec<(&'static str, Vec<BPreferenceRow>)> = Vec::new();
        for row in self.rows() {
            let group = group_of(self.title, row.metadata().label);
            match out.iter_mut().find(|(name, _)| *name == group) {
                Some((_, rows)) => rows.push(row),
                None => out.push((group, vec![row])),
            }
        }
        Some(out)
    }
}

/// The most children one accordion lists before its rows are grouped.
pub const MAX_CHILDREN: usize = 8;

/// Which group a long section's row belongs to: `(section, group, labels)`.
/// One table for every long section, each group at most [`MAX_CHILDREN`]
/// rows; a row it does not name falls into "Other".
pub static GROUPS: &[(&str, &str, &[&str])] = &[
    ("Colors", "Window", &["Application Background", "Activity Background", "Top Background", "Bottom Background",
        "Left Background", "Right Background", "Header Background", "Panel Background"]),
    ("Colors", "Controls", &["Viewport Background", "Control Background", "Hover Background", "Active Background", "Modal Dim"]),
    ("Colors", "Text & Accent", &["Text", "Dimmed Text", "Accent", "Soft Accent", "Online"]),
    ("Colors", "Lines & Rows", &["Border", "Separator", "Alternate Row", "Selected Row", "Active Row", "Matching Row"]),
    ("Colors", "Viewport", &["Viewport Grid", "X Axis", "Y Axis", "Z Axis", "Active Wire"]),
    ("Dimensions", "Bars", &["Header Height", "Footer Height", "Row Height", "Button Height", "Scrubbing Height",
        "Asset Shelf Height", "Bottom Strip Height", "Bottom Panel Height"]),
    ("Dimensions", "Panels", &["Menu Width", "Status Log Height", "Banner Height", "Panel Width", "Channels Width",
        "Tool Properties Width", "Number Field Minimum Width"]),
    ("Dimensions", "Rails & Areas", &["Activity Width", "Toolbar Width", "Sidebar Width", "Sidebar Minimum Width",
        "Navigation Width", "Area Left Width", "Area Right Width"]),
    ("Dimensions", "Columns", &["File Date Column Width", "File Size Column Width", "Indent Unit", "Restriction Column Width"]),
    ("Dimensions", "Icons", &["Icon Size", "Icon Size Small", "Brand Mark Size", "Gizmo Button Size", "Gizmo Icon Size",
        "Object Icon Size", "Object Glyph Size"]),
    ("Effects", "Gradients", &["Gradients", "Gradient Lift", "Sheen Lift", "Selected Hover Lift", "Strip Gradient Lift"]),
    ("Effects", "Surfaces", &["Footer Lift", "Sidebar Lift", "Card Lift", "Section Lift", "Strip Recess", "Surface Shadow"]),
    ("Effects", "Tabs", &["Tab Accent Mix", "Tab Saturation Lift", "Tab Top Lift"]),
    ("Effects", "Viewport & Nodes", &["Viewport Lightness Delta", "Viewport Grid Alpha", "Axis Alpha", "Node Grid Lift", "Node Grid Alpha"]),
    ("Effects", "Images & Motion", &["Image Light Lift", "Image Dark Lift", "Grip Alpha", "Motion Duration"]),
];

/// The group [`GROUPS`] puts `label` in within `section`.
pub fn group_of(section: &str, label: &str) -> &'static str {
    GROUPS
        .iter()
        .find(|(s, _, labels)| *s == section && labels.contains(&label))
        .map(|(_, group, _)| *group)
        .unwrap_or("Other")
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
    fn no_group_lists_more_than_the_most_children() {
        for section in SECTIONS.iter() {
            let Some(groups) = section.groups() else {
                assert!(section.len() <= MAX_CHILDREN, "{} is short", section.title());
                continue;
            };
            let total: usize = groups.iter().map(|(_, rows)| rows.len()).sum();
            assert_eq!(total, section.len(), "{}: every row lands in a group", section.title());
            for (name, rows) in groups {
                assert!(rows.len() <= MAX_CHILDREN, "{} / {name} lists {} rows", section.title(), rows.len());
                assert_ne!(name, "Other", "{}: every row is named in GROUPS", section.title());
            }
        }
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
