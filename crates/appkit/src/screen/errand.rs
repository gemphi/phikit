//! Errands: what a part asked for that only the system can do. A part
//! wishes for one as it wishes for any deed; the stage, which cannot do
//! it, keeps it, and the screen does it here after the event that asked.
//!
//! * `Copy` puts a text where the system keeps what was copied.
//! * `Launch` opens a file in the system's own program for it.
//! * `Save` asks the person where a copy of a file is to be kept, in
//!   the system's own dialog, which opens in their own folder with the
//!   file's name; a copy is kept where they say, and none when they
//!   close the dialog. A system that hides a file's ending in its
//!   dialog gives back a name without one: the copy is then given the
//!   file's own ending, unless something stands there under that name.
//! * `Choose` asks the person for a file, in the system's own dialog,
//!   and tells the app its word with the file's path after it.

use std::path::{Path, PathBuf};

use gpui::{ClipboardItem, Context, PathPromptOptions};

use super::{PScreen, warn};
use crate::pui::window::PDeed;

/// Where a copy is kept: where the person said, with the file's own
/// ending when they gave none and nothing has that name already.
pub fn place(from: &Path, to: PathBuf) -> PathBuf {
    match (to.extension(), from.extension()) {
        (None, Some(ending)) => Some(to.with_extension(ending)).filter(|with| !with.exists()).unwrap_or(to),
        _ => to,
    }
}

/// Do one errand. A deed that is none is not the screen's, and is left.
pub fn run(deed: PDeed, cx: &mut Context<PScreen>) {
    match deed {
        PDeed::Copy(text) => cx.write_to_clipboard(ClipboardItem::new_string(text)),
        PDeed::Launch(path) => cx.open_with_system(Path::new(&path)),
        PDeed::Save(path) => {
            let from = PathBuf::from(path);
            let name = from.file_name().map(|name| name.to_string_lossy().to_string());
            let folder = std::env::home_dir().unwrap_or_default();
            let asked = cx.prompt_for_new_path(&folder, name.as_deref());
            let kept = async move {
                // A dialog that was closed, or could not open, keeps nothing.
                let Ok(Ok(Some(to))) = asked.await else { return };
                let to = place(&from, to);
                let copied = std::fs::copy(&from, &to);
                copied.err().into_iter().for_each(|error| warn(&format!("a copy could not be kept at {}: {error}", to.display())));
            };
            cx.background_executor().spawn(kept).detach();
        }
        PDeed::Choose(word) => {
            let asked = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: false, prompt: None });
            let told = cx.spawn(async move |screen, cx| {
                // A dialog that was closed tells nothing.
                let Ok(Ok(Some(path))) = asked.await else { return };
                let Some(path) = path.first() else { return };
                let word = format!("{word} {}", path.display());
                _ = screen.update(cx, |screen, cx| {
                    screen.stage.tell(&word);
                    cx.notify();
                });
            });
            told.detach();
        }
        _ => {}
    }
}
