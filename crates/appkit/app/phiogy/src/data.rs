//! Phiogy's tables, as the files in `data/` hold them. They are built
//! into the program, so the same app runs where there is no disk to read:
//! in a browser. Nothing a person reads, no limit and no name on the
//! wire is anywhere else.

/// What Phiogy is to appkit, and its window.
pub const APP: &str = include_str!("../data/app.tsv");
pub const SPACE: &str = include_str!("../data/space.tsv");
pub const LAYOUT: &str = include_str!("../data/layout.tsv");
pub const HOST: &str = include_str!("../data/host.tsv");
pub const REGION: &str = include_str!("../data/region.tsv");
pub const PART: &str = include_str!("../data/part.tsv");
pub const MODAL: &str = include_str!("../data/modal.tsv");
/// The words: what a person reads in the window, then what the command
/// line says. Two files, one table.
pub const TEXT: &str = concat!(include_str!("../data/text.tsv"), include_str!("../data/say.tsv"));
pub const THEME: &str = include_str!("../data/theme.tsv");
pub const LOOK: &str = include_str!("../data/look.tsv");
pub const ACTIVITY: &str = include_str!("../data/activity.tsv");
pub const MENU: &str = include_str!("../data/menu.tsv");
pub const USER: &str = include_str!("../data/user.tsv");
/// How the web Phiogy's files are read, and its icons drawn.
pub const SITE: &str = include_str!("../data/site.tsv");
pub const ICON: &str = include_str!("../data/icon.tsv");
/// Phiora, which is asked for the app's document before its file is.
pub const PHIORA: &str = include_str!("../data/phiora.tsv");
/// How a place over the network is reached: the app's server, a line.
pub const NET: &str = include_str!("../data/net.tsv");
/// What a picture of Phiegg's is called on the app's server.
pub const MEDIA: &str = include_str!("../data/media.tsv");
/// The wallet a person starts with, and the names in its documents.
pub const WALLET: &str = include_str!("../data/wallet.tsv");
/// What a theme that is worn changes of the window.
pub const WEAR: &str = include_str!("../data/wear.tsv");
/// The pages that are no room, and the space each is shown in.
pub const PAGE: &str = include_str!("../data/page.tsv");
/// Making clubs and PFTs from a sheet, and publishing them on Phiora.
pub const MAKE: &str = include_str!("../data/make.tsv");
/// What a person keeps: their own documents, and where.
pub const KEEP: &str = include_str!("../data/keep.tsv");
/// A room's controls, and the viewer's: what each does, its icon, its
/// word.
pub const ACT: &str = include_str!("../data/act.tsv");
pub const TOOL: &str = include_str!("../data/tool.tsv");
/// The inks a mark on a picture can be in.
pub const MARK: &str = include_str!("../data/mark.tsv");
/// The 3D view's tools.
pub const VIEWPORT: &str = include_str!("../data/viewport.tsv");
/// Search, as Phiegg's is: its tabs, what each finds, where Phiegg's
/// records and what is sought most often are.
pub const SEARCH: &str = include_str!("../data/search.tsv");
/// A person's profile: its tabs.
pub const PROFILE: &str = include_str!("../data/profile.tsv");
/// One thing of a shop: its tabs and figures.
pub const THING: &str = include_str!("../data/thing.tsv");
/// The column beside a room: the host's tabs, facts and rules.
pub const ASIDE: &str = include_str!("../data/aside.tsv");
/// A record's page: the facts of each kind.
pub const RECORD: &str = include_str!("../data/record.tsv");
/// Where the pictures the site names are on disk.
pub const ASSET: &str = include_str!("../data/asset.tsv");
/// The rules of a call.
pub const LIMIT: &str = include_str!("../data/limit.tsv");
pub const NAME: &str = include_str!("../data/name.tsv");
pub const FAULT: &str = include_str!("../data/fault.tsv");
pub const WIRE: &str = include_str!("../data/wire.tsv");
pub const ICE: &str = include_str!("../data/ice.tsv");
/// The line.
pub const LINE: &str = include_str!("../data/line.tsv");
pub const HEAD: &str = include_str!("../data/head.tsv");
pub const STATUS: &str = include_str!("../data/status.tsv");
pub const SERVE: &str = include_str!("../data/serve.tsv");

/// This window as a caller: what it asks of the line, the words two
/// callers say to each other, and its own numbers.
pub const DIAL: &str = include_str!("../data/dial.tsv");
pub const SIGNAL: &str = include_str!("../data/signal.tsv");
pub const PEER: &str = include_str!("../data/peer.tsv");

/// The rows of a table with more than two cells a line: the cells of
/// every line that is not empty and does not start with `#`, between
/// tabs.
pub fn row(text: &str) -> Vec<Vec<&str>> {
    let line = text.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#'));
    line.map(|line| line.split('\t').map(str::trim).collect()).collect()
}

/// The asset library: its places, what a file is by its end, its
/// filters, orders and tools, how a source is turned into USD, the
/// colours and finishes a model is dressed in, its 3D pictures, and
/// the forms it is shown in, their lists and the values they begin with.
pub const PLACE: &str = include_str!("../data/place.tsv");
pub const KIND: &str = include_str!("../data/kind.tsv");
pub const FILTER: &str = include_str!("../data/filter.tsv");
pub const CONVERT: &str = include_str!("../data/convert.tsv");
pub const PAINT: &str = include_str!("../data/paint.tsv");
pub const FINISH: &str = include_str!("../data/finish.tsv");
pub const MODEL: &str = include_str!("../data/model.tsv");
pub const FORM: &str = include_str!("../data/form.tsv");
pub const LISTING: &str = include_str!("../data/listing.tsv");
pub const STATE: &str = include_str!("../data/state.tsv");
/// The asset library's own words and numbers: where Phiegg's shelf is,
/// its icons, a set's size, the drives' letters.
pub const SHELF: &str = include_str!("../data/shelf.tsv");
