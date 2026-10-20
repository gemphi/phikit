//! Draws the manual's **step-by-step** figures: each step of a workflow as
//! the whole window — the top bar's menus and workspace tabs, the activity
//! rail, each area with its editor's icon, name and header menus, the menu a
//! step opens with its real entries and shortcuts — then a numbered marker
//! on what to press, and the step's caption under the window.
//!
//! Nothing here is drawn from memory. Every label is read from the sets the
//! app itself is built from: the menus from `BMenuSpec::list_for`, the
//! workspace tabs from `BSpaceType::ALL`, the areas from each space's
//! `spec().areas`, the rail from `activity::set::ENTRIES`, the Inspector's
//! tabs and Render panels from `navbar::tabs_for` and `props::sections_for`,
//! the status bar from `statusbar::set::SEGMENTS` and `event_summary`, the
//! Outliner's Status rows from `outliner::status::rows` over a real event
//! (`BRenderAudit::open_event` + `render_job::report_*`), the Image editor's
//! info line from `BImageView::info_pairs_at`, the file browser from
//! `BFilePurpose`, and the icons from `assets/icons`. A renamed entry or a
//! new area shows in the manual the next time this runs.
//!
//! The figure sits in the manual's deck (`BDeck::SILVER`): the window is the
//! face, two narrower cards behind it show their edges above and below.
//!
//! Colours are **theme variables** set on the figure (`--ph-panel`, …): the
//! figure carries the deck palette's dark and light values (the light ones
//! under `prefers-color-scheme: light`), and `theme.css`, written beside the
//! figures, repaints them from the docs site's own switches — `data-theme`
//! for the mode, `data-theme-style` for the palette, the brand's
//! `--phi-color-primary` for the accent — when the figure is inlined.
//!
//! Run with:  cargo run --release -p phiber --example manual_steps
//!            (MANUAL_RENDER=<png> puts a real render in the Image editor)
//! Writes to: crates/phiber/docs/manual/steps/<workflow>/NN-<slug>.svg,
//!            crates/phiber/docs/manual/steps/<workflow>.md and theme.css

use phiber::v2::area::BAreaType;
use phiber::v2::host::BHostType;
use phiber::v2::manual::BDeck;
use phiber::v2::part::activity::set::{BActivityGroup, ENTRIES};
use phiber::v2::part::file::purpose::BFilePurpose;
use phiber::v2::part::menubar::item::{BMenuItem, BMenuKind, BMenuSpec};
use phiber::v2::part::navbar::set::tabs_for;
use phiber::v2::part::outliner::mode::{BOutlineMode, MODES};
use phiber::v2::part::props::prop::{BDefault, BPropKind, BPropSpec};
use phiber::v2::part::props::set::{BSectionSpec, sections_for};
use phiber::v2::part::statusbar::{BStatusbarItem, event_summary, set::SEGMENTS};
use phiber::v2::view::{BView, BViewItem};
use phiber::v2::scene::render_audit::BRenderAudit;
use phiber::v2::scene::render_job::{report_end, report_sample};
use phiber::v2::scene::render_status::BRenderStatus;
use phiber::v2::scene::BRenderPhase;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::event::{BEvent, BEvents};
use phiber::v2::state::{BState, OutlinerKey};
use phiber::v2::theme::{BColor, BPalette, BPaletteName, BThemeMode, palette_in};
use phiber::v2::view::draw::canvas::image::{BChannels, BImageView};
use std::fmt::Write as _;
use std::time::Duration;

/// The window, in figure units; the deck around it adds its edges.
const W: f32 = 1280.0;
const H: f32 = 760.0;
const BAR: f32 = 28.0;
const RAIL: f32 = 56.0;
const STATUS: f32 = 24.0;
const HEADER: f32 = 26.0;
const ROW: f32 = 22.0;
const CAPTION: f32 = 64.0;

const ICONS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons");

fn hex(c: BColor) -> String {
    let (h, s, l) = (c.0 / 360.0, c.1 / 100.0, c.2 / 100.0);
    let f = |n: f32| {
        let k = (n + h * 12.0) % 12.0;
        let a = s * l.min(1.0 - l);
        let v = l - a * (-1.0f32).max((k - 3.0).min(9.0 - k).min(1.0));
        (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
    };
    format!("#{:02x}{:02x}{:02x}", f(0.0), f(8.0), f(4.0))
}

/// The tokens a figure paints with, by name, from a palette.
fn tokens(p: &BPalette) -> Vec<(&'static str, String)> {
    vec![
        ("app", hex(p.bg_app)),
        ("top", hex(p.bg_top)),
        ("rail", hex(p.bg_activity)),
        ("header", hex(p.bg_header)),
        ("panel", hex(p.bg_panel)),
        ("viewport", hex(p.bg_viewport)),
        ("control", hex(p.bg_control)),
        ("hover", hex(p.bg_hover)),
        ("active", hex(p.bg_active)),
        ("selected", hex(p.row_selected)),
        ("text", hex(p.text)),
        ("dim", hex(p.text_dim)),
        ("accent", hex(p.accent)),
        ("border", hex(p.border)),
        ("sep", hex(p.sep)),
        ("scrim", hex(p.bg_modal_dim)),
        // Text on the accent: the accent is a saturated mid-tone in every
        // palette, so white reads on it in both modes.
        ("on-accent", "#ffffff".to_string()),
    ]
}

fn declare(p: &BPalette) -> String {
    tokens(p).into_iter().map(|(n, v)| format!("--ph-{n}:{v};")).collect()
}

fn slug(s: &str) -> String {
    s.to_lowercase().split(|c: char| !c.is_ascii_alphanumeric()).filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-")
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Width a label takes at the figure's 12px font, near enough to lay out.
fn width(s: &str) -> f32 {
    s.chars().count() as f32 * 6.7
}

/// An SVG being drawn, in window coordinates.
struct Fig {
    svg: String,
    /// Where the step's marker points; the last part drawn that names a
    /// target wins.
    target: Option<(f32, f32)>,
}

impl Fig {
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, fill: &str, r: f32) {
        let _ = writeln!(self.svg, r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}" class="f-{fill}"/>"#);
    }

    fn framed(&mut self, x: f32, y: f32, w: f32, h: f32, fill: &str, r: f32) {
        let _ = writeln!(self.svg, r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{r}" class="f-{fill} s-border" stroke-width="0.8"/>"#);
    }

    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, stroke: &str) {
        let _ = writeln!(self.svg, r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" class="s-{stroke}"/>"#);
    }

    fn text(&mut self, x: f32, y: f32, s: &str, fill: &str, bold: bool) {
        let _ = writeln!(self.svg, r#"<text x="{x:.1}" y="{y:.1}" class="f-{fill}{}">{}</text>"#, if bold { " b" } else { "" }, esc(s));
    }

    fn small(&mut self, x: f32, y: f32, s: &str, fill: &str) {
        let _ = writeln!(self.svg, r#"<text x="{x:.1}" y="{y:.1}" text-anchor="middle" class="f-{fill} sm">{}</text>"#, esc(s));
    }

    /// One of the app's own icons, stroked in `color`.
    fn icon(&mut self, name: &str, x: f32, y: f32, size: f32, color: &str) {
        let Ok(src) = std::fs::read_to_string(format!("{ICONS}/{name}.svg")) else {
            return;
        };
        let Some(open) = src.find("<svg") else { return };
        let placed = format!(r#"<svg x="{x:.1}" y="{y:.1}" width="{size}" height="{size}" class="c-{color}""#);
        self.svg.push_str(&src[..open]);
        self.svg.push_str(&placed);
        self.svg.push_str(&src[open + 4..]);
        self.svg.push('\n');
    }
}

/// What the status bar is showing.
#[derive(Clone, Copy, PartialEq)]
enum Bar {
    Idle,
    /// A render under way: its progress, the X and the event.
    Rendering,
    /// The render's event, finished.
    Done,
}

/// One step: the screen as it stands, what is open over it, and the
/// caption.
struct Step {
    title: &'static str,
    caption: &'static str,
    space: BSpaceType,
    /// The Center area's editor, when the step has changed it.
    center: Option<BHostType>,
    outliner: BOutlineMode,
    bar: Bar,
    /// A menu open: the header's host, the menu's label, the entry marked.
    menu: Option<(BHostType, &'static str, &'static str)>,
    browser: Option<BFilePurpose>,
    /// Where the marker goes, when it is not the menu or the browser.
    mark: Mark,
}

#[derive(Clone, Copy, PartialEq)]
enum Mark {
    None,
    Info,
    StatusButton,
    Outliner,
    Viewport,
}

/// What the figures depict, made the way the app makes it.
struct World {
    state: BState,
    event_running: BEvent,
    event_done: BEvent,
    info_line: String,
    render: Option<String>,
}

impl World {
    fn new() -> Self {
        let state = BState::default();
        let event = BRenderAudit::open_event(&state, Some(1.0), (1920, 1080), 256, 3, None);
        report_sample(event, Duration::from_millis(31_200), 12, 256, false);
        let event_running = BEvents::all().into_iter().find(|e| e.id == event).expect("the event just opened");
        report_sample(event, Duration::from_millis(432_400), 256, 256, true);
        report_end(event, "Finished", Duration::from_millis(432_400));
        let event_done = BEvents::all().into_iter().find(|e| e.id == event).expect("the event");
        let view = BImageView {
            zoom: None,
            pan: (0.0, 0.0),
            channels: BChannels::Color,
            pass: "Combined".into(),
            slot_filled: true,
            status: Some(BRenderStatus { w: 1920, h: 1080, taken: 12, target: 256, done: false, has_image: true, depth: false, normal: false, albedo: false, bytes: 111 << 20, peak: 111 << 20, seq: 1 }),
            saved: false,
            phase: BRenderPhase::Rendering,
            shown: phiber::v2::part::image::shown::BShownImage::RenderResult,
        };
        let pairs = view.info_pairs_at(&("Scene".into(), "ViewLayer".into()), "1", Duration::from_millis(31_200)).expect("a status");
        let info_line = BImageView::info_line(&pairs).expect("every pair");
        let render = std::env::var("MANUAL_RENDER").ok().and_then(|p| std::fs::read(p).ok()).map(|bytes| {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.encode(bytes)
        });
        Self { state, event_running, event_done, info_line, render }
    }
}

/// Each area's rectangle in the window: the rail down the left, Right
/// areas stacked in a column (the first 40%), the Bottom strip above the
/// status bar, the Center the rest.
fn layout(space: BSpaceType) -> Vec<(BAreaType, BHostType, (f32, f32, f32, f32))> {
    let areas = space.spec().areas;
    let rights: Vec<_> = areas.iter().filter(|a| a.kind == BAreaType::Right).collect();
    let strip = areas.iter().any(|a| a.kind == BAreaType::Bottom && a.default_host != BHostType::Statusbar);
    let (rw, bh) = (if rights.is_empty() { 0.0 } else { 340.0 }, if strip { 132.0 } else { 0.0 });
    let body = H - BAR - STATUS;
    let mut right_y = BAR;
    let mut out = Vec::new();
    for a in areas {
        let r = match (a.kind, a.default_host) {
            (BAreaType::Top, _) => (0.0, 0.0, W, BAR),
            (BAreaType::Left, _) => (0.0, BAR, RAIL, body),
            (BAreaType::Right, _) => {
                let first = std::ptr::eq(a, *rights.first().expect("a right area"));
                let h = if rights.len() == 1 { body } else if first { body * 0.4 } else { (body * 0.6) / (rights.len() - 1) as f32 };
                let r = (W - rw, right_y, rw, h);
                right_y += h;
                r
            }
            (BAreaType::Bottom, BHostType::Statusbar) => (0.0, H - STATUS, W, STATUS),
            (BAreaType::Bottom, _) => (RAIL, H - STATUS - bh, W - RAIL - rw, bh),
            (BAreaType::Center, _) => (RAIL, BAR, W - RAIL - rw, body - bh),
        };
        out.push((a.kind, a.default_host, r));
    }
    out
}

fn topbar(f: &mut Fig, step: &Step, menus: &mut Vec<(BHostType, &'static str, f32, f32)>) {
    f.rect(0.0, 0.0, W, BAR, "top", 0.0);
    f.icon("phiber", 10.0, 6.0, 16.0, "accent");
    let mut x = 36.0;
    for m in BMenuSpec::list_for(BHostType::Topbar) {
        f.text(x, 18.5, m.label, "text", false);
        menus.push((BHostType::Topbar, m.label, x, 18.5));
        x += width(m.label) + 16.0;
    }
    x += 18.0;
    for space in BSpaceType::ALL {
        let on = *space == step.space;
        if on {
            f.rect(x - 7.0, 4.0, width(space.label()) + 14.0, 20.0, "active", 5.0);
        }
        f.text(x, 18.5, space.label(), if on { "text" } else { "dim" }, on);
        x += width(space.label()) + 18.0;
    }
}

/// The rail: each entry an icon with its name under it, Slack-style, in
/// its group — top, centre, bottom.
fn rail(f: &mut Fig, (x, y, w, h): (f32, f32, f32, f32)) {
    f.rect(x, y, w, h, "rail", 0.0);
    let cell = 50.0;
    let place = |group: BActivityGroup, k: usize, n: usize| match group {
        BActivityGroup::Top => y + 8.0 + k as f32 * cell,
        BActivityGroup::Center => y + h / 2.0 - n as f32 * cell / 2.0 + k as f32 * cell,
        BActivityGroup::Bottom => y + h - 8.0 - (n - k) as f32 * cell,
    };
    for group in [BActivityGroup::Top, BActivityGroup::Center, BActivityGroup::Bottom] {
        let entries: Vec<_> = ENTRIES.iter().filter(|e| e.group == group).collect();
        for (k, e) in entries.iter().enumerate() {
            let ey = place(group, k, entries.len());
            let on = e.label == "Spaces";
            if on {
                f.rect(x + 8.0, ey, w - 16.0, cell - 6.0, "active", 8.0);
            }
            f.icon(e.icon, x + w / 2.0 - 10.0, ey + 5.0, 20.0, if on { "text" } else { "dim" });
            f.small(x + w / 2.0, ey + 38.0, e.label, if on { "text" } else { "dim" });
        }
    }
}

/// An area's header: the editor's icon and name, then its menus while they
/// fit.
fn header(f: &mut Fig, host: BHostType, (x, y, w, _): (f32, f32, f32, f32), menus: &mut Vec<(BHostType, &'static str, f32, f32)>) -> f32 {
    f.rect(x + 2.0, y + 2.0, w - 4.0, HEADER, "header", 6.0);
    f.icon(host.spec().icon, x + 10.0, y + 7.0, 16.0, "dim");
    let mut cx = x + 32.0;
    f.text(cx, y + 19.0, host.spec().label, "text", true);
    cx += width(host.spec().label) + 18.0;
    for m in BMenuSpec::list_for(host) {
        if cx + width(m.label) > x + w - 12.0 {
            break;
        }
        f.text(cx, y + 19.0, m.label, "text", false);
        menus.push((host, m.label, cx, y + 19.0));
        cx += width(m.label) + 14.0;
    }
    cx
}

/// The Outliner, its rows as `outliner::tree::flatten` builds them for the
/// mode: the startup scene's tree in View Layer, the event rows in Status.
fn outliner(f: &mut Fig, world: &World, mode: BOutlineMode, r: (f32, f32, f32, f32), end: f32) {
    let (x, y, w, _) = r;
    let label = MODES.iter().find(|m| m.kind == mode).map_or("", |m| m.label);
    let mx = (x + w - 16.0 - width(label) - 24.0).max(end);
    f.framed(mx, y + 5.0, width(label) + 24.0, 20.0, "control", 5.0);
    f.text(mx + 8.0, y + 19.0, label, "text", false);
    f.text(mx + width(label) + 12.0, y + 19.0, "▾", "dim", false);
    if mode == BOutlineMode::Status {
        f.target = Some((mx, y + 15.0));
    }
    let mut ry = y + HEADER + 8.0;
    let mut state = world.state.clone();
    state.set(OutlinerKey::DisplayMode, mode.slug());
    let rows: Vec<_> = phiber::v2::part::outliner::tree::flatten(&state).into_iter().map(|r| (r.depth, r.icon, r.label, r.expanded)).collect();
    for (k, (depth, icon, text, open)) in rows.iter().enumerate() {
        if ry + ROW > r.1 + r.3 - 4.0 {
            break;
        }
        if k % 2 == 1 {
            f.rect(x + 4.0, ry, w - 8.0, ROW, "hover", 0.0);
        }
        let ix = x + 12.0 + *depth as f32 * 16.0;
        if let Some(open) = open {
            f.text(ix, ry + 15.0, if *open { "▾" } else { "▸" }, "dim", false);
        }
        f.icon(icon, ix + 12.0, ry + 3.0, 16.0, "dim");
        f.text(ix + 34.0, ry + 15.0, text, "text", false);
        ry += ROW;
    }
}

/// The Inspector on its Render tab: the tab column of `tabs_for(Buts)` and
/// the Render panels in `sections_for` order, open or closed as Blender
/// starts them, each open one with its rows at their RNA defaults.
fn inspector(f: &mut Fig, (x, y, w, h): (f32, f32, f32, f32)) {
    let col = 30.0;
    f.rect(x + 2.0, y + HEADER + 2.0, col, h - HEADER - 6.0, "header", 0.0);
    let mut ty = y + HEADER + 8.0;
    for tab in tabs_for(BHostType::Buts) {
        if ty + 24.0 > y + h - 4.0 {
            break;
        }
        let on = tab.label == "Render";
        if on {
            f.rect(x + 4.0, ty - 2.0, col - 4.0, 24.0, "active", 5.0);
        }
        f.icon(tab.icon, x + 9.0, ty + 2.0, 16.0, if on { "text" } else { "dim" });
        ty += 26.0;
    }
    let mut py = y + HEADER + 8.0;
    let sections: Vec<_> = sections_for(BHostType::Buts, "Render").iter().filter(|s| !s.headless).collect();
    for section in sections {
        panel(f, section, x + col + 8.0, w - col - 14.0, &mut py, y + h - 6.0);
    }
}

/// One panel: its header strip, then — when open — its rows and its
/// sub-panels, indented.
fn panel(f: &mut Fig, section: &BSectionSpec, x: f32, w: f32, y: &mut f32, bottom: f32) {
    if *y + ROW > bottom {
        return;
    }
    f.rect(x, *y, w, ROW, "control", 5.0);
    f.text(x + 8.0, *y + 15.0, if section.closed { "▸" } else { "▾" }, "dim", false);
    f.text(x + 22.0, *y + 15.0, section.title, "text", true);
    *y += ROW + 3.0;
    if section.closed {
        return;
    }
    for prop in section.props {
        if *y + ROW > bottom {
            return;
        }
        prop_row(f, prop, x, w, *y);
        *y += ROW + 2.0;
    }
    for sub in section.list {
        panel(f, sub, x + 10.0, w - 10.0, y, bottom);
    }
}

/// A property row as the panel lays it: the label right-aligned in the
/// left 45%, the control in the rest, showing the RNA default.
fn prop_row(f: &mut Fig, prop: &BPropSpec, x: f32, w: f32, y: f32) {
    let split = x + w * 0.45;
    f.text(split - 8.0 - width(prop.label), y + 15.0, prop.label, "dim", false);
    let (cx, cw) = (split, w * 0.55 - 6.0);
    match (prop.kind, prop.default) {
        (BPropKind::Bool, value) => {
            let on = value == Some(BDefault::Bool(true));
            f.framed(cx, y + 4.0, 14.0, 14.0, if on { "accent" } else { "control" }, 3.0);
            if on {
                f.text(cx + 2.0, y + 15.5, "✓", "on-accent", true);
            }
        }
        (BPropKind::Enum(items), value) => {
            let key = match value {
                Some(BDefault::Enum(key)) => key,
                _ => items.first().map_or("", |i| i.key),
            };
            let label = items.iter().find(|i| i.key == key).map_or(key, |i| i.label);
            f.framed(cx, y + 1.0, cw, ROW - 2.0, "control", 5.0);
            f.text(cx + 8.0, y + 15.0, label, "text", false);
            f.text(cx + cw - 16.0, y + 15.0, "▾", "dim", false);
        }
        (BPropKind::Int { .. } | BPropKind::Float { .. }, value) => {
            let text = match value {
                Some(BDefault::Int(n)) => n.to_string(),
                Some(BDefault::Float(v)) => format!("{v:.3}").trim_end_matches('0').trim_end_matches('.').to_string(),
                _ => if matches!(prop.kind, BPropKind::Int { .. }) { "0".into() } else { "0.000".into() },
            };
            f.framed(cx, y + 1.0, cw, ROW - 2.0, "control", 5.0);
            f.text(cx + cw / 2.0 - width(&text) / 2.0, y + 15.0, &text, "text", false);
        }
        _ => f.framed(cx, y + 1.0, cw, ROW - 2.0, "control", 5.0),
    }
}

/// The Image editor's canvas: the render (when one was given) and the info
/// line across its top, as `draw_render_info` lays it.
fn image_canvas(f: &mut Fig, world: &World, step: &Step, (x, y, w, h): (f32, f32, f32, f32)) {
    let (cw, ch) = (w - 40.0, h - HEADER - 60.0);
    let (iw, ih) = if cw / ch > 16.0 / 9.0 { (ch * 16.0 / 9.0, ch) } else { (cw, cw * 9.0 / 16.0) };
    let (ix, iy) = (x + (w - iw) / 2.0, y + HEADER + 30.0 + (ch - ih) / 2.0);
    match &world.render {
        Some(png) => {
            let _ = writeln!(f.svg, r#"<image x="{ix:.1}" y="{iy:.1}" width="{iw:.1}" height="{ih:.1}" preserveAspectRatio="xMidYMid slice" href="data:image/png;base64,{png}"/>"#);
        }
        None => {
            f.rect(ix, iy, iw, ih, "control", 0.0);
            f.small(ix + iw / 2.0, iy + ih / 2.0, "Render Result", "dim");
        }
    }
    if step.bar == Bar::Rendering {
        f.rect(x + 2.0, y + HEADER + 2.0, w - 4.0, 22.0, "header", 0.0);
        f.text(x + 12.0, y + HEADER + 17.0, &world.info_line, "text", false);
        if step.mark == Mark::Info {
            f.target = Some((x + 12.0 + width("Frame:1 | Time:00:31.20 | Remaining:"), y + HEADER + 24.0));
        }
    }
}

/// The status bar, as `BStatusbarList::items` lays it: a render's progress
/// and X while it runs, the newest event as a button, then the segments.
fn statusbar(f: &mut Fig, world: &World, step: &Step, (x, y, w, h): (f32, f32, f32, f32)) {
    f.rect(x, y, w, h, "top", 0.0);
    let mut sx = x + 10.0;
    if step.bar == Bar::Rendering {
        f.framed(sx, y + 6.0, 140.0, h - 12.0, "control", 4.0);
        f.rect(sx, y + 6.0, 140.0 * 12.0 / 256.0, h - 12.0, "accent", 4.0);
        f.text(sx + 150.0, y + 16.5, "Rendering", "dim", false);
        f.icon("x", sx + 220.0, y + 5.0, 14.0, "dim");
        sx += 250.0;
    }
    if step.bar != Bar::Idle {
        let event = if step.bar == Bar::Rendering { &world.event_running } else { &world.event_done };
        let summary = event_summary(event);
        f.framed(sx - 6.0, y + 3.0, width(&summary) + 12.0, h - 6.0, "control", 4.0);
        f.text(sx, y + 16.5, &summary, "text", false);
        if step.mark == Mark::StatusButton {
            f.target = Some((sx + width(&summary) / 2.0, y + 3.0));
        }
    }
    // The segments as `BStatusbarItem::build` makes them for this state:
    // a caption in the dim shade, its figure in the text one.
    let mut pairs = Vec::new();
    for seg in SEGMENTS {
        flatten_view(&BStatusbarItem::new(*seg).build(&world.state), &mut pairs);
    }
    let mut rx = x + w - 10.0;
    for (caption, value) in pairs.iter().rev() {
        rx -= width(value);
        f.text(rx, y + 16.5, value, "text", false);
        if !caption.is_empty() {
            rx -= width(caption) + 5.0;
            f.text(rx, y + 16.5, caption, "dim", false);
        }
        rx -= 16.0;
    }
}

/// The text a status segment's view carries, as (caption, value) pairs.
fn flatten_view(view: &BView, out: &mut Vec<(String, String)>) {
    match view {
        BView::Pair { label, value } => out.push((label.clone(), value.clone())),
        BView::Label { text } => out.push((String::new(), text.clone())),
        BView::Row { list } => list.iter().for_each(|v| flatten_view(v, out)),
        _ => {}
    }
}

fn menu(f: &mut Fig, x: f32, y: f32, items: &[BMenuItem], mark: &str) {
    let w = items.iter().map(|i| width(i.label) + width(i.shortcut) + 56.0).fold(200.0f32, f32::max);
    let h: f32 = items.iter().map(|i| if matches!(i.kind, BMenuKind::Sep) { 9.0 } else { ROW }).sum::<f32>() + 8.0;
    f.framed(x, y, w, h, "panel", 7.0);
    let mut cy = y + 4.0;
    for item in items {
        if matches!(item.kind, BMenuKind::Sep) {
            f.line(x + 8.0, cy + 4.5, x + w - 8.0, cy + 4.5, "sep");
            cy += 9.0;
            continue;
        }
        let on = item.label == mark;
        if on {
            f.rect(x + 4.0, cy, w - 8.0, ROW, "accent", 5.0);
            f.target = Some((x + w - 4.0, cy + ROW / 2.0));
        }
        f.text(x + 14.0, cy + 15.0, item.label, if on { "on-accent" } else { "text" }, on);
        let tail = if matches!(item.kind, BMenuKind::Sub(_)) { "›" } else { item.shortcut };
        if !tail.is_empty() {
            f.text(x + w - 14.0 - width(tail), cy + 15.0, tail, if on { "on-accent" } else { "dim" }, false);
        }
        cy += ROW;
    }
}

fn browser(f: &mut Fig, purpose: BFilePurpose) {
    let (bw, bh) = (780.0, 470.0);
    let (bx, by) = ((W - bw) / 2.0, (H - bh) / 2.0);
    let _ = writeln!(f.svg, r#"<rect x="0" y="0" width="{W}" height="{H}" class="f-scrim" opacity="0.6"/>"#);
    f.framed(bx, by, bw, bh, "panel", 9.0);
    f.rect(bx + 1.0, by + 1.0, bw - 2.0, 32.0, "header", 9.0);
    f.text(bx + 16.0, by + 21.0, purpose.title(), "text", true);
    f.framed(bx + 12.0, by + 44.0, 190.0, bh - 100.0, "control", 6.0);
    // The sidebar's System places (`part::file::set::SYSTEM`).
    for (k, place) in phiber::v2::part::file::set::SYSTEM.iter().take(8).enumerate() {
        f.icon(place.icon, bx + 22.0, by + 56.0 + k as f32 * 26.0, 16.0, "dim");
        f.text(bx + 46.0, by + 69.0 + k as f32 * 26.0, place.label, "text", false);
    }
    f.framed(bx + 212.0, by + 44.0, bw - 224.0, bh - 100.0, "viewport", 6.0);
    f.framed(bx + 12.0, by + bh - 44.0, bw - 260.0, 30.0, "control", 6.0);
    f.text(bx + 24.0, by + bh - 24.0, "untitled.png", "text", false);
    let exec = purpose.exec_label();
    let ex = bx + bw - 236.0;
    f.rect(ex, by + bh - 44.0, width(exec) + 28.0, 30.0, "accent", 6.0);
    f.text(ex + 14.0, by + bh - 24.0, exec, "on-accent", true);
    f.framed(ex + width(exec) + 40.0, by + bh - 44.0, 80.0, 30.0, "control", 6.0);
    f.text(ex + width(exec) + 56.0, by + bh - 24.0, "Cancel", "text", false);
    f.target = Some((ex + (width(exec) + 28.0) / 2.0, by + bh - 44.0));
}

/// A numbered marker, offset from its target, with a leader to it.
fn marker(f: &mut Fig, n: usize, to: (f32, f32)) {
    let dx = if to.0 > W - 200.0 { -90.0 } else { 90.0 };
    let dy = if to.1 < 140.0 { 70.0 } else { -70.0 };
    let at = (to.0 + dx, to.1 + dy);
    let _ = writeln!(f.svg, r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" class="s-accent" stroke-width="2.5"/>"#, at.0, at.1, to.0, to.1);
    let _ = writeln!(f.svg, r#"<circle cx="{:.1}" cy="{:.1}" r="4" class="f-accent"/>"#, to.0, to.1);
    let _ = writeln!(f.svg, r#"<circle cx="{:.1}" cy="{:.1}" r="15" class="f-accent"/>"#, at.0, at.1);
    let _ = writeln!(f.svg, r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" class="f-on-accent b big">{n}</text>"#, at.0, at.1 + 5.0);
}

/// One step's figure: the window in the deck, then its caption.
fn draw(step: &Step, n: usize, world: &World, deck: &BDeck) -> String {
    let dark = palette_in(deck.palette, BThemeMode::Dark);
    let light = palette_in(deck.palette, BThemeMode::Light);
    let mut f = Fig { svg: String::new(), target: None };
    let mut menus = Vec::new();
    let screen = layout(step.space);
    for (kind, default_host, r) in &screen {
        let host = match (kind, step.center) {
            (BAreaType::Center, Some(host)) => host,
            _ => *default_host,
        };
        match host {
            BHostType::Topbar => topbar(&mut f, step, &mut menus),
            BHostType::Activity => rail(&mut f, *r),
            BHostType::Statusbar => statusbar(&mut f, world, step, *r),
            _ => {
                let (x, y, w, h) = *r;
                let canvas = matches!(host, BHostType::View3D | BHostType::Image);
                f.framed(x + 2.0, y + 2.0, w - 4.0, h - 4.0, if canvas { "viewport" } else { "panel" }, 7.0);
                let end = header(&mut f, host, *r, &mut menus);
                match host {
                    BHostType::Outliner => {
                        let before = f.target;
                        outliner(&mut f, world, step.outliner, *r, end);
                        if step.mark != Mark::Outliner {
                            f.target = before;
                        }
                    }
                    BHostType::Buts => inspector(&mut f, *r),
                    BHostType::Image => image_canvas(&mut f, world, step, *r),
                    BHostType::View3D if step.mark == Mark::Viewport => f.target = Some((x + w / 2.0, y + h / 2.0)),
                    _ => {}
                }
            }
        }
    }
    if let Some((host, label, mark)) = step.menu
        && let Some(&(_, _, mx, my)) = menus.iter().find(|(h, l, _, _)| *h == host && *l == label)
        && let Some(spec) = BMenuSpec::list_for(host).iter().find(|m| m.label == label)
    {
        f.rect(mx - 7.0, my - 14.0, width(label) + 14.0, 20.0, "active", 5.0);
        f.text(mx, my, label, "text", true);
        menu(&mut f, mx - 7.0, my + 10.0, spec.items, mark);
    }
    if let Some(purpose) = step.browser {
        browser(&mut f, purpose);
    }
    if let Some(to) = f.target {
        marker(&mut f, n, to);
    }

    // The deck: the face is the window; the cards behind show their edges.
    let layers = deck.layers();
    let face = deck.face();
    let total_h = H / face.3;
    let (fw, fh) = (W, total_h + CAPTION);
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" class="ph-fig" viewBox="0 0 {fw} {fh:.0}" width="{fw}" height="{fh:.0}" role="img" aria-label="{}">"#,
        esc(step.title)
    );
    let _ = write!(
        svg,
        "\n<style>\nsvg.ph-fig{{{}}}\n@media (prefers-color-scheme: light){{svg.ph-fig{{{}}}}}\n",
        declare(dark),
        declare(light)
    );
    for (name, _) in tokens(dark) {
        let _ = write!(svg, ".f-{name}{{fill:var(--ph-{name})}}.s-{name}{{stroke:var(--ph-{name})}}.c-{name}{{color:var(--ph-{name})}}");
    }
    svg += "\nsvg.ph-fig text{font-family:system-ui,-apple-system,'Segoe UI',sans-serif;font-size:12px}.b{font-weight:600}.sm{font-size:10px}.big{font-size:15px}\n</style>\n";
    for (k, (lx, ly, lw, lh)) in layers.iter().enumerate().take(layers.len() - 1) {
        let fill = if k == 0 { "control" } else { "header" };
        let _ = writeln!(svg, r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="12" class="f-{fill}"/>"#, lx * fw, ly * total_h, lw * fw, lh * total_h);
    }
    let _ = writeln!(svg, r#"<g transform="translate(0 {:.1})"><clipPath id="win{n}"><rect width="{W}" height="{H}" rx="10"/></clipPath><g clip-path="url(#win{n})">"#, face.1 * total_h);
    svg += &f.svg;
    svg += "</g></g>\n";
    let cy = total_h + 24.0;
    let _ = writeln!(svg, r#"<text x="8" y="{cy:.1}" class="f-text b big">{n}. {}</text>"#, esc(step.title));
    let _ = writeln!(svg, r#"<text x="8" y="{:.1}" class="f-dim">{}</text>"#, cy + 22.0, esc(step.caption));
    svg += "</svg>\n";
    svg
}

/// The site's switches, as variables on the figure: `data-theme` picks the
/// mode, `data-theme-style` the palette when it names one of ours, and a
/// brand's primary colour is the accent.
fn theme_css(deck: &BDeck) -> String {
    let mut css = String::from("/* Generated by examples/manual_steps.rs: the step figures' colours, from the docs site's theme. */\n");
    for mode in BThemeMode::ALL {
        let _ = writeln!(css, "[data-theme=\"{}\"] svg.ph-fig{{{}}}", mode.slug(), declare(palette_in(deck.palette, mode)));
    }
    for name in BPaletteName::ALL {
        for mode in BThemeMode::ALL {
            let _ = writeln!(css, "[data-theme-style=\"{}\"][data-theme=\"{}\"] svg.ph-fig{{{}}}", slug(name.label()), mode.slug(), declare(palette_in(*name, mode)));
        }
    }
    css += "[data-brand] svg.ph-fig{--ph-accent:var(--phi-color-primary)}\n";
    css
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let deck = BDeck::SILVER;
    let world = World::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/manual/steps");
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("theme.css"), theme_css(&deck))?;

    // Render an image — as `app::op::render`, `render_open`,
    // `part::statusbar`, `part::outliner::status` and `part::file::op` do it.
    let s = |title, caption, center, outliner, bar, menu, browser, mark| Step { title, caption, space: BSpaceType::Layout, center, outliner, bar, menu, browser, mark };
    let (lay, img, view, status) = (None, Some(BHostType::Image), BOutlineMode::ViewLayer, BOutlineMode::Status);
    let render = [
        s("Render ▸ Render Image", "Open Render in the top bar and choose Render Image — or press F12 anywhere.", lay, view, Bar::Idle, Some((BHostType::Topbar, "Render", "Render Image")), None, Mark::None),
        s("The render takes the viewport's area", "The area turns into the Image editor on Render Result. The line across it reads Frame, Time, Remaining, Mem and Sample as Blender's does.", img, view, Bar::Rendering, None, None, Mark::Info),
        s("The status bar carries the render", "Its progress and an X to cancel (Esc does the same), then the render as one button: Status, Sample, Time, Remaining.", img, view, Bar::Rendering, None, None, Mark::StatusButton),
        s("Click it to see the whole event", "The Outliner turns to its Status mode: what the render was asked for, the settings it was given, and how it ended.", img, status, Bar::Done, None, None, Mark::Outliner),
        s("Image ▸ Save As", "Open the Image editor's Image menu and choose Save As… (Shift Alt S).", img, status, Bar::Done, Some((BHostType::Image, "Image", "Save As...")), None, Mark::None),
        s("Name the file and save", "The file browser opens over the window. Type a name and press the button to write the render.", img, status, Bar::Done, None, Some(BFilePurpose::SaveImage), Mark::None),
        s("Esc gives the area back", "Esc (or F11, View Render) returns the area to the editor it was before the render.", lay, status, Bar::Done, None, None, Mark::Viewport),
    ];
    let workflows: [(&str, &[Step]); 1] = [("Render an image", &render)];
    for (name, steps) in workflows {
        let dir = root.join(slug(name));
        std::fs::create_dir_all(&dir)?;
        // A step renamed or dropped leaves no figure behind.
        for old in std::fs::read_dir(&dir)?.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "svg")) {
            std::fs::remove_file(old.path())?;
        }
        let mut page = format!(
            "# {name}\n\n<!-- Generated by examples/manual_steps.rs from the app's own menu, space, editor and status sets. Do not edit by hand. -->\n\n<link rel=\"stylesheet\" href=\"theme.css\">\n\n"
        );
        for (k, step) in steps.iter().enumerate() {
            let file = format!("{:02}-{}.svg", k + 1, slug(step.title));
            std::fs::write(dir.join(&file), draw(step, k + 1, &world, &deck))?;
            let _ = writeln!(page, "## {}. {}\n\n![{}]({}/{file})\n\n{}\n", k + 1, step.title, step.title, slug(name), step.caption);
        }
        std::fs::write(root.join(format!("{}.md", slug(name))), page)?;
        println!("{name}: {} steps", steps.len());
    }
    Ok(())
}
