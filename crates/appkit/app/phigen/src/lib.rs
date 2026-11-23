//! Phigen: a chat, and the first app on appkit.
//!
//! It builds no window. It hands appkit its spaces, hosts, parts and
//! modals, read from the tables in `data/`, and the view of each part
//! (`part/`); appkit opens the window, lays it out for a desk or a hand,
//! and sends it what a hand does. What it looks like is `data/theme.tsv`
//! and what it says is `data/text.tsv`.
//!
//! `Reply` is where a model answers (`model.rs`); where none is connected
//! `Stand` stands in and says so.

pub mod app;
pub mod chat;
pub mod choice;
pub mod data;
pub mod date;
pub mod fact;
pub mod history;
pub mod keep;
pub mod model;
pub mod part;
pub mod reply;
pub mod run;
#[cfg(target_family = "wasm")]
pub mod web;

pub use app::Phigen;
pub use chat::{Chat, Turn, Who};
pub use choice::Choice;
pub use history::History;
pub use model::Model;
pub use reply::{Reply, Source, Stand, Whole};
pub use run::run;

#[cfg(test)]
mod test;
