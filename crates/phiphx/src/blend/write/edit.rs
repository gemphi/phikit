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
use crate::blend::{BlendShadowCast, BlendVisibility};
use crate::blend::file::types::*;
use crate::blend::object::layer::{BASE_HIDDEN, BASE_SELECTED, LAYER_COLLECTION_EXCLUDE, LAYER_COLLECTION_HIDE};
use crate::blend::object::visibility::{COLLECTION_HIDE_RENDER, COLLECTION_HIDE_SELECT, COLLECTION_HIDE_VIEWPORT, OB_HIDE_RENDER, OB_HIDE_SELECT, OB_HIDE_VIEWPORT};

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
    pub shadow: Option<crate::blend::BlendShadowCast>,
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
pub struct BlendEdits {
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
    pub playback: Option<crate::blend::BlendPlayback>,
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
    pub output: Option<crate::blend::BlendOutput>,
    /// The render region, when the session changed it (`Scene.r.border`,
    /// `R_BORDER` and `R_CROP` in `Scene.r.mode`).
    pub border: Option<crate::blend::BlendRenderBorder>,
    /// Render-panel values the session changed, by their name in
    /// `scene["cycles"]` (`write/render.rs`).
    pub cycles: Vec<(String, crate::blend::BIdValue)>,
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
    pub keys: Vec<super::keys::BlendKeyInsert>,
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

impl BlendEdits {
    /// Write the edits through `file`.
    pub fn apply(&self, file: &mut BlendFile) -> BlendSaveReport {
        let mut report = BlendSaveReport { patched: 0, warnings: self.unwritable.clone() };
        let Some(dna) = file.dna.clone() else {
            report.warnings.push("The file carries no DNA: nothing can be written".to_string());
            return report;
        };
        if file.current_scene_index().is_none() {
            report.warnings.push("No scene in the file".to_string());
            return report;
        }
        let mut patched = self.restructure(file, &mut report);
        let Some(scene) = file.current_scene_index() else { return report };
        let set = |patched: &mut usize, file: &mut BlendFile, index: usize, s: &str, f: &str, v: i64| {
            if file.set_int(index, s, f, v) {
                *patched += 1;
            }
        };
        let set_f32 = |patched: &mut usize, file: &mut BlendFile, index: usize, f: &str, v: f32| {
            if file.set_f32(index, "Object", f, v) {
                *patched += 1;
            }
        };
        for (field, value) in [("r.cfra", self.frame), ("r.sfra", self.frame_start), ("r.efra", self.frame_end)] {
            if let Some(value) = value {
                set(&mut patched, file, scene, "Scene", field, i64::from(value));
            }
        }
        if let Some(playback) = &self.playback {
            patched += file.write_playback(scene, &dna, playback);
        }
        for (field, value) in ["r.xsch", "r.ysch", "r.size"].into_iter().zip(self.resolution.into_iter().flatten()) {
            set(&mut patched, file, scene, "Scene", field, i64::from(value));
        }
        if let Some((transform, exposure, gamma)) = &self.view {
            let wrote = [
                file.set_text(scene, "Scene", "view_settings.view_transform", transform),
                file.set_f32(scene, "Scene", "view_settings.exposure", *exposure),
                file.set_f32(scene, "Scene", "view_settings.gamma", *gamma),
            ];
            patched += wrote.iter().filter(|w| **w).count();
        }
        if let Some(cursor) = self.cursor {
            patched += file.write_cursor(scene, &dna, cursor);
        }
        if let Some(engine) = &self.engine {
            patched += usize::from(file.set_text(scene, "Scene", "r.engine", engine));
        }
        if let Some(transparent) = self.film_transparent {
            patched += usize::from(file.set_int(scene, "Scene", "r.alphamode", i64::from(transparent)));
        }
        if let Some(output) = &self.output {
            patched += file.write_output(scene, output);
        }
        if let Some(border) = &self.border {
            let mode = file.field_span("Scene", "r.mode", &dna).filter(|(o, n)| *n == 4 && o + 4 <= file.blocks[scene].data.len()).map_or(0, |(o, _)| file.read_i32(&file.blocks[scene].data[o..o + 4])) as u32;
            patched += usize::from(file.set_int(scene, "Scene", "r.mode", i64::from(border.mode_over(mode))));
            for (field, value) in ["xmin", "xmax", "ymin", "ymax"].into_iter().zip(border.rect) {
                patched += usize::from(file.set_f32(scene, "Scene", &format!("r.border.{field}"), value));
            }
        }
        patched += file.write_cycles(scene, &dna, &self.cycles, &mut report.warnings);

        let view_layer = file.view_layer_index(scene);
        let bases = view_layer.map(|vl| file.bases(vl)).unwrap_or_default();
        let mut active_base: Option<u64> = None;
        for edit in &self.objects {
            let Some(object) = file.find_id("OB", &edit.name) else {
                report.warnings.push(format!("'{}' is not in the file", edit.name));
                continue;
            };
            if edit.visibility.is_some() || edit.shadow.is_some() {
                let field = file.object_visibility_field(&dna);
                let mut flag = file.int_field(&file.blocks[object], "Object", field, &dna);
                if let Some(v) = edit.visibility {
                    flag = with_bit(with_bit(with_bit(flag, OB_HIDE_VIEWPORT, v.hide_viewport), OB_HIDE_SELECT, v.hide_select), OB_HIDE_RENDER, v.hide_render);
                }
                if let Some(shadow) = edit.shadow {
                    flag = with_bit(flag, BlendShadowCast::OB_HIDE_SHADOW, !shadow.ray);
                }
                set(&mut patched, file, object, "Object", field, i64::from(flag));
            }
            if let Some(shadow) = edit.shadow {
                let dtx = file.int_field(&file.blocks[object], "Object", "dtx", &dna);
                set(&mut patched, file, object, "Object", "dtx", i64::from(with_bit(dtx, BlendShadowCast::OB_DRAW_NO_SHADOW_CAST, !shadow.display)));
            }
            if let Some((loc, rot, scale)) = edit.transform {
                for axis in 0..3 {
                    set_f32(&mut patched, file, object, &format!("loc[{axis}]"), loc[axis]);
                    set_f32(&mut patched, file, object, &format!("rot[{axis}]"), rot[axis]);
                    set_f32(&mut patched, file, object, &format!("size[{axis}]"), scale[axis]);
                }
                set(&mut patched, file, object, "Object", "rotmode", ROT_MODE_XYZ);
            }
            let address = file.blocks[object].old_address;
            if let Some(base) = bases.iter().find(|b| b.object_address == address) {
                if edit.hidden.is_some() || edit.selected.is_some() || edit.visibility.is_some() {
                    let mut flag = file.int_field(&file.blocks[base.base], "Base", "flag", &dna);
                    if let Some(hidden) = edit.hidden {
                        flag = with_bit(flag, BASE_HIDDEN, hidden);
                    }
                    if let Some(selected) = edit.selected {
                        flag = with_bit(flag, BASE_SELECTED, selected);
                    }
                    // What cannot be seen cannot be selected: Blender drops the
                    // selection of a hidden or viewport-disabled object when it
                    // evaluates the bases (`BKE_base_eval_flags`), so a file
                    // saved selected-and-hidden opens deselected. Save it so.
                    let disabled = file.int_field(&file.blocks[object], "Object", file.object_visibility_field(&dna), &dna) & OB_HIDE_VIEWPORT != 0;
                    if flag & BASE_HIDDEN != 0 || disabled {
                        flag &= !BASE_SELECTED;
                    }
                    set(&mut patched, file, base.base, "Base", "flag", i64::from(flag));
                }
                if self.active.as_deref() == Some(edit.name.as_str()) {
                    active_base = Some(base.base_address);
                }
            }
            if let Some(to) = &edit.rename {
                if file.set_id_name(object, to) {
                    patched += 1;
                }
            }
        }
        // The active object may be one the save just added.
        if active_base.is_none() {
            if let Some(address) = self.active.as_deref().and_then(|name| file.find_id("OB", name)).map(|i| file.blocks[i].old_address) {
                active_base = bases.iter().find(|b| b.object_address == address).map(|b| b.base_address);
            }
        }
        if let (Some(vl), Some(base)) = (view_layer, active_base) {
            if file.set_ptr(vl, "ViewLayer", "*basact", base) {
                patched += 1;
            }
        }

        if !self.collections.is_empty() {
            let at: Vec<BlendCollectionAt> = view_layer.map(|vl| file.collections_at(scene, vl)).unwrap_or_default();
            for edit in &self.collections {
                let Some(found) = at.iter().find(|c| c.path == edit.path) else {
                    report.warnings.push(format!("Collection '{}' is not in the file", edit.path));
                    continue;
                };
                if let (true, Some(lc)) = (edit.hidden.is_some() || edit.exclude.is_some(), found.layer_collection) {
                    let mut flag = file.int_field(&file.blocks[lc], "LayerCollection", "flag", &dna);
                    if let Some(hidden) = edit.hidden {
                        flag = with_bit(flag, LAYER_COLLECTION_HIDE, hidden);
                    }
                    if let Some(exclude) = edit.exclude {
                        flag = with_bit(flag, LAYER_COLLECTION_EXCLUDE, exclude);
                    }
                    set(&mut patched, file, lc, "LayerCollection", "flag", i64::from(flag));
                }
                if let Some(v) = edit.visibility {
                    let flag = file.int_field(&file.blocks[found.collection], "Collection", "flag", &dna);
                    let flag = with_bit(with_bit(with_bit(flag, COLLECTION_HIDE_VIEWPORT, v.hide_viewport), COLLECTION_HIDE_SELECT, v.hide_select), COLLECTION_HIDE_RENDER, v.hide_render);
                    set(&mut patched, file, found.collection, "Collection", "flag", i64::from(flag));
                }
            }
        }

        if let Some(markers) = &self.markers {
            file.set_markers(scene, markers);
            patched += 1;
        }
        // Last: a move adds and drops blocks, which moves the indices the
        // edits above hold.
        patched += self.relink(file, &mut report);
        patched += self.animate(file, &mut report);
        report.patched = patched;
        report
    }

    /// The Outliner's drops, once every other object edit is in: moves
    /// into a collection, then parents (`write/relink.rs`).
    fn relink(&self, file: &mut BlendFile, report: &mut BlendSaveReport) -> usize {
        let mut done = 0;
        for (name, path) in &self.moved {
            match file.find_id("OB", name) {
                Some(object) if file.move_object(object, path) => done += 1,
                Some(_) => report.warnings.push(format!("'{name}' not moved: collection '{path}' is not in the file")),
                None => report.warnings.push(format!("'{name}' is not in the file to move")),
            }
        }
        for (name, path) in &self.linked {
            match file.find_id("OB", name) {
                Some(object) if file.link_object_to(object, path) => done += 1,
                _ => report.warnings.push(format!("'{name}' not linked into '{path}'")),
            }
        }
        // Collections last: the objects above name them by the file's paths.
        for (child, parent) in &self.nested {
            if !file.nest_collection(child, parent) {
                report.warnings.push(format!("'{child}' not nested in '{parent}'"));
            }
        }
        if self.parented.is_empty() {
            return done;
        }
        // Where everything stands now, to keep a child there.
        let Ok(scene) = file.extract_scene() else { return done };
        let world = |name: &str| scene.objects.iter().find(|o| o.name == name).map(|o| (o.matrix_world, o.matrix_local));
        for (child, parent) in &self.parented {
            let (Some(c), Some(p), Some((child_world, child_local)), Some((parent_world, _))) = (file.find_id("OB", child), file.find_id("OB", parent), world(child), world(parent)) else {
                report.warnings.push(format!("'{child}' not parented to '{parent}': not both in the file"));
                continue;
            };
            if file.parent_object(c, p, child_world, child_local, parent_world) {
                done += 1;
            }
        }
        done
    }

    /// The structural edits — objects removed, added and duplicated —
    /// which move blocks about, so they go first. Returns the count of
    /// objects written.
    fn restructure(&self, file: &mut BlendFile, report: &mut BlendSaveReport) -> usize {
        let mut done = 0;
        for name in &self.deleted {
            let (Some(scene), Some(object)) = (file.current_scene_index(), file.find_id("OB", name)) else {
                report.warnings.push(format!("'{name}' is not in the file to delete"));
                continue;
            };
            let Some(view_layer) = file.view_layer_index(scene) else { continue };
            file.remove_object(scene, view_layer, object);
            done += 1;
        }
        if !self.added.is_empty() && !file.writes_meshes() {
            report.warnings.push(format!(
                "{} added object(s) not written: a file saved by Blender {} is older than 3.6, whose mesh layout the writer does not produce",
                self.added.len(),
                file.header.version
            ));
        }
        for new in &self.added {
            if !file.writes_meshes() {
                break;
            }
            let (Some(me_template), Some(ob_template)) = (file.mesh_template(), file.mesh_object_template()) else {
                report.warnings.push(format!("'{}' not written: the file has no mesh object to model it on", new.name));
                continue;
            };
            let mut mesh = new.mesh.clone();
            if mesh.name.is_empty() {
                mesh.name = new.name.clone();
            }
            let Some((_, mesh_address)) = file.add_mesh(me_template, &mesh) else { continue };
            let Some((object, object_address)) = file.add_object(ob_template, &new.name, mesh_address, new.location, new.scale) else { continue };
            if let Some(v) = new.visibility {
                let field = file.object_visibility_field(&file.dna.clone().unwrap());
                let flag = with_bit(with_bit(with_bit(0, OB_HIDE_VIEWPORT, v.hide_viewport), OB_HIDE_SELECT, v.hide_select), OB_HIDE_RENDER, v.hide_render);
                file.set_int(object, "Object", field, i64::from(flag));
            }
            link_new(file, &new.collection, object_address, new.selected, report);
            done += 1;
        }
        for dup in &self.duplicated {
            let Some(source) = file.find_id("OB", &dup.source) else {
                report.warnings.push(format!("'{}' not duplicated: '{}' is not in the file", dup.copy, dup.source));
                continue;
            };
            let dna1 = file.blocks.iter().position(|b| &b.code == DNA1).unwrap_or(file.blocks.len());
            let Some(dna) = file.dna.clone() else { continue };
            // Shift D puts the copy in the source's collection
            // (`BKE_collection_object_add_from`), not the scene's.
            let source_address = file.blocks[source].old_address;
            let home = file
                .current_scene_index()
                .and_then(|s| file.view_layer_index(s).map(|vl| file.collections_holding(s, vl, source_address)))
                .and_then(|paths| paths.into_iter().next())
                .unwrap_or_default();
            let mesh = file.ptr(&file.blocks[source], file.off("Object", "*data", &dna));
            let Some((object, object_address)) = file.clone_block_cleared(source, dna1.saturating_sub(1)) else { continue };
            file.fresh_object(object, &dup.copy);
            file.set_ptr(object, "Object", "*data", mesh);
            if let Some(source) = file.find_id("OB", &dup.source) {
                file.copy_material_slots(source, object);
            }
            link_new(file, &home, object_address, dup.selected, report);
            done += 1;
        }
        done
    }
}

/// Put a new object into the collection at `path` (the scene collection
/// when empty or not found) and the view layer.
fn link_new(file: &mut BlendFile, path: &str, object_address: u64, selected: bool, report: &mut BlendSaveReport) {
    let (Some(scene), Some(view_layer)) = (file.current_scene_index(), file.current_scene_index().and_then(|s| file.view_layer_index(s))) else { return };
    let at = file.collections_at(scene, view_layer);
    let Some(target) = at.iter().find(|c| c.path == path).or_else(|| at.first()).map(|c| c.collection) else {
        report.warnings.push("The file has no collection to put a new object in".to_string());
        return;
    };
    file.link_object(scene, view_layer, target, object_address, if selected { i64::from(BASE_SELECTED) } else { 0 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_edits_land_in_the_file_and_read_back_through_the_scene_reader() {
        let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
        let mut file = BlendFile::parse_bytes(&std::fs::read(&path).expect("fixture")).expect("parse");
        let edits = BlendEdits {
            frame: Some(42),
            frame_start: Some(10),
            frame_end: Some(120),
            objects: vec![BlendObjectEdit {
                name: "Cube".into(),
                rename: Some("Crate".into()),
                visibility: Some(BlendVisibility { hide_viewport: false, hide_select: true, hide_render: true }),
                hidden: Some(true),
                selected: Some(true),
                transform: None,
                shadow: Some(BlendShadowCast { display: false, ray: false }),
            }],
            active: Some("Cube".into()),
            markers: Some(vec![(12, "Intro".into()), (100, "Cut".into())]),
            collections: vec![BlendCollectionEdit { path: "Scene Collection".into(), hidden: None, exclude: None, visibility: Some(BlendVisibility { hide_render: true, ..Default::default() }) }],
            unwritable: vec!["1 added object".into()],
            border: Some(crate::blend::BlendRenderBorder { on: true, crop: true, rect: [0.25, 0.75, 0.1, 0.9] }),
            ..Default::default()
        };
        let report = edits.apply(&mut file);
        assert!(report.patched >= 7, "{report:?}");
        assert_eq!(report.warnings, ["1 added object"]);

        let again = BlendFile::parse_bytes(&file.to_bytes()).expect("the written file parses");
        let scene = again.extract_scene().expect("scene");
        assert_eq!((scene.props.frame_current, scene.props.frame_start, scene.props.frame_end), (42, 10, 120));
        let crate_ = scene.objects.iter().find(|o| o.name == "Crate").expect("renamed");
        assert!(!scene.objects.iter().any(|o| o.name == "Cube"));
        assert!(crate_.visibility.hide_select && crate_.visibility.hide_render && !crate_.visibility.hide_viewport);
        assert_eq!(crate_.shadow, BlendShadowCast { display: false, ray: false }, "both Shadow switches, `dtx` and the visibility flag");
        let index = scene.objects.iter().position(|o| o.name == "Crate").unwrap();
        // Hidden, so saved deselected: Blender drops the selection of what it
        // cannot see (`BKE_base_eval_flags`), and the file says so.
        assert!(scene.hidden.contains(&index) && !scene.selected.contains(&index), "the base flags");
        assert_eq!(scene.active_object, Some(index), "the active object");
        assert_eq!(scene.props.markers.iter().map(|m| (m.frame, m.name.as_str())).collect::<Vec<_>>(), [(12, "Intro"), (100, "Cut")]);
        assert!(scene.collections[0].visibility.hide_render, "the collection's camera");
        assert_eq!(scene.props.border, crate::blend::BlendRenderBorder { on: true, crop: true, rect: [0.25, 0.75, 0.1, 0.9] }, "the render region and its two mode bits");
    }
}

