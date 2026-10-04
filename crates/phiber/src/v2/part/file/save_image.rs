//! `image.save_as` — the Image editor's Save / Save As… / Save a Copy…
//! commit: the shown image written where the browser was pointed. The
//! Render Result goes out in Properties → Output's format, as Blender's
//! `image.save_as` reads `scene->r.im_format` for it — the extension
//! follows the format, not the other way round — with its passes under a
//! multi-layer format; one of the session's images (`scene::image_list`)
//! in the format its name's extension says. Reported in the status bar and
//! the Info editor either way, since a silent write leaves no way to tell
//! a finished save from a refused one.

use crate::v2::part::image::shown::BShownImage;
use crate::v2::part::text::info::{BReportLevel, push};
use crate::v2::state::BState;
use std::path::Path;

pub struct BImageSave;

impl BImageSave {
    pub fn to(s: &mut BState, target: &str) {
        if let BShownImage::Named(name) = BShownImage::now(s) {
            let Some(image) = crate::v2::scene::image_list::BImageList::get(&name) else {
                push(s, BReportLevel::Error, "No image to save.");
                return;
            };
            let file = Path::new(target);
            let file = if file.extension().is_some() { file.to_path_buf() } else { file.with_extension("png") };
            match image.save(&file) {
                Ok(()) => push(s, BReportLevel::Info, &format!("Saved: '{}'", file.display())),
                Err(why) => push(s, BReportLevel::Error, &format!("Cannot save: {why}")),
            }
            return;
        }
        let (shot, _) = crate::v2::scene::render_job::shot();
        let Some(shot) = shot else {
            push(s, BReportLevel::Error, "No render to save. Render an image first (F12).");
            return;
        };
        // A render still in flight is refused by the writer itself, with its
        // own message — the guard is not repeated here, so there is one place
        // that decides whether a shot can be written.
        let output = crate::v2::scene::output_set(s);
        let pass = crate::v2::scene::pass_set(s, Default::default());
        let file = output.file_of(Path::new(target).parent().unwrap_or(Path::new("")));
        // `file_of` is given the typed path's folder and the panel's own name,
        // so a name typed in the browser wins over the panel's while the
        // format still decides the extension.
        let file = match Path::new(target).file_name() {
            Some(name) => file.with_file_name(name).with_extension(output.format.extension().trim_start_matches('.')),
            None => file,
        };
        let written = crate::v2::scene::written_pass(&shot, pass);
        match crate::v2::scene::save_render_to(&shot, &output, pass, &file) {
            Ok(()) => {
                let note = if written.len() > 1 { format!(" ({} passes)", written.len()) } else { String::new() };
                push(s, BReportLevel::Info, &format!("Saved: '{}'{note}", file.display()));
            }
            Err(why) => push(s, BReportLevel::Error, &format!("Cannot save: {why}")),
        }
    }
}
