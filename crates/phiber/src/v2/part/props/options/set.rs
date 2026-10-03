//! What the Properties header's Options popover holds —
//! `PROPERTIES_PT_options`.
//!
//! Blender ref: `bl_ui/space_properties.py` (`PROPERTIES_PT_options.draw`)
//! and `SpaceProperties.outliner_sync` in `makesrna/intern/rna_space.cc`.
//!
//! The panel is one control: whether the Properties tab follows the
//! Outliner's selection. It is drawn `expand=True`, so the three choices sit
//! in a row rather than behind a dropdown, under their own label.
//!
//! One row today, and a table anyway: the popover is Blender's panel, and
//! when Blender adds a second control to it this is where the row goes —
//! not a second `if` in the builder beside it.

/// A choice in the Options popover: the RNA name, its label as Blender
/// writes it, and what it ships with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPropsOptionSpec {
    /// The RNA name, which is also the key under `ctx.buts.options`.
    pub key: &'static str,
    /// The label over the row, as `col.label(text=…)` writes it.
    pub heading: &'static str,
    /// The enum's items, in Blender's order: value then label.
    pub items: &'static [(&'static str, &'static str)],
    /// What Blender ships with, used until the row is touched.
    pub default: &'static str,
}

/// The popover's rows, in Blender's order.
pub static OPTIONS: [BPropsOptionSpec; 1] = [SYNC];

/// `SpaceProperties.outliner_sync` — whether picking in the Outliner
/// switches the Properties tab. `AUTO` is Blender's default: it follows the
/// Outliner only while the Outliner is in a mode that has a tab to follow.
pub const SYNC: BPropsOptionSpec = BPropsOptionSpec {
    key: "outliner_sync",
    heading: "Sync with Outliner",
    items: &[("ALWAYS", "Always"), ("NEVER", "Never"), ("AUTO", "Auto")],
    default: "AUTO",
};
