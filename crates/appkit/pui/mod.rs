//! pui: the UI kit. What a thing on screen is (`core`, `control`, `view`,
//! `widget`, `layout`), how a window is built (`window`), and what an app
//! supplies to get one (`PApp`, in `app.rs`).

mod app;
pub mod control;
pub mod core;
pub mod layout;
pub mod view;
pub mod widget;
pub mod window;

pub use app::PApp;
