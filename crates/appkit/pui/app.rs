//! An app: what it supplies, and the window appkit makes of it.
//!
//! An app builds no window structure of its own. It names itself, hands
//! over four sets (its spaces, its hosts, its parts and its modals), its
//! theme, the rows of its activity bar and its user, and gives each part
//! its content. All but the content is data. appkit opens the window. Phiber is one
//! app, Phiace another, Phigen the first, and all get their window here.

use std::rc::Rc;

use crate::pui::core::PView;
use crate::pui::window::{PActivity, PFact, PHost, PModal, PPart, PRail, PSet, PSpace, PStage, PWindow, PWish};
use crate::service::PTable;
use crate::theme::{PShade, PTheme};

/// What an app supplies. `open` and `missing` are appkit's and the same for
/// every app.
pub trait PApp {
    /// What the window is titled.
    fn name(&self) -> &str;

    /// Its spaces, in the order of their tabs, each with its zones and the
    /// hosts its areas hold.
    fn space(&self) -> &PSet<PSpace>;

    /// Its hosts, each with the regions it declares.
    fn host(&self) -> &PSet<PHost>;

    fn part(&self) -> &PSet<PPart>;

    fn modal(&self) -> &PSet<PModal>;

    /// What it looks like: every colour, size and face anything in its
    /// window is drawn with.
    fn theme(&self) -> Rc<dyn PTheme>;

    /// The rows of its activity bar, each with the menu it shows beside
    /// the bar. The bar is appkit's and the same in every app; an app
    /// with no rows has none.
    fn activity(&self) -> PActivity {
        PActivity::default()
    }

    /// Its user, as a table: their `name`, their `mail`, the `modal` that
    /// is their profile, and whatever else the profile is to show. The
    /// user's button and the profile are appkit's.
    fn user(&self) -> PTable {
        PTable::default()
    }

    /// The rows it adds to its bar while it runs, after the bar's top
    /// rows: the places a person is in, each with its picture. They are
    /// asked again whenever the window is laid out, and once a frame.
    fn rail(&self) -> Option<PRail> {
        None
    }

    /// What is true of the user now, a name and what it stands for: how
    /// many chats they have, which model answers. It is asked once a
    /// frame and shown in the profile under the rows of `user`.
    fn fact(&self) -> Option<PFact> {
        None
    }

    /// The looks a person can choose among, when its theme is one of
    /// several: the profile has a pill for each. A press on one turns
    /// the window to it and tells the app `look` and the look's name.
    fn shade(&self) -> Option<Rc<PShade>> {
        None
    }

    /// The view of one of its parts or modals: what it shows. `None` for
    /// one that shows nothing yet.
    fn content(&self, _part: &str) -> Option<Box<dyn PView>> {
        None
    }

    /// The host in front when the window is a mobile one and shows one
    /// area at a time. Nothing leaves the first area in front.
    fn front(&self) -> &str {
        ""
    }

    /// The host shown in a menu of the activity bar, in the group that
    /// menu's table gives it: a list of chats, of files. It is a host
    /// like any other, with its regions, panes and parts. Nothing leaves
    /// the bar without one.
    fn side(&self) -> &str {
        ""
    }

    /// Where its parts ask the window for deeds, if they do.
    fn wish(&self) -> Option<PWish> {
        None
    }

    /// Whether its spaces are shown as tabs over the body. An app that
    /// reaches its spaces from the bar alone says no.
    fn tab(&self) -> bool {
        true
    }

    /// The part that takes the keys when the window opens, if one does.
    fn focus(&self) -> &str {
        ""
    }

    /// The app's stage: its window with the content of its parts built.
    fn stage(&self) -> PStage
    where
        Self: Sized,
    {
        PStage::of(self)
    }

    /// The app's window, showing its first space.
    fn open(&self) -> PWindow {
        PWindow::new(self.name(), self.space().list().to_vec())
    }

    /// Every host, part and modal the app's spaces and hosts name that its
    /// sets do not have. Empty for an app whose window can be built whole.
    fn missing(&self) -> Vec<String> {
        self.open().missing(self.host(), self.part(), self.modal())
    }
}
