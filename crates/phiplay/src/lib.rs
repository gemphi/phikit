//! # phiplay — the game Phiber plays
//!
//! Five sports — basketball, netball, volleyball, beach volleyball and
//! soccer — and everything a match of one is made of, each said once as a
//! declarative set the way Phiber says its workspaces (`space/<name>/set.rs`):
//! a `mod.rs` holding the type and its shape, a `set.rs` holding the names,
//! and an index whose exhaustive `match` stops the build when a variant has
//! no spec.
//!
//! | set | how many | what it chooses |
//! |---|---|---|
//! | [`sport`] | 5 | the field, its markings, the ball, the goals, how many a side, the positions, the rules |
//! | [`team`] | 16 | a name, a code and three colours |
//! | [`kit::uniform`] | 8 | the cut and the pattern of the strip |
//! | [`kit::sneaker`] | 8 | the shoe: collar height (all above the ankle), sole, colour blocking |
//! | [`look::hair`] | 16 | a hairstyle, grown from parameters, not modelled |
//! | [`look`] | 8 | skin tone and build |
//! | [`theme`] | 8 | the palette the venue's lights and boards run |
//! | [`venue`] | 8 | the stadium: its shell, its façade's light show, its seats |
//! | [`turf`] | 8 | what the field is made of |
//!
//! A [`choice::BChoice`] picks one of each, and the eight-way picker in
//! Phiber's Play host is these sets looped over, never a list of its own.
//!
//! The play runs on phiecs (`crates/phicore/phiecs`, the core of phinet's): RealityKit's
//! entity-component-system with the entity a word. Players, the ball and the
//! goals are words; what they do to each other — who guards whom, who is
//! open to whom, who holds the ball — are relations; the phases of a match
//! are a `Condition` with the transitions each phase allows, as BOT-anist's
//! `AnimationState` lists its valid next states (`state`).

pub mod choice;
pub mod colour;
pub mod field;
pub mod build;
pub mod kit;
pub mod look;
pub mod pick;
pub mod play;
pub mod pose;
pub mod sport;
pub mod team;
pub mod theme;
pub mod turf;
pub mod venue;

pub use choice::BChoice;
pub use colour::Rgb;
