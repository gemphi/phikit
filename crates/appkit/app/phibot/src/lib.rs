//! The botanist: Apple's BOT-anist sample as an app on appkit.
//!
//! The robot and its greenhouse are phiace's world on phiecs
//! (`phiace::bot::botanist`): what the robot is and does is phiace's
//! tables, its scenes Apple's own USD. This app is only the screens that
//! show it, as BOT-anist's are: loading, the creator (the customiser at
//! the left, the robot at the right, Start Planting), and the greenhouse
//! with its score, Replay and Exit Game. Its window, look and words are
//! the files in `data/`.

pub mod app;
pub mod data;
pub mod part;
pub mod run;

pub use app::Phibot;
pub use run::run;

#[cfg(test)]
mod test;
