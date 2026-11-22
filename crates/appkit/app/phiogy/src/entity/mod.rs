//! What there is in a call: a room, a caller, a line. Each is a word of
//! the board's world (`phiecs`), and what is known of it an aspect bound
//! to the word: these types.

mod caller;
mod line;
mod room;

pub use caller::Caller;
pub use line::Line;
pub use room::Room;
