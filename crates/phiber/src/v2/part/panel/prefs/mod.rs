//! Preferences context — the User Preferences screen, rendered with the exact
//! same NavBar + accordion-section machinery as the Properties (Buts) editor.
//! Blender ref: `bl_ui/space_userpref.py` + `rna_userdef.cc`.

pub mod control;
pub mod set;
pub use control::{
    BPreferenceControls, BPreferenceEditor, BPreferenceField, BPreferenceReset,
    BPreferenceSelector, BPreferenceToggle,
};
pub use set::{BPreferenceDeclarationSet, BPreferenceRow, BPreferenceSection, SECTIONS, sections};
