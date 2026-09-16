//! Modal — a top-level overlay peer of Space. Opens on top of the active
//! space when `ctx.modal` is set. Each modal is a self-contained mini-app.

pub mod item;
pub mod list;
pub mod set;
pub mod spec;
pub mod r#type;

// Per-modal content — one folder per BModalType variant.
pub mod command;
pub mod file_browser;
pub mod help;
pub mod profile;
pub mod settings;
pub mod splash;

pub use item::BModalItem;
pub use list::BModalList;
pub use set::SET;
pub use spec::BModalSpec;
pub use r#type::BModalType;
