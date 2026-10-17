//! A project bundle, written from any asset [`phiphx`] can read.
//!
//! Opening a `.blend`, a `.gltf` or a `.usd` gives a scene in memory;
//! *keeping* one gives a directory the app can reopen without parsing the
//! original again. That directory is this crate: a manifest saying what the
//! project is, and a scene file saying what is in it, laid out the way
//! Reality Composer Pro lays a project out.
//!
//! It is a crate rather than a module of `phiphx` because the two jobs pull
//! in opposite directions. `phiphx` reads formats and owns no files;
//! this writes files and reads no formats. It had been a module —
//! `phiphx/src/import/project.rs`, behind an `appkit` feature that was
//! never switched on — and it did not compile: the feature gate hid a
//! `use crate::appkit::project::scaffold` whose module does not exist. A
//! dead module inside a live crate is invisible; a crate of its own is
//! built and tested like everything else.
//!
//! ```ignore
//! let write = BProjectWrite::new(Path::new("rig.blend"), Path::new("/out"));
//! let made = write.run(&|part, note| println!("{part:.0}% {note}"))?;
//! println!("{}", made.root.display());
//! ```
//!
//! The pieces: [`bundle::BBundle`] is the directory and its parts,
//! [`bundle::manifest::BManifest`] is `project.json`,
//! [`bundle::scene::BScene`] is the entity graph and its file, and
//! [`import::BProjectWrite`] is the one call that does all three with
//! progress.

pub mod bundle;
pub mod import;

pub use bundle::{BBundle, BBundlePart, BProjectName};
pub use import::{BProjectMade, BProjectWrite, BWriteError};
