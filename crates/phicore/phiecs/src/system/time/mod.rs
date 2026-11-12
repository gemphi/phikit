//! Time: the systems that keep the journal and read it back.
mod dump;
pub mod record;
mod seek;
#[cfg(test)]
mod tests;
pub use record::{Recorder, record};
