//! The Text editor's sidebar panels — `bl_ui/space_text.py`.
//!
//! Blender puts both of them under one `bl_category`, "Text":
//! `TEXT_PT_find` then `TEXT_PT_properties`. The tab set here had two tabs,
//! "Text" and "Properties", which is one more than Blender has and left
//! nothing obvious to put in either — so the set collapses to Blender's one
//! tab holding Blender's two panels.
//!
//! RNA: `SpaceTextEditor` (`rna_space.cc`) and `Text` (`rna_text.cc`). Note
//! which object each row belongs to: the editor owns how the text is shown
//! (font size, wrapping, the margin guide) and the datablock owns its own
//! indentation, because that travels with the file.

use crate::v2::part::props::prop::{BEnumItem, BPropSpec, b, e, en, i, s};
use crate::v2::part::props::set::{BSectionSpec, sec};

/// `Text.indentation` — what the Tab key inserts into this file.
static INDENT: &[BEnumItem] = &[en("TABS", "Tabs"), en("SPACES", "Spaces")];

/// `TEXT_PT_properties` — how the editor draws the text, in Blender's order.
///
/// `margin_column` is the guide's column and `show_margin` the checkbox that
/// enables it; Blender draws them on one row with the number greyed while the
/// box is off, so they stay adjacent here.
static PROPS: &[BPropSpec] = &[
    i("font_size", "Font Size", 8, 32),
    i("tab_width", "Tab Width", 2, 8),
    e("text.indentation", "Indentation", INDENT),
    b("show_margin", "Margin"),
    i("margin_column", "Margin Column", 0, 1024),
    b("show_line_numbers", "Line Numbers"),
    b("show_word_wrap", "Word Wrap"),
    b("show_syntax_highlight", "Syntax Highlight"),
    b("show_line_highlight", "Highlight Line"),
    b("use_live_edit", "Live Edit"),
];

/// `TEXT_PT_find` — the find and replace fields and how they match.
static FIND: &[BPropSpec] = &[
    s("find_text", "Find"),
    s("replace_text", "Replace"),
    b("use_match_case", "Match Case"),
    b("use_find_wrap", "Wrap Around"),
    b("use_find_all", "All Data-Blocks"),
];

/// The Text tab: find and replace over the properties, as `space_text.py`
/// registers them. Find is closed by default — it is the panel you open to
/// do a thing, not the one you read.
pub static TEXT: &[BSectionSpec] = &[FIND_PANEL, sec("Properties", PROPS)];

const FIND_PANEL: BSectionSpec = sec("Find & Replace", FIND).closed();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panels_are_blenders_two_in_blenders_order() {
        assert_eq!(TEXT.iter().map(|s| s.title).collect::<Vec<_>>(), vec!["Find & Replace", "Properties"]);
        assert!(TEXT[0].closed, "Find & Replace is a thing you open");
        assert!(!TEXT[1].closed, "Properties is a thing you read");
    }

    #[test]
    fn the_editor_owns_the_view_and_the_datablock_owns_its_indentation() {
        // One row in this table is the file's, not the editor's: indentation
        // travels with the text, so it is keyed on the datablock.
        let datablock: Vec<_> = PROPS.iter().filter(|p| p.key.starts_with("text.")).map(|p| p.key).collect();
        assert_eq!(datablock, vec!["text.indentation"]);
    }

    #[test]
    fn the_margin_guide_and_its_column_stay_adjacent() {
        // Blender draws them on one row, the number greyed while the box is
        // off. Separating them loses that they are one control.
        let keys: Vec<_> = PROPS.iter().map(|p| p.key).collect();
        let box_at = keys.iter().position(|k| *k == "show_margin").expect("the checkbox");
        assert_eq!(keys[box_at + 1], "margin_column");
    }

    #[test]
    fn every_row_names_an_rna_property_and_a_label() {
        for section in TEXT {
            assert!(!section.props.is_empty(), "{} draws nothing", section.title);
            for prop in section.all_props() {
                assert!(!prop.key.is_empty(), "{} has a row with no key", section.title);
                assert!(!prop.label.is_empty(), "{} has a row with no label", section.title);
            }
        }
    }
}
