//! `BlendEditList::apply`: the session's edits written through the file.
use super::*;

impl BlendEditList {
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
    }}
