//! The peer: this window as a caller. Round one made the line and
//! showed who is on it; this is the other side of it, what a browser's
//! page does in `puikit/app/phi-kit/src/call.ts` and `peer.ts`, so the
//! window can be in a room's call itself.
//!
//! * `line`: a room's line as a caller holds it, over HTTP as a page
//!   does.
//! * `mesh`: one connection to each other caller, and the rule of who
//!   gives way when both offer at once.
//! * `engine`: what a connection is asked and what it makes, which is
//!   WebRTC's part; `stand` is one that carries nothing, for the tests
//!   and for a window built without WebRTC; `rtc` is the real library.
//! * `call`: a call this window is in, on a thread of its own.
//!
//! Its words are `data/signal.tsv`, what it asks of the line
//! `data/dial.tsv`, and its numbers `data/peer.tsv`.

mod call;
mod engine;
mod line;
mod mesh;
#[cfg(all(feature = "rtc", not(target_family = "wasm")))]
pub mod rtc;
mod stand;

pub use call::{Call, Seen, name};
pub use engine::{Engine, Kind, Link, Made, Maker, Own, Stage};
pub use line::{Heard, Line};
pub(crate) use line::{head, reach};
pub use mesh::{Far, Mesh, Out, Picture};
pub use stand::Stand;

/// The name a picture of a call has in the window's film: how such
/// names begin, whose picture it is, and the stream it comes under.
pub fn shown(begin: &str, who: &str, stream: &str) -> String {
    format!("{begin}{who}/{stream}")
}

/// That name, as `data/peer.tsv` begins it.
pub fn film(table: &appkit::service::PTable, who: &str, stream: &str) -> String {
    shown(table.get("film"), who, stream)
}
