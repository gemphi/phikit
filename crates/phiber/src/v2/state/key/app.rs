//! The window's own keys — things that belong to the app rather than to a
//! Blender editor: the window, the theme, links, typing, the command
//! palette, the status log and the collaboration session.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppKey {
    /// The window's width, for narrow-layout decisions.
    WindowWidth,
    /// The window's height, for the things that may not grow past it — a menu
    /// popover is as tall as its rows need or as the window allows, whichever
    /// is less (`view::draw::popover`).
    WindowHeight,
    /// The theme's slug.
    Theme,
    /// The theme's mode: `dark` or `light` (`BThemeMode`).
    ThemeMode,
    /// The dimensional coat's scope: `auto`, `flat` or `spatial`
    /// (`BFlavourScope`, `theme/style/flavour`). One setting over the whole
    /// window — flat, or the spatial virtual-world look.
    Flavour,
    UseGradients,
    /// A URL to hand to the system browser (`wm.url_open`); side effect only.
    Link,
    /// The key of the field that has the caret, if any.
    Edit,
    /// What that field held when typing started, for Escape.
    EditUndo,
    /// The command palette's filter text.
    CommandQuery,
    /// The editor the palette was opened over (its host slug): Menu Search
    /// lists its menus after the Topbar's, as Blender's F3 searches the
    /// area under the pointer.
    CommandHost,
    /// The palette's tab: what it searches — everything, the spaces, the
    /// editors, the panels or the menus (`modal::command::find::BFindKind`).
    CommandTab,
    /// How the palette lays its results out: lines, or cards of three sizes
    /// (`modal::command::layout::BFindLayout`).
    CommandLayout,
    /// Where a picked palette result goes: the keys to set, one
    /// `ctx.path=value` per line (`find::BFindGoRule`), so a pick can open a
    /// space, then an editor's tab, then one of its panels in one step.
    CommandGo,
    /// The Welcome window's tab: New File, Recent Files or Learn
    /// (`modal::splash::tab::BWelcomeTab`).
    WelcomeTab,
    /// The rename dialog's field (`TOPBAR_PT_name`): the name being typed.
    RenameText,
    /// The frame of the marker the dialog renames (`marker.rename`); absent
    /// when it renames the active object.
    RenameMarker,
    /// Whether the status bar's action log is unfolded.
    StatusLogOpen,
    /// Which column the unfolded log is ordered by, and which way.
    StatusLogOrder,
    StatusLogAscending,
    /// A profile field: `ctx.profile.<field>`.
    Profile(&'static str),
    /// The collaboration session's status (`online` / `offline`).
    CollabStatus,
    CollabServer,
    /// One peer of the session: `ctx.collab.peer.<n>`.
    CollabPeer(&'static str),
    /// One channel of the session: `ctx.collab.channel.<n>` — `name\towner`.
    CollabChannel(&'static str),
    /// The render device picked in Preferences ▸ System: a GPU's name, or
    /// empty for Auto (`part::pref::device`).
    Gpu,
    /// Preferences ▸ Viewport ▸ Quality ▸ Viewport Anti-Aliasing
    /// (`U.viewport_aa`): `OFF`, `FXAA` or a sample count
    /// (`scene::viewport_aa`); unset is Blender's 8.
    ViewportAa,
}

impl BKey for AppKey {
    fn path(&self) -> BPath {
        match self {
            Self::WindowWidth => BPath::owned(vec!["window", "width"]),
            Self::WindowHeight => BPath::owned(vec!["window", "height"]),
            Self::Theme => BPath::owned(vec!["theme"]),
            Self::ThemeMode => BPath::owned(vec!["theme_mode"]),
            Self::Flavour => BPath::owned(vec!["flavour"]),
            Self::UseGradients => BPath::owned(vec!["use_gradients"]),
            Self::Link => BPath::owned(vec!["link"]),
            Self::Edit => BPath::owned(vec!["edit"]),
            Self::EditUndo => BPath::owned(vec!["edit_undo"]),
            Self::CommandQuery => BPath::owned(vec!["command", "query"]),
            Self::CommandHost => BPath::owned(vec!["command", "host"]),
            Self::CommandTab => BPath::owned(vec!["command", "tab"]),
            Self::CommandLayout => BPath::owned(vec!["command", "layout"]),
            Self::CommandGo => BPath::owned(vec!["command", "go"]),
            Self::WelcomeTab => BPath::owned(vec!["welcome", "tab"]),
            Self::RenameText => BPath::owned(vec!["rename", "text"]),
            Self::RenameMarker => BPath::owned(vec!["rename", "marker"]),
            Self::StatusLogOpen => BPath::owned(vec!["status", "log", "open"]),
            Self::StatusLogOrder => BPath::owned(vec!["status", "log", "order"]),
            Self::StatusLogAscending => BPath::owned(vec!["status", "log", "ascending"]),
            Self::Profile(field) => BPath::owned(vec!["profile", field]),
            Self::CollabStatus => BPath::owned(vec!["collab", "status"]),
            Self::CollabServer => BPath::owned(vec!["collab", "server"]),
            Self::CollabPeer(n) => BPath::owned(vec!["collab", "peer", n]),
            Self::CollabChannel(n) => BPath::owned(vec!["collab", "channel", n]),
            Self::Gpu => BPath::owned(vec!["gpu"]),
            Self::ViewportAa => BPath::owned(vec!["pref", "viewport_aa"]),
        }
    }
}
