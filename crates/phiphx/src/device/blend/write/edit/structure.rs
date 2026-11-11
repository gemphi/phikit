//! The edits that change structure: the Outliner's drops, added and deleted objects.
use super::*;

impl BlendEditList {
    /// The Outliner's drops, once every other object edit is in: moves
    /// into a collection, then parents (`write/relink.rs`).
    pub(super) fn relink(&self, file: &mut BlendFile, report: &mut BlendSaveReport) -> usize {
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
    pub(super) fn restructure(&self, file: &mut BlendFile, report: &mut BlendSaveReport) -> usize {
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
pub(super) fn link_new(file: &mut BlendFile, path: &str, object_address: u64, selected: bool, report: &mut BlendSaveReport) {
    let (Some(scene), Some(view_layer)) = (file.current_scene_index(), file.current_scene_index().and_then(|s| file.view_layer_index(s))) else { return };
    let at = file.collections_at(scene, view_layer);
    let Some(target) = at.iter().find(|c| c.path == path).or_else(|| at.first()).map(|c| c.collection) else {
        report.warnings.push("The file has no collection to put a new object in".to_string());
        return;
    };
    file.link_object(scene, view_layer, target, object_address, if selected { i64::from(BASE_SELECTED) } else { 0 });
}
