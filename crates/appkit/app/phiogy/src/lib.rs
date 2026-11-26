//! Phiogy: the app that carries the calls, on appkit, as Phigen is.
//!
//! Three things, each on the one before:
//!
//! * The rules of a call, with no network in them, in the shape of
//!   entity, comp and system: a room, a caller and a line (`entity/`);
//!   who is in a call, what one sends another, the limits (`comp/`);
//!   join, pass, leave and beat (`system/`). They are kept on a board
//!   (`board.rs`), a world of `phiecs`.
//! * The line: the routes pages find each other on, served on those
//!   rules (`exchange/`). They are the Node line's, so the web pages
//!   speak to this one unchanged.
//! * The window (`app.rs`, `part/`): the web Phiogy's own hosts and
//!   rooms (`site.rs`), asked of Phiora first (`phiora.rs`), and, for a
//!   room, what was said in it and who is on its line.
//!
//! It builds no window of its own: appkit opens it. Every limit, every
//! word on the wire and every word a person reads is a file in `data/`.

pub mod app;
pub mod board;
pub mod choice;
pub mod comp;
pub mod data;
pub mod entity;
#[cfg(not(target_family = "wasm"))]
pub mod exchange;
pub mod fact;
pub mod hosted;
pub mod hour;
pub mod kept;
#[cfg(not(target_family = "wasm"))]
pub mod library;
pub mod make;
pub mod net;
pub mod part;
pub mod peer;
pub mod phiora;
pub mod records;
pub mod run;
pub mod site;
pub mod system;
#[cfg(target_family = "wasm")]
pub mod web;

pub use app::Phiogy;
pub use board::Board;
pub use choice::Choice;
pub use run::run;
pub use site::Site;

#[cfg(test)]
mod test;
