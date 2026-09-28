//! The file-browser commits the menus wait on: `File ▸ Save As` and `Save
//! Copy` write the `.blend` back with the session's edits (`scene::save`),
//! `File ▸ Export` writes the scene in the chosen format, `File ▸ Import`
//! reads a file's meshes into it, Link and Append report themselves until
//! they can copy from another `.blend`.

use super::super::BPhiberApp;
use crate::v2::part::file::purpose::BFilePurpose;
use crate::v2::part::text::info::{self, BReportLevel};
use crate::v2::state::{BKey as _, CtxKey, FileKey, OpKey, log};

impl BPhiberApp {
    /// A file-browser commit that exports or imports, once the browser has
    /// its path. `purpose` is what the browser was opened for.
    pub fn handle_file_commit(&mut self, path: &str, purpose: BFilePurpose) {
        match purpose {
            BFilePurpose::Export => {
                let format = self.state.get(OpKey::ExportFormat).unwrap_or("obj").to_string();
                let hidden = crate::v2::part::outliner::key::BHidden::now(&self.state);
                let objects: Vec<_> = crate::v2::scene::scene_objects(&self.state)
                    .iter()
                    .filter(|o| !hidden.hides(&self.state, o.name()))
                    .cloned()
                    .collect();
                match crate::v2::scene::export::write(std::path::Path::new(path), &format, &objects) {
                    Ok(file) => {
                        let text = format!("Exported {} object(s) to '{}'", objects.len(), file.display());
                        log::push(text.clone());
                        info::push(&mut self.state, BReportLevel::Info, text);
                    }
                    Err(why) => {
                        log::push(format!("Cannot export: {why}"));
                        info::push(&mut self.state, BReportLevel::Error, format!("Cannot export: {why}"));
                    }
                }
                self.state.remove(OpKey::ExportFormat);
                self.state.remove(&FileKey::Purpose.path());
            }
            BFilePurpose::Import => {
                self.state.remove(&FileKey::Purpose.path());
                if !crate::v2::scene::import::readable(path) {
                    let text = format!("Cannot import '{}': not a format phiber reads yet", file_name(path));
                    log::push(text.clone());
                    info::push(&mut self.state, BReportLevel::Error, text);
                    return;
                }
                match crate::v2::scene::import::read(path) {
                    Ok(objects) => self.merge(objects, path),
                    Err(why) => {
                        log::push(format!("Cannot import: {why}"));
                        info::push(&mut self.state, BReportLevel::Error, format!("Cannot import: {why}"));
                    }
                }
            }
            BFilePurpose::SaveAs | BFilePurpose::SaveCopy => {
                self.state.remove(&FileKey::Purpose.path());
                let target = with_blend_extension(path);
                if self.save_to(&target) && purpose == BFilePurpose::SaveAs {
                    // `wm.save_as_mainfile`: the document is the new file now,
                    // and it is saved; a copy leaves the document as it was.
                    self.state.set(&crate::v2::scene::loaded_path_key(), target.to_string_lossy().to_string());
                    super::close::tag_saved(&mut self.state);
                }
            }
            BFilePurpose::Link | BFilePurpose::Append => {
                self.state.remove(&FileKey::Purpose.path());
                let text = format!("{}: not available yet", purpose.title());
                log::push(text.clone());
                info::push(&mut self.state, BReportLevel::Warning, text);
            }
            _ => {}
        }
    }

    /// Ctrl S / Ctrl Alt S: the document written to its own path, or to the
    /// next numbered one; an untitled document (a new file) asks where.
    pub(crate) fn save_document(&mut self, incremental: bool) {
        let Some(model) = self.scene_cache.as_ref().map(|(_, m)| std::sync::Arc::clone(m)) else {
            self.report_no_source();
            return;
        };
        if self.state.get(&crate::v2::scene::loaded_path_key()).filter(|p| !p.is_empty()).is_none() {
            // `wm.save_mainfile` on an untitled file is `wm.save_as_mainfile`.
            crate::v2::state::action::apply_action(&mut self.state, &crate::v2::view::prim::BAction::Set(FileKey::Purpose.path(), BFilePurpose::SaveAs.slug().to_string()));
            return;
        }
        let Some(source) = crate::v2::scene::save::source_path(&self.state, &model) else {
            self.report_no_source();
            return;
        };
        let target = if incremental { crate::v2::scene::save::incremental_path(&source) } else { source };
        if self.save_to(&target) {
            if incremental {
                self.state.set(&crate::v2::scene::loaded_path_key(), target.to_string_lossy().to_string());
            }
            super::close::tag_saved(&mut self.state);
        }
    }

    fn report_no_source(&mut self) {
        let text = "Nothing to save through: open a .blend first (writing a new file is not available yet)".to_string();
        log::push(text.clone());
        info::push(&mut self.state, BReportLevel::Warning, text);
    }

    /// The save itself, reported as Blender reports it: `Saved "file.blend"`
    /// in the status bar and the Info editor, a `Warning:` line per edit the
    /// writer had to leave out, an error when it could not write.
    fn save_to(&mut self, target: &std::path::Path) -> bool {
        let Some(model) = self.scene_cache.as_ref().map(|(_, m)| std::sync::Arc::clone(m)) else {
            self.report_no_source();
            return false;
        };
        match crate::v2::scene::save::save(&self.state, &model, target) {
            Ok(report) => {
                let name = target.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                let text = format!("Saved \"{name}\"");
                log::push(text.clone());
                info::push(&mut self.state, BReportLevel::Info, text);
                for warning in &report.warnings {
                    info::push(&mut self.state, BReportLevel::Warning, warning.clone());
                }
                true
            }
            Err(why) => {
                log::push(why.clone());
                info::push(&mut self.state, BReportLevel::Error, why);
                false
            }
        }
    }

    /// Imported objects added to the scene, selected, the last one active.
    fn merge(&mut self, objects: Vec<crate::phiphx::scene::SceneObject>, path: &str) {
        let names: Vec<String> = objects.iter().map(|o| o.name.clone()).collect();
        let base = match &self.scene_cache {
            Some((_, model)) => (**model).clone(),
            None => crate::phiphx::scene::SceneModel::default(),
        };
        let key = self.scene_cache.as_ref().map(|(k, _)| k.clone()).unwrap_or_else(|| path.to_string());
        let merged = std::sync::Arc::new(crate::v2::scene::import::append(&base, objects));
        crate::v2::scene::source::set_loaded_shared(Some(std::sync::Arc::clone(&merged)));
        crate::v2::scene::source::clear_scene_objects();
        self.scene_cache = Some((key, merged));
        if let Some(last) = names.last() {
            self.state.set(CtxKey::Object, last.clone());
        }
        crate::v2::scene::select::BSelection { list: names.clone() }.write(&mut self.state);
        let text = format!("Imported {} object(s) from '{}'", names.len(), file_name(path));
        log::push(text.clone());
        info::push(&mut self.state, BReportLevel::Info, text);
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// `wm.save_as_mainfile` adds `.blend` when the name typed has no extension.
fn with_blend_extension(path: &str) -> std::path::PathBuf {
    let path = std::path::PathBuf::from(path);
    match path.extension() {
        Some(_) => path,
        None => path.with_extension("blend"),
    }
}
