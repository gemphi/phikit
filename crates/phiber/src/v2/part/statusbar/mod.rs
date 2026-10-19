//! Statusbar (part) — horizontal segments: scene • frame • stats • mem • version.
//!
//! While a file is opening the whole bar gives way to a loading panel (see
//! [`loading_panel`]): the status zone grows to hold it and collapses back to
//! the one-bar segments when the scene settles. Blender shows only a thin bar
//! in the status bar during a read; Phiber opens the zone into a panel so the
//! viewport is never covered and the stages are legible with their timings —
//! a deliberate divergence, recorded here.

pub mod set;

use std::time::Duration;

use crate::v2::host::BHostType;
use crate::v2::scene::load_cancel_key;
use crate::v2::scene::loaded_path_key;
use crate::v2::scene::selected_key;
use crate::v2::scene::view_load_progress;
use crate::v2::scene::view_load_stage;
use crate::v2::state::BKey as _;
use crate::v2::state::BPath;
use crate::v2::state::BState;
use crate::v2::state::log;
use crate::v2::state::val::V_TRUE;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use self::set::SEGMENTS;
pub use set::BStatusSegment;
pub use set::BStatusSegmentSpec;

// `BStatusbarItem` — one segment. Reads its runtime value from state via the
// spec's key; falls back to the spec's label when the key is absent. No
// inline strings live here.
#[derive(Clone, Copy, Debug)]
pub struct BStatusbarItem {
    pub spec: BStatusSegmentSpec,
}

impl BStatusbarItem {
    pub fn new(spec: BStatusSegmentSpec) -> Self {
        Self { spec }
    }

    fn state_key(kind: BStatusSegment) -> &'static str {
        match kind {
            BStatusSegment::Scene => "scene",
            BStatusSegment::Frame => "frame",
            BStatusSegment::Stats => "stats",
            BStatusSegment::Memory => "memory",
            BStatusSegment::Version => "version",
        }
    }
}

impl BStatusbarItem {
    /// What a segment reads when nothing has written its key.
    ///
    /// NOT its own name. `unwrap_or(self.spec.label)` is why the foot of the
    /// window read "Scene 1 Stats Memory Version": three segments with no
    /// value announcing themselves. Blender's status bar shows the counts or
    /// shows nothing — it never prints the word "Memory" with no figure
    /// after it.
    fn fallback(&self, s: &BState) -> Option<String> {
        match self.spec.kind {
            // What the view layer shows, counted — the same line the
            // viewport's read-out draws (`scene::stats_line`), so the two
            // cannot disagree about how big the scene is.
            // The viewport's read-out draws these too (`scene::stats_pairs`),
            // so the two cannot disagree about how big the scene is.
            BStatusSegment::Stats => Some(crate::v2::scene::stats_line(s)),
            // The build, as Blender ends its status bar with its version.
            BStatusSegment::Version => Some(format!("v{}", env!("CARGO_PKG_VERSION"))),
            // Nothing has measured it, so there is nothing to say.
            BStatusSegment::Memory => None,
            BStatusSegment::Scene | BStatusSegment::Frame => None,
        }
    }
}

impl BViewItem for BStatusbarItem {
    fn build(&self, s: &BState) -> BView {
        // Stats is three read-outs, not one, and each gets its own caption.
        if self.spec.kind == BStatusSegment::Stats {
            return BView::row(crate::v2::scene::stats_pairs(s).into_iter().map(|(label, value)| BView::pair(label, value)).collect());
        }
        let path = BPath::owned(vec![Self::state_key(self.spec.kind)]);
        let value = match s.get(&path) {
            Some(value) if !value.is_empty() => Some(value.to_string()),
            _ => self.fallback(s),
        };
        match value {
            // The version is the build, not a reading of anything, so it
            // has no caption to recede behind.
            Some(value) if self.spec.kind == BStatusSegment::Version => BView::label(value),
            Some(value) => BView::pair(self.spec.label, value),
            None => BView::empty(),
        }
    }
}

// `BStatusbarList` — iterates `set::SEGMENTS` and delegates each segment's
// rendering (including runtime value resolution) to `BStatusbarItem`.
//
// A render under way prepends its progress bar + Cancel, then a spacer pushes
// the ordinary segments to the far end. A file *import* is not shown here —
// it takes the whole bar as the loading panel ([`loading_panel`]), so this
// list is only ever built when no import is in flight.

#[derive(Clone, Copy, Debug, Default)]
pub struct BStatusbarList;

impl BStatusbarList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BStatusbarList {
    fn items(&self, s: &BState) -> Vec<BView> {
        let mut out: Vec<BView> = Vec::new();

        // A render under way is a job too (`render_view_cancel`): its
        // progress, and the same X — Esc in the render window is the other.
        if crate::v2::scene::render_job::rendering()
            && let Some(shot) = crate::v2::scene::render_job::shot().0
        {
            out.push(BView::progress_bar(shot.fraction(), if crate::v2::scene::render_job::paused() { "Paused" } else { "Rendering" }));
            out.push(BView::Button(
                BControl::new()
                    .with_icon("x")
                    .with_tooltip("Cancel the render (Esc)")
                    .with_action(BAction::Set(crate::v2::state::OpKey::Render.path(), "cancel".to_string())),
            ));
            out.push(BView::spacer());
        }

        out.extend(
            SEGMENTS
                .iter()
                .map(|spec| BStatusbarItem::new(*spec).build(s)),
        );
        out
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BStatusbar {
    pub host: BHostType,
}

impl BStatusbar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BStatusbar {
    fn build(&self, s: &BState) -> BView {
        // A file in flight takes the whole bar: the zone grows to hold the
        // loading panel (the status area is content-sized), then collapses
        // back to the segment row when the scene settles and the progress
        // key is cleared. The progress-fraction gate is what decides it:
        // present => an import is running.
        match view_load_progress(s) {
            Some(fraction) => loading_panel(s, fraction),
            None => BView::row(BStatusbarList::new().items(s)),
        }
    }
}

/// How many of the newest action-log lines the loading panel shows as its
/// running log. Enough for the four load phases and a line or two before them.
const LOG_LINES: usize = 6;

/// The loading panel that fills the status zone while a file opens: the file
/// being read and a Cancel, the running log with the time each step took, and
/// a full-width progress bar across the bottom edge.
fn loading_panel(s: &BState, fraction: f32) -> BView {
    // Header: what is opening, and the Cancel (Blender's job "X"), pushed to
    // the far end by a spacer.
    let header = BView::row(vec![
        BView::icon("folder_open"),
        BView::label(opening(s)),
        BView::spacer(),
        BView::Button(
            BControl::new()
                .with_label("Cancel")
                .with_action(BAction::Set(load_cancel_key(), V_TRUE.to_string())),
        ),
    ]);

    let mut body = vec![header];
    body.extend(running_log());
    // The bar last, so it sits across the bottom edge. Its label is the live
    // (detailed) stage — "Reading objects 431/2104 Cube.003" — on its own
    // line beneath the full-width bar, so a changing stage never moves it.
    let stage = view_load_stage(s).unwrap_or("Opening").to_string();
    body.push(BView::progress_bar_wide(fraction, stage));
    BView::column(body)
}

/// The file being opened, by its name alone (the selection that triggered the
/// read, else the path the scene is settling to).
fn opening(s: &BState) -> String {
    let path = s
        .get(&selected_key())
        .filter(|p| !p.is_empty())
        .or_else(|| s.get(&loaded_path_key()))
        .unwrap_or("");
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}

/// The newest action-log lines, each with the time since the line before it —
/// the panel's running log. The log already carries the load's phase lines
/// (`app::load` pushes one per phase); reading it back keeps the panel and the
/// session journal telling the same story.
fn running_log() -> Vec<BView> {
    let entries = log::entries();
    let shown = entries.len().min(LOG_LINES);
    (0..shown)
        .map(|i| {
            let timing = match entries.get(i + 1) {
                Some(prev) => BView::label(elapsed(entries[i].at.saturating_duration_since(prev.at))),
                None => BView::empty(),
            };
            BView::row(vec![BView::label(entries[i].text.clone()), BView::spacer(), timing])
        })
        .collect()
}

/// A step's elapsed time, as the journal writes gaps: milliseconds under a
/// second, seconds above it.
fn elapsed(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 1000.0 {
        format!("+{ms:.0} ms")
    } else {
        format!("+{:.1} s", ms / 1000.0)
    }
}
