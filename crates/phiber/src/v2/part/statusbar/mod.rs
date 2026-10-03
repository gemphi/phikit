//! Statusbar (part) — horizontal segments: scene • frame • stats • mem • version.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::scene::load_cancel_key;
use crate::v2::scene::view_load_progress;
use crate::v2::scene::view_load_stage;
use crate::v2::state::BKey as _;
use crate::v2::state::BPath;
use crate::v2::state::BState;
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
        let path = BPath::owned(vec![Self::state_key(self.spec.kind)]);
        match s.get(&path) {
            Some(value) if !value.is_empty() => BView::label(value),
            _ => match self.fallback(s) {
                Some(value) => BView::label(value),
                None => BView::empty(),
            },
        }
    }
}

// `BStatusbarList` — iterates `set::SEGMENTS` and delegates each segment's
// rendering (including runtime value resolution) to `BStatusbarItem`.
// 
// While a blend import is running, a leading progress bar + stage label is
// prepended (Blender's `WM_progress` in the status bar), then a flexible
// spacer pushes the ordinary segments to the far end.

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

        // Import-progress readout, when an import is in flight. The progress
        // fraction gate is what decides visibility: absent => no import.
        if let Some(fraction) = view_load_progress(s) {
            let stage = view_load_stage(s).unwrap_or("Loading");
            out.push(BView::progress_bar(fraction, stage));
            // Blender's job progress has an "X" to cancel (`WM_jobs_stop`).
            out.push(BView::Button(
                BControl::new()
                    .with_icon("x")
                    .with_tooltip("Cancel")
                    .with_action(BAction::Set(load_cancel_key(), V_TRUE.to_string())),
            ));
            out.push(BView::spacer());
        }
        // A render under way is a job too (`render_view_cancel`): its
        // progress, and the same X — Esc in the render window is the other.
        if crate::v2::scene::render_job::rendering()
            && let Some(shot) = crate::v2::scene::render_job::shot().0
        {
            out.push(BView::progress_bar(shot.fraction(), "Rendering"));
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
        BView::row(BStatusbarList::new().items(s))
    }
}
