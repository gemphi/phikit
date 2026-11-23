//! What a room, a caller and a line carry, and what the rules read and
//! say: the limits of a call, the words on the wire, who is in a call,
//! what one caller sends another, what the line refuses and what it
//! tells a page. None of it has a network in it.

mod fault;
mod ice;
mod limit;
mod note;
mod roster;
mod signal;
mod wire;

pub use fault::{Fault, Refusal};
pub use ice::Ice;
pub use limit::{APP, Limit, PEER, ROOM};
pub use note::{Hold, Note, Tell};
pub use roster::Roster;
pub use signal::{Post, Signal};
pub use wire::Wire;
