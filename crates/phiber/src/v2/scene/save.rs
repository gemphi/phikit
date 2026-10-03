//! Saving — `wm.save_mainfile` / `wm.save_as_mainfile`: the opened `.blend`
//! written back with the session's edits through it (`phiphx::blend::write`).
//!
//! What goes in is what the state holds over the file: the frame and the
//! range, every object's toggles, eye, selection, name and transform, the
//! active object, the collections' eye and toggles, the marker list once
//! it was edited, the Outliner's moves, links, parents and nestings, the
//! actions made and assigned, the Render and Output tabs (`render_edit.rs`),
//! and the objects added (a primitive's mesh, welded and paired into
//! quads), duplicated (a linked copy) or deleted. What cannot go in — a new
//! mesh into a file older than Blender 3.6 — is said in the report, as
//! Blender's `Warning:` lines are, so a save never quietly drops an edit.
//! An existing file becomes `file.blend1` first (`use_save_versions`); a
//! file the session did not open — the default scene, a new file — has
//! nothing to write through, and says so.

use crate::phiphx::blend::write::{BlendCollectionEdit, BlendCompression, BlendDuplicate, BlendEdits, BlendNewMesh, BlendNewObject, BlendObjectEdit, BlendSaveReport};
use crate::phiphx::blend::{BlendFile, BlendVisibility};
use crate::phiphx::scene::{SceneCollection, SceneModel};
use crate::v2::part::outliner::key::{is_hidden, is_render_hidden, is_unselectable, k_restrict};
use crate::v2::part::outliner::layer::collection_id;
use crate::v2::state::key::int_or;
use crate::v2::state::{BState, CtxKey, val::V_TRUE};
use std::path::{Path, PathBuf};

/// The file the document was read from: what a save writes through.
pub fn source_path(state: &BState, model: &SceneModel) -> Option<PathBuf> {
    let loaded = state.get(&super::loaded_path_key()).filter(|p| !p.is_empty()).map(PathBuf::from);
    loaded.or_else(|| model.source_path.as_deref().map(PathBuf::from)).filter(|p| p.is_file())
}

/// The name an object has now, given the name the file has for it.
fn current_name(edit: &super::edit::BSceneEdit, file_name: &str) -> String {
    edit.renamed.iter().find(|(_, old)| old == file_name).map(|(new, _)| new.clone()).unwrap_or_else(|| file_name.to_string())
}

fn toggles(state: &BState, name: &str) -> BlendVisibility {
    BlendVisibility {
        hide_viewport: state.get(&k_restrict(name, "hide_viewport")) == Some(V_TRUE),
        hide_select: is_unselectable(state, name),
        hide_render: is_render_hidden(state, name),
    }
}

fn collections(state: &BState, c: &SceneCollection, path: &str, out: &mut Vec<BlendCollectionEdit>) {
    let id = collection_id(path);
    out.push(BlendCollectionEdit {
        path: path.to_string(),
        hidden: Some(state.get(&k_restrict(id, "hide_get")) == Some(V_TRUE)),
        exclude: Some(crate::v2::part::outliner::key::is_excluded(state, id)),
        visibility: Some(toggles(state, id)),
    });
    for k in &c.children {
        collections(state, k, &format!("{path}/{}", k.name), out);
    }
}

/// Everything the state holds over the file, as the writer takes it.
pub fn gather(state: &BState, model: &SceneModel) -> BlendEdits {
    let edit = super::edit::BSceneEdit::now(state);
    let selection = super::select::BSelection::now(state);
    let active = state.get(CtxKey::Object).unwrap_or("").to_string();
    let file_names: Vec<String> = model
        .objects
        .iter()
        .map(|o| o.name.clone())
        .chain(model.cameras.iter().map(|c| c.name.clone()))
        .chain(model.lights.iter().map(|l| l.name.clone()))
        .chain(model.empties.iter().map(|e| e.name.clone()))
        .collect();
    let mut active_file_name = None;
    let transforms = super::transform::BTransforms::now(state);
    let mut objects: Vec<BlendObjectEdit> = file_names
        .iter()
        .filter(|name| !edit.deleted.contains(name))
        .map(|name| {
            let now = current_name(&edit, name);
            if now == active {
                active_file_name = Some(name.clone());
            }
            BlendObjectEdit {
                name: name.clone(),
                rename: (now != *name).then_some(now.clone()),
                visibility: Some(toggles(state, &now)),
                hidden: Some(is_hidden(state, &now) && state.get(&k_restrict(&now, "hide_viewport")) != Some(V_TRUE)),
                selected: Some(selection.contains(&now)),
                transform: transforms.get(&now).map(|t| t.to_blend()),
            }
        })
        .collect();
    objects.extend(transforms.session_edits(&edit));
    let mut collection_edits = Vec::new();
    for c in &model.collections {
        collections(state, c, &c.name, &mut collection_edits);
    }
    // Added primitives: their mesh as the viewport draws it. A duplicate of
    // a file object is a linked copy; of an added primitive, it again.
    let mut added: Vec<BlendNewObject> = Vec::new();
    let mut duplicated: Vec<BlendDuplicate> = Vec::new();
    let new_object = |name: &str, shape: &str| -> Option<BlendNewObject> {
        let mesh = super::edit::primitive_mesh(shape)?;
        Some(BlendNewObject {
            name: name.to_string(),
            // Welded and paired into quads: a cube saves as Blender's eight
            // vertices and six faces, not the viewport's twenty-four and twelve.
            mesh: BlendNewMesh::from_triangles(name, &mesh.vertices, &mesh.indices),
            location: [0.0; 3],
            scale: [1.0; 3],
            collection: String::new(),
            selected: selection.contains(name),
            visibility: Some(toggles(state, name)),
        })
    };
    for (name, shape) in &edit.added {
        if let Some(object) = new_object(name, shape) {
            added.push(object);
        }
    }
    for (copy, source) in &edit.copied {
        match edit.added.iter().find(|(name, _)| name == source) {
            Some((_, shape)) => {
                if let Some(object) = new_object(copy, shape) {
                    added.push(object);
                }
            }
            None => duplicated.push(BlendDuplicate { copy: copy.clone(), source: source.clone(), selected: selection.contains(copy) }),
        }
    }
    let edits = BlendEdits {
        added,
        duplicated,
        deleted: edit.deleted.clone(),
        frame: Some(int_or(state, CtxKey::Frame, model.timeline.current_frame)),
        frame_start: Some(int_or(state, CtxKey::FrameStart, model.timeline.frame_start)),
        frame_end: Some(int_or(state, CtxKey::FrameEnd, model.timeline.frame_end)),
        cursor: super::cursor::BCursor::edited(state),
        resolution: super::render_set::BResolution::now(state).edited(model),
        view: super::render_set::BColorManagement::now(state, Some(model)).edited(model),
        objects,
        active: active_file_name,
        markers: state.get(CtxKey::Markers).map(|_| super::marker::BMarkerList::now(state).list),
        playback: Some(crate::v2::part::playback::settings::now(state)),
        collections: collection_edits,
        ..super::render_edit::BRenderEdits::of(state, model).over(super::links::BSceneLinks::now(state).edits())
    };
    super::action::BSceneActions::now(state).edits(model, edits)
}

/// Write the document to `target`: the source file with the edits through
/// it, the existing `target` kept as `target1`.
pub fn save(state: &BState, model: &SceneModel, target: &Path) -> Result<BlendSaveReport, String> {
    let source = source_path(state, model).ok_or_else(|| "Nothing to save through: this scene was not opened from a .blend (writing a new file is not available yet)".to_string())?;
    let bytes = std::fs::read(&source).map_err(|e| format!("Cannot read '{}': {e}", source.display()))?;
    let compression = BlendCompression::of(&bytes);
    let mut file = BlendFile::parse_bytes(&bytes).map_err(|e| format!("Cannot read '{}': {e}", source.display()))?;
    let report = gather(state, model).apply(&mut file);
    file.write_to(target, compression).map_err(|e| format!("Cannot write '{}': {e}", target.display()))?;
    Ok(report)
}

/// `wm.save_mainfile(incremental=True)`: the next numbered name beside
/// `path` that no file has — `cube.blend` → `cube1.blend` → `cube2.blend`,
/// a trailing number kept to its digits (`shot_010.blend` → `shot_011.blend`).
pub fn incremental_path(path: &Path) -> PathBuf {
    let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let digits = stem.chars().rev().take_while(char::is_ascii_digit).count();
    let (head, number) = stem.split_at(stem.len() - digits);
    let mut n: u64 = number.parse().unwrap_or(0);
    let width = digits.max(1);
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    loop {
        n += 1;
        let candidate = folder.join(format!("{head}{n:0width$}{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_next_numbered_name_keeps_the_digits_and_skips_what_exists() {
        let dir = std::env::temp_dir().join(format!("phiber_incr_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(incremental_path(&dir.join("cube.blend")), dir.join("cube1.blend"));
        assert_eq!(incremental_path(&dir.join("shot_010.blend")), dir.join("shot_011.blend"));
        std::fs::write(dir.join("cube1.blend"), b"x").unwrap();
        assert_eq!(incremental_path(&dir.join("cube.blend")), dir.join("cube2.blend"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_gathered_edits_name_the_file_objects_and_report_what_cannot_be_written() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures/nla_cube.blend");
        let bytes = std::fs::read(path).expect("fixture");
        let blend = BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
        let model = SceneModel::from_blend_owned(blend, path);
        let mut state = BState::new();
        state.set(CtxKey::Frame, "42");
        state.set(CtxKey::Object, "Crate");
        let mut edit = super::super::edit::BSceneEdit::now(&state);
        edit.rename("Cube", "Crate");
        edit.added.push(("Suzanne".into(), "monkey".into()));
        edit.write(&mut state);
        state.set(&k_restrict("Crate", "hide_render"), V_TRUE);
        state.set(CtxKey::Markers, "10=Go");
        let edits = gather(&state, &model);
        assert_eq!(edits.frame, Some(42));
        let cube = edits.objects.iter().find(|o| o.name == "Cube").expect("keyed by the file's name");
        assert_eq!(cube.rename.as_deref(), Some("Crate"));
        assert!(cube.visibility.unwrap().hide_render, "the toggles are read under the current name");
        assert_eq!(edits.active.as_deref(), Some("Cube"), "the active object, by its file name");
        assert_eq!(edits.markers, Some(vec![(10, "Go".to_string())]));
        assert!(edits.unwritable.is_empty());
        assert_eq!(edits.added.len(), 1, "the added monkey goes in");
        assert!(edits.added[0].mesh.faces.iter().any(|f| f.len() == 4) && edits.added[0].mesh.faces.len() > 100, "the monkey's faces paired into quads where they are flat");
        assert!(edits.collections.iter().any(|c| c.path == "Scene Collection"));

        // And it saves: the written file re-reads with the edits, the original untouched.
        let dir = std::env::temp_dir().join(format!("phiber_save_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("out.blend");
        let report = save(&state, &model, &target).expect("saved");
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        let again = BlendFile::parse_bytes(&std::fs::read(&target).unwrap()).unwrap().extract_scene().unwrap();
        assert!(again.objects.iter().any(|o| o.name == "Crate"));
        let monkey = again.objects.iter().find(|o| o.name == "Suzanne").expect("the added monkey is in the file");
        assert!(monkey.vertices.len() > 100 && monkey.faces.len() > 100, "with its mesh");
        assert_eq!(again.props.frame_current, 42);
        // Saving again keeps the first as `out.blend1`.
        save(&state, &model, &target).expect("saved again");
        assert!(dir.join("out.blend1").is_file());
        assert_eq!(std::fs::read(path).unwrap(), bytes, "the fixture is untouched");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
