//! A `SceneModel` from Blender data: the conversion from a read `BlendScene`
//! (`import`, `object`, `share`), the progressive open (`progressive`), the
//! scene built from code (`build`), and the parts of the model that are
//! Blender's own (rigs, motion, collections, relation lines).
pub mod build;
pub mod collection;
pub(crate) mod import;
pub(crate) mod import_extra;
pub mod motion;
pub(crate) mod object;
pub mod progressive;
pub(crate) mod progressive_link;
pub mod relation;
pub mod rig;
pub(crate) mod share;#[cfg(test)]
#[path = "../../../test/read/scene/model/mod.rs"]
mod test;
