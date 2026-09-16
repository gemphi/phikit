//! Host — one editor kind. `host/<name>/` folders own region declarations only.
//! Widget content lives in `part::custom::<name>`.

pub mod item;
pub mod list;
pub mod set;
pub mod spec;
pub mod r#type;

// Per-host region declarations — one folder per BHostType variant.
pub mod action;
pub mod activity;
pub mod asset;
pub mod buts;
pub mod clip;
pub mod compositor;
pub mod console;
pub mod drivers;
pub mod file;
pub mod geonode;
pub mod image;
pub mod info;
pub mod ipo;
pub mod logic;
pub mod nla;
pub mod node;
pub mod outliner;
pub mod prefs;
pub mod seq;
pub mod shader;
pub mod spreadsheet;
pub mod statusbar;
pub mod text;
pub mod time;
pub mod topbar;
pub mod uv;
pub mod view3d;

pub use item::BHostItem;
pub use list::BHostList;
pub use set::SET;
pub use spec::BHostSpec;
pub use r#type::{BHostType, host_slug};
