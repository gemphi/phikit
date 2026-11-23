//! OpenUSD in the world: a text layer (`.usda`) read into prims, a
//! stage that composes what they reference, and its prims put into the
//! world as words with their poses, types, properties and the
//! components Reality Composer Pro authors on them. A scene is data:
//! the same file Reality Composer Pro writes is what the world runs.

pub mod intake;
pub mod layer;
mod read;
pub mod stage;
pub mod token;
pub mod value;

pub use intake::{Component, ComponentSet, Kind, intake};
pub use layer::{Layer, Prim, Property, Spec};
pub use stage::Stage;
pub use value::Value;
