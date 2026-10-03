//! The node editors' sidebar panels — `bl_ui/space_node.py`.
//!
//! Four hosts, one space type (`SpaceNodeEditor`) with a different tree in
//! it: the Shader editor, the generic Node editor, Geometry Nodes and the
//! Compositor. Blender registers most of the N panel once for all of them
//! and polls on the tree type, so the tables here are shared and only the
//! Compositor's Options tab is its own.
//!
//! | tab | panels | trees |
//! |---|---|---|
//! | Node | `NODE_PT_active_node_generic` ▸ `NODE_PT_active_node_color` | all |
//! | Group | `NODE_PT_node_tree_properties` | all but the Compositor |
//! | Options | `NODE_PT_quality` | the Compositor's |
//!
//! What is NOT here, deliberately: `NODE_PT_active_node_properties`, which
//! calls the active node's own `draw_buttons_ext`. That is per node type,
//! not a table, so declaring it would mean inventing rows — the Shader and
//! Node editors' Options tabs and Geometry Nodes' Attributes tab are empty
//! for the same reason and stay named in
//! `tests/an_open_sidebar_paints_its_panels.rs`.
//!
//! RNA: `Node` and `NodeTree` (`rna_nodetree.cc`), `SpaceNodeEditor`
//! (`rna_space.cc`).

use crate::v2::part::props::prop::{BEnumItem, BPropSpec, b, col, e, en, s};
use crate::v2::part::props::set::{BSectionSpec, sec};

/// `NODE_PT_active_node_generic`: what the active node is called.
static GENERIC: &[BPropSpec] = &[s("node.name", "Name"), s("node.label", "Label")];

/// `NODE_PT_active_node_color`'s header is the checkbox that enables it, and
/// its body the swatch — `layout.prop(node, "use_custom_color", text="")`
/// then `layout.prop(node, "color", text="")`.
static COLOR_ON: &[BPropSpec] = &[b("node.use_custom_color", "Custom Color")];
static COLOR: &[BPropSpec] = &[col("node.color", "Color", false)];

/// `NODE_PT_node_tree_properties`: the tree being edited, which in a group
/// is the group's own datablock.
static TREE: &[BPropSpec] = &[s("node_tree.name", "Name"), s("node_tree.description", "Description")];

/// `NodeTree.precision` — how the compositor evaluates.
static PRECISION: &[BEnumItem] = &[en("AUTO", "Auto"), en("FULL", "Full")];

/// `NodeTree.render_quality` / `edit_quality`.
static QUALITY_LEVEL: &[BEnumItem] =
    &[en("HIGH", "High"), en("MEDIUM", "Medium"), en("LOW", "Low")];

/// `NodeTree.chunk_size` — the tile the compositor works in.
static CHUNK: &[BEnumItem] = &[
    en("32", "32px"),
    en("64", "64px"),
    en("128", "128px"),
    en("256", "256px"),
    en("512", "512px"),
    en("1024", "1024px"),
];

/// `NODE_PT_quality`, the Compositor's Options tab.
static QUALITY: &[BPropSpec] = &[
    e("node_tree.precision", "Precision", PRECISION),
    e("node_tree.render_quality", "Render", QUALITY_LEVEL),
    e("node_tree.edit_quality", "Edit", QUALITY_LEVEL),
    e("node_tree.chunk_size", "Chunk Size", CHUNK),
];

/// The Node tab: the active node, with its colour as a closed sub-panel, as
/// `bl_parent_id` and `DEFAULT_CLOSED` have it.
pub static NODE: &[BSectionSpec] =
    &[sec("Node", GENERIC).subs(COLOR_SUB)];

const COLOR_SUB: &[BSectionSpec] = &[COLOR_PANEL];
const COLOR_PANEL: BSectionSpec = sec("Color", COLOR).header(COLOR_ON).closed();

/// The Group tab: the tree being edited.
pub static GROUP: &[BSectionSpec] = &[sec("Group", TREE)];

/// The Compositor's Options tab.
pub static OPTIONS: &[BSectionSpec] = &[sec("Performance", QUALITY)];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_colour_panel_is_a_closed_sub_panel_of_the_node() {
        let node = &NODE[0];
        assert_eq!(node.title, "Node");
        assert_eq!(node.list.len(), 1, "Color hangs off it by `bl_parent_id`");
        let color = &node.list[0];
        assert_eq!(color.title, "Color");
        assert!(color.closed, "DEFAULT_CLOSED");
        assert_eq!(color.header.len(), 1, "the checkbox is drawn in the header");
        assert_eq!(color.header[0].key, "node.use_custom_color");
    }

    #[test]
    fn every_row_names_an_rna_property_and_a_label() {
        for section in NODE.iter().chain(GROUP).chain(OPTIONS) {
            for prop in section.all_props() {
                assert!(!prop.key.is_empty(), "{} has a row with no key", section.title);
                assert!(!prop.label.is_empty(), "{} has a row with no label", section.title);
            }
        }
    }

    #[test]
    fn the_quality_rows_are_the_trees_own_and_not_the_spaces() {
        // `NODE_PT_quality` reads the tree, not the editor: two compositor
        // editors on one tree show one set of settings.
        for prop in QUALITY {
            assert!(prop.key.starts_with("node_tree."), "{} is not the tree's", prop.key);
        }
    }
}
