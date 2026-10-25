//! The sidebar panels of the editors whose tabs were names on a strip with
//! nothing behind them: the 3D Viewport's Tool and View extras, the
//! Sequencer, the Movie Clip editor, the Graph editor and the NLA — each
//! Blender's own panels (`bl_ui/space_*.py`), over `ctx.buts.<rna path>`
//! like every other property row.
//!
//! | editor | tab | panels |
//! |---|---|---|
//! | 3D Viewport | Tool | `VIEW3D_PT_tools_active` ▸ Options (`VIEW3D_PT_tools_object_options_transform`), Workspace (`VIEW3D_PT_tools_meshedit_options`'s object mode) |
//! | 3D Viewport | View | `VIEW3D_PT_view_3d_cursor`, `VIEW3D_PT_collections`, `VIEW3D_PT_annotations` |
//! | Sequencer | Strip | `SEQUENCER_PT_strip`, `_effect`, `_time`, `_adjust_transform`, `_adjust_color`, `_adjust_sound` |
//! | Sequencer | Modifiers | `SEQUENCER_PT_modifiers` |
//! | Sequencer | Preview | `SEQUENCER_PT_view`, `_frame_overlay`, `_view_safe_areas` |
//! | Movie Clip | Track | `CLIP_PT_track`, `CLIP_PT_track_settings`, `CLIP_PT_track_settings_extras` |
//! | Movie Clip | Solve | `CLIP_PT_tools_solve`, `CLIP_PT_tools_cleanup` |
//! | Graph | View | `GRAPH_PT_view` |
//! | Graph | F-Curve | `GRAPH_PT_properties`, `GRAPH_PT_key_properties` |
//! | Graph | Modifiers | `GRAPH_PT_modifiers` |
//! | Graph | Drivers | `GRAPH_PT_driven_property`, `GRAPH_PT_drivers` |
//! | NLA | Edited Action | `NLA_PT_animdata` |
//! | NLA | Item | `NLA_PT_stripname`, `NLA_PT_properties`, `NLA_PT_actionclip` |

mod view3d;
mod seq;
mod clip;
mod graph;
mod nla;

pub use view3d::*;
pub use seq::*;
pub use clip::*;
pub use graph::*;
pub use nla::*;
