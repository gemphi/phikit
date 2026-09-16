//! Space — workspace tab. Per-space content in sibling modules.

pub mod item;
pub mod list;
pub mod set;
pub mod spec;
pub mod r#type;

// Per-space content — one file per Blender workspace.
pub mod animation;
pub mod compositing;
pub mod geometry_nodes;
pub mod layout;
pub mod modeling;
pub mod rendering;
pub mod scripting;
pub mod sculpting;
pub mod shading;
pub mod texture_paint;
pub mod uv_editing;

pub use item::BSpaceItem;
pub use list::BSpaceList;
pub use set::SET;
pub use spec::BSpaceSpec;
pub use r#type::BSpaceType;
