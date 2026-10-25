//! The game's geometry, generated: everything a match is drawn with that
//! is not a player's body — the field and its markings, the goals, the
//! ball, the stadium, its seats and its crowd, and the kit, shoes and hair
//! the players wear — as plain meshes, images and materials. Phiber turns
//! these into its scene (`scene/play` in phiber); nothing here knows a
//! renderer.
//!
//! Every generator is a pure function of the sets (`BChoice`): the same
//! choice builds the same arena, so a choice can be rebuilt on any machine
//! and two players of a match see one stadium.

pub mod ball;
pub mod board;
pub mod body;
pub mod crowd;
pub mod dress;
pub mod field;
pub mod font;
pub mod footwear;
pub mod goal;
pub mod hair;
pub mod image;
pub mod kit;
pub mod mesh;
pub mod part;
pub mod ring;
pub mod seat;
pub mod shoe;
pub mod venue;

pub use image::BImage;
pub use mesh::BMesh;
pub use part::{BMat, BPart};
