//! Region — a slice inside a host (Header/Center/Footer/…). Owns its panes.

pub mod item;
pub mod list;
pub mod spec;
pub mod r#type;

pub use item::BRegionItem;
pub use list::BRegionList;
pub use spec::BRegionSpec;
pub use r#type::BRegionType;
