//! The session's edits as one value, and how they land in the file.
//!
//! What a save writes through: the current frame and the range
//! (`Scene.r.cfra / sfra / efra`), each object's restriction toggles
//! (`Object.visibility_flag`), its name (`ID.name`), whether the view
//! layer hides or selects it (`Base.flag`), the active object
//! (`ViewLayer.basact`), each collection's eye and toggles
//! (`LayerCollection.flag`, `Collection.flag`), the marker list, and the
//! objects the session added (`object.rs`, `mesh.rs`, `link.rs`),
//! duplicated (a linked copy sharing the mesh) or deleted. A mesh can only
//! be written into a file saved by Blender 3.6 or later; for an older
//! file the new objects come back as a warning in [`BlendSaveReport`], so
//! the save says what it left out.

use super::layer::BlendCollectionAt;
use super::mesh::BlendNewMesh;
use crate::device::blend::{BlendShadowCast, BlendVisibility};
use crate::device::blend::file::kind::*;
use crate::device::blend::read::object::layer::{BASE_HIDDEN, BASE_SELECTED, LAYER_COLLECTION_EXCLUDE, LAYER_COLLECTION_HIDE};
use crate::device::blend::read::object::visibility::{COLLECTION_HIDE_RENDER, COLLECTION_HIDE_SELECT, COLLECTION_HIDE_VIEWPORT, OB_HIDE_RENDER, OB_HIDE_SELECT, OB_HIDE_VIEWPORT};

/// One object's edits, keyed by the name the file has for it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendObjectEdit {
    pub name: String,
    /// The name it has now, when renamed.
    pub rename: Option<String>,
    /// The monitor / arrow / camera toggles.
    pub visibility: Option<BlendVisibility>,
    /// The eye: `BASE_HIDDEN` in the view layer.
    pub hidden: Option<bool>,
    /// `BASE_SELECTED`.
    pub selected: Option<bool>,
    /// Location, XYZ Euler rotation (radians) and scale, when the session
    /// changed them (the Transform panel); written with `rotmode` XYZ.
    pub transform: Option<([f32; 3], [f32; 3], [f32; 3])>,
    /// Viewport Display ▸ Shadow (`dtx`'s `OB_DRAW_NO_SHADOW_CAST`) and
    /// Ray Visibility ▸ Shadow (`OB_HIDE_SHADOW`), when the panel set them.
    pub shadow: Option<crate::device::blend::BlendShadowCast>,
}

/// One collection's edits, keyed by its path (`Scene Collection/Props`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendCollectionEdit {
    pub path: String,
    /// The eye: `LAYER_COLLECTION_HIDE`.
    pub hidden: Option<bool>,
    /// The checkbox: `LAYER_COLLECTION_EXCLUDE`.
    pub exclude: Option<bool>,
    /// `Collection.flag`'s toggles.
    pub visibility: Option<BlendVisibility>,
}

/// An object the session added: a primitive at `location`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendNewObject {
    pub name: String,
    pub mesh: BlendNewMesh,
    pub location: [f32; 3],
    pub scale: [f32; 3],
    /// The collection to put it in, by path; the scene collection when empty.
    pub collection: String,
    pub selected: bool,
    pub visibility: Option<BlendVisibility>,
}

/// A duplicate: `copy` is `source` again, sharing its mesh.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendDuplicate {
    pub copy: String,
    pub source: String,
    pub selected: bool,
}

/// Everything a save writes through.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendEditList {
    pub added: Vec<BlendNewObject>,
    pub duplicated: Vec<BlendDuplicate>,
    /// Objects removed, by the file's name for them.
    pub deleted: Vec<String>,
    pub frame: Option<i32>,
    pub frame_start: Option<i32>,
    pub frame_end: Option<i32>,
    pub objects: Vec<BlendObjectEdit>,
    /// The active object, by the file's name for it.
    pub active: Option<String>,
    /// The whole marker list, when it was edited.
    pub markers: Option<Vec<(i32, String)>>,
    /// The Playback popover's settings (`write/playback.rs`).
    pub playback: Option<crate::device::blend::BlendPlayback>,
    /// The 3D cursor, when the session moved it (`write/cursor.rs`).
    pub cursor: Option<[[f32; 4]; 4]>,
    /// Output ▸ Format's Resolution X, Y and %, when the session changed
    /// them (`Scene.r.xsch / ysch / size`).
    pub resolution: Option<[i32; 3]>,
    /// Color Management's View Transform, Exposure and Gamma, when the
    /// session changed them (`Scene.view_settings`).
    pub view: Option<(String, f32, f32)>,
    /// The render engine, when the session changed it (`Scene.r.engine`).
    pub engine: Option<String>,
    /// Film ▸ Transparent, when the session changed it (`Scene.r.alphamode`).
    pub film_transparent: Option<bool>,
    /// Output's path and image format, when the session changed them
    /// (`Scene.r.pic`, `Scene.r.im_format`; `write/render.rs`).
    pub output: Option<crate::device::blend::BlendOutput>,
    /// The render region, when the session changed it (`Scene.r.border`,
    /// `R_BORDER` and `R_CROP` in `Scene.r.mode`).
    pub border: Option<crate::device::blend::BlendRenderBorder>,
    /// Render-panel values the session changed, by their name in
    /// `scene["cycles"]` (`write/render.rs`).
    pub cycles: Vec<(String, crate::device::blend::BIdValue)>,
    pub collections: Vec<BlendCollectionEdit>,
    /// Objects the Outliner moved to another collection: the object's name
    /// (as saved, renames applied) and the collection's path.
    pub moved: Vec<(String, String)>,
    /// Objects the Outliner parented: the child's name and the parent's.
    pub parented: Vec<(String, String)>,
    /// Objects the Outliner linked into another collection as well (Ctrl
    /// drag): the object's name and the collection's path.
    pub linked: Vec<(String, String)>,
    /// Collections the Outliner nested in another: each one's name and its
    /// new parent's.
    pub nested: Vec<(String, String)>,
    /// Actions the session made (`action.new`): `(name, source)` — a copy
    /// of `source`, or an empty action when `source` is empty.
    pub actions: Vec<(String, String)>,
    /// Objects given another action, or none (`write/action.rs`).
    pub assigned: Vec<super::action::BlendActionAssign>,
    /// Actions' Manual Frame Range and Cyclic Animation.
    pub ranges: Vec<super::action::BlendActionRange>,
    /// Keys inserted (`I`): each object's touched curves, whole.
    pub keys: Vec<super::key::BlendKeyInsert>,
    /// Each object's NLA as the session left it — Push Down, Stash, the
    /// NLA editor's edits (`write/nla.rs`).
    pub nla: Vec<super::nla::BlendNlaStack>,
    /// Edits the writer cannot make yet, said in the user's terms — added,
    /// duplicated or deleted objects. Reported, not written.
    pub unwritable: Vec<String>,
}

/// What a save did: how many fields it patched, what it had to leave out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlendSaveReport {
    pub patched: usize,
    pub warnings: Vec<String>,
}

/// `Object.rotmode` for XYZ Euler (`ROT_MODE_XYZ`).
const ROT_MODE_XYZ: i64 = 1;

fn with_bit(flag: u32, bit: u32, on: bool) -> u32 {
    if on { flag | bit } else { flag & !bit }
}
mod apply;
mod structure;

#[cfg(test)]
#[path = "../../test/write/edit.rs"]
mod edit_test;
