//! Menubar menus - top-level dropdown menus (File, Edit, Render, Window, Help).
//!
//! Re-exports shared types from `super` so each menu file can `use super::*`.

pub mod edit;
pub mod file;
pub mod help;
pub mod render;
pub mod window;

pub use super::{BMenu, MenuSpec, MenuSpecProvider};
