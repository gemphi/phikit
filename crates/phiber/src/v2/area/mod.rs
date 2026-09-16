//! Area — a slot inside a zone that holds one active host.

pub mod current;
pub mod item;
pub mod list;
pub mod spec;
pub mod r#type;

pub use current::{area_key, current_area, current_space, switcher_key, with_area, with_space};
pub use item::BAreaItem;
pub use list::BAreaList;
pub use spec::BAreaSpec;
pub use r#type::BAreaType;
