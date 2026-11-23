//! Phigen's parts: what each slot of its window shows. `title`, `talk`
//! and `ask` are the chat host's, `find` the search's
//! (`data/region.tsv`), `hello` and `card` the start's, and `thread` the
//! history's, the host in the bar. `roster` and `setting` are what two
//! modals show, and `pick` is the pill in the title that opens the first.
//! Each is a view; appkit gives it its rectangle and sends it what a hand
//! does.

mod ask;
mod card;
mod find;
mod hello;
mod pick;
mod roster;
mod setting;
mod talk;
mod thread;
mod title;

pub use ask::Ask;
pub use card::Card;
pub use find::Find;
pub use hello::Hello;
pub use pick::Pick;
pub use roster::Roster;
pub use setting::Setting;
pub use talk::Talk;
pub use thread::Thread;
pub use title::Title;

use std::cell::RefCell;
use std::rc::Rc;

use appkit::pui::core::PPaint;
use appkit::pui::window::{PDeed, PWish};
use appkit::service::PTable;
use appkit::theme::{PLook, PShade, PTheme};

use crate::choice::Choice;
use crate::history::History;

/// The theme's tokens more than one part is drawn with.
pub const GAP: &str = "gap";
pub const INK: &str = "text.ink";
pub const DIM: &str = "text.dim";
pub const LINE: &str = "text.line";
/// How wide the column down the middle of the body is, which every part
/// there keeps to: so the field, the cards, the turns and the Search
/// window share their edges.
pub const COLUMN: &str = "column.width";
/// The word that ends a text cut short.
pub const CUT: &str = "text.cut";

/// What every part works on: the chats, the words a person reads, and the
/// look, which a part asks for its measures when it is given a rectangle.
#[derive(Clone)]
pub struct Desk {
    pub history: Rc<RefCell<History>>,
    pub text: Rc<PTable>,
    pub look: Rc<PLook>,
    /// The looks Phigen can be shown in, and the one it is shown in: what
    /// everything is drawn with.
    pub shade: Rc<PShade>,
    /// What the person chose, which is kept.
    pub choice: Rc<Choice>,
    /// Where a part asks the window for a deed.
    pub wish: PWish,
    /// The space a chat is shown in: what a part wishes shown when a chat
    /// is opened from somewhere else.
    pub home: String,
    /// The host a chat is shown in before anything was said in it, and
    /// the one it is shown in after.
    pub empty: String,
    pub full: String,
}

impl Desk {
    /// A size of the look; nothing for a token it does not have.
    pub fn size(&self, token: &str) -> f32 {
        self.look.size(token).unwrap_or_default()
    }

    /// The column down the middle of a part that wide: its left edge and
    /// its width, in the part's own coordinates. It is as wide as the
    /// theme says, and never nearer the part's sides than a gap.
    pub fn column(&self, width: f32) -> (f32, f32) {
        let wide = self.look.size(COLUMN).unwrap_or(f32::MAX).min(width - self.size(GAP) * 2.0).max(0.0);
        ((width - wide) / 2.0, wide)
    }

    /// Ask the window for the chat's space, with the host the open chat
    /// is shown in: the one for a chat nothing was said in, or the other.
    fn show(&self) {
        let used = self.history.borrow().current().title().is_some();
        let hold = match used {
            true => PDeed::Hold(self.empty.clone(), self.full.clone()),
            false => PDeed::Hold(self.full.clone(), self.empty.clone()),
        };
        self.wish.borrow_mut().extend([PDeed::Show(self.home.clone()), hold]);
    }

    /// The person says something in the open chat. `false` when it is
    /// blank or an answer is still being written.
    pub fn say(&self, text: &str) -> bool {
        let said = self.history.borrow_mut().say(text);
        if said {
            self.show();
        }
        said
    }

    /// Open the chat of that number. `false` when there is none.
    pub fn open(&self, index: usize) -> bool {
        let opened = self.history.borrow_mut().open(index);
        if opened {
            self.show();
        }
        opened
    }

    /// Show Phigen in the look of that name from now on, and keep that.
    /// `false` when there is no such look.
    pub fn dress(&self, name: &str) -> bool {
        let turned = self.shade.turn(name);
        if turned {
            self.choice.put("look", name);
        }
        turned
    }

    /// Have the model of that name answer from now on, and keep that.
    /// `false` where there is no model to choose.
    pub fn choose(&self, name: &str) -> bool {
        let chosen = self.history.borrow_mut().choose(name);
        if chosen {
            self.choice.put("model", name);
        }
        chosen
    }

    /// Begin a chat.
    pub fn begin(&self) {
        self.history.borrow_mut().begin();
        self.show();
    }
}

/// `text` when it is no wider than `width`; else as much of its start
/// as fits with the theme's mark for a text cut short after it.
pub fn fit(text: &str, width: f32, paint: &dyn PPaint) -> String {
    if paint.measure(text) <= width {
        return text.to_string();
    }
    let mark = paint.theme().word(CUT).unwrap_or_default().to_string();
    let mut out = String::new();
    for letter in text.chars() {
        out.push(letter);
        if paint.measure(&format!("{out}{mark}")) > width {
            out.pop();
            break;
        }
    }
    format!("{}{mark}", out.trim_end())
}
