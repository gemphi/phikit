//! appkit: the common layer every phi app stands on.
//!
//! `pui` is the UI kit, in two halves. What a thing on screen is: something
//! drawn, a view, a control, a widget (`pui::core`, `pui::control`,
//! `pui::view`, `pui::widget`). And how a window is built: a space, a
//! zone, an area, a host, a region, a pane, a part, a modal
//! (`pui::window`). appkit creates the window; an app supplies its entries
//! and the content of its parts.
//!
//! Beside the kit: `theme`, what everything looks like; `service`, what a
//! part asks for that is not drawn; and `screen`, which shows an app's
//! stage in a real window and is the one place that names gpui. The numbers and tables are in
//! `src/data`, the tests in `src/test`, and the icons every app shares in
//! `asset/` at appkit's root. An app keeps its own `asset/` and `data/`
//! in its folder under `app/`.
//!
//! Every type starts with `P`. Every name is one word and singular. The
//! plan, the hierarchy and the order of work are in `../PLAN.md`.

#[path = "../pui/mod.rs"]
pub mod pui;
pub mod screen;
pub mod service;
pub mod theme;

#[cfg(test)]
mod test;
