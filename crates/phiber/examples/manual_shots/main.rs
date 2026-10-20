//! Takes the manual's **step-by-step** figures from the app itself.
//!
//! Each workflow opens Phiber on the manual's scene, on a virtual display,
//! and does what the page tells the reader to do — the menu opened and the
//! entry clicked, the key pressed, the value typed — through `xdotool`.
//! After each step the screen is taken as it stands (`import`), so every
//! figure is the real window with every word it draws: the menus' entries
//! and shortcuts, the Image editor's info line counting real samples, the
//! status bar's event, the Dope Sheet's keys. The marker is placed by
//! reading the screenshot back (`tesseract`) and finding the words the
//! step points at, so a renamed entry moves its marker with it — or stops
//! the run, saying which words it could not find.
//!
//! The screenshot is set as the face of the manual's deck (`BDeck`), and
//! each workflow's page lists its steps with the figure and the caption.
//!
//! Needs, on Linux: Xvfb (or a `DISPLAY`), `xdotool`, ImageMagick,
//! `tesseract`, and a Vulkan driver (Mesa's lavapipe is enough; the render
//! then runs on the CPU, which is why the manual's scene is Blender's
//! factory scene with Max Samples at 128; its Output is `//render/`, so the
//! file browser opens on a folder of its own).
//!
//! Build the app with its X11 window first, then run:
//!   CARGO_TARGET_DIR=target/gui cargo build --release -p phiber --features gpui_platform/x11,gpui/x11
//!   cargo run --release -p phiber --example manual_shots
//! `PHIBER_BIN` names another build of the app; `MANUAL_ONLY=<slug>` takes
//! one workflow's figures (or `figures` the per-screen ones, `figures:<file>`
//! one of those) and leaves the others as they are.
//!
//! The run also takes the manual's per-screen figures
//! (`v2::manual::figures`): each workspace with the editor a page is about
//! ringed by its name.
//!
//! Writes to: crates/phiber/docs/manual/steps/<workflow>/NN-<slug>.png,
//!            crates/phiber/docs/manual/steps/<workflow>.md and README.md,
//!            crates/phiber/docs/manual/figures/<Space>-<editor>.png and
//!            README.md

mod drive;
mod figure;

use drive::{Act, Display, Find, Mark, Region, Res, Session};
use phiber::v2::host::BHostType;
use phiber::v2::part::switcher::set::category;
use phiber::v2::manual::BDeck;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// One step: what is done, then the screen as it stands, the marker on
/// what was pressed, and what the page says about it.
struct Step {
    title: &'static str,
    caption: &'static str,
    acts: Vec<Act>,
    mark: Mark,
}

fn step(title: &'static str, caption: &'static str, acts: Vec<Act>, mark: Mark) -> Step {
    Step { title, caption, acts, mark }
}

// Where things are on the 1920 × 1080 window, for the words that appear in
// more than one place: the top bar's Render is not the Image editor's Render
// Result.
const TOPBAR: Region = Region(0, 0, 1920, 36);
const TOPBAR_MENU: Region = Region(0, 36, 900, 420);
const STATUS: Region = Region(0, 1044, 1920, 1080);
const IMAGE_INFO: Region = Region(70, 80, 1640, 100);
const IMAGE_HEADER_MENUS: Region = Region(300, 45, 360, 78);
const IMAGE_MENU: Region = Region(280, 70, 900, 700);
const OUTLINER: Region = Region(1640, 80, 1920, 540);
/// The file browser's foot: the file name and the button that saves.
const BROWSER_FOOT: Region = Region(480, 1030, 1440, 1078);
const VIEWPORT: Find = Find::At(850, 480);
const ANIM_OBJECT_MENU: Region = Region(840, 45, 920, 78);
const ANIM_MENUS: Region = Region(820, 70, 1640, 700);
const ANIM_VIEWPORT: Find = Find::At(1100, 450);
const DOPE_RULER: Region = Region(720, 705, 1610, 740);
const DOPE_CHANNELS: Region = Region(517, 740, 717, 1000);
/// The Properties' tab column in the Rendering workspace: icons, read by
/// place (Blender's order: Tool, Render, Output, …).
const PROPS_RENDER_TAB: Find = Find::At(1659, 152);
const PROPS_OUTPUT_TAB: Find = Find::At(1659, 192);
const PROPS: Region = Region(1680, 80, 1920, 1040);

fn text(words: &'static str, region: Region) -> Find {
    Find::Text(words, region)
}

/// Render an image from the Layout workspace: F12's menu entry, the render
/// in the area, its event, and saving it.
fn render_an_image() -> Vec<Step> {
    vec![
        step(
            "Render ▸ Render Image",
            "Open Render in the top bar and choose Render Image — or press F12 anywhere.",
            vec![Act::Click(text("Render", TOPBAR)), Act::Hover(text("Render Image", TOPBAR_MENU))],
            Mark::On(text("Render Image", TOPBAR_MENU)),
        ),
        step(
            "The render takes the viewport's area",
            "The area turns into the Image editor on Render Result. The line across it reads Frame, Time, Remaining, Mem and Sample as Blender's does.",
            vec![Act::Click(text("Render Image", TOPBAR_MENU)), Act::Until("Render pace", 600), Act::Pause(8000)],
            Mark::On(text("Frame:1", IMAGE_INFO)),
        ),
        step(
            "The status bar carries the render",
            "Its progress and an X to cancel (Esc does the same), then the render as one button: Status, Sample, Time, Remaining.",
            vec![Act::Pause(1000)],
            Mark::On(text("Rendering", STATUS)),
        ),
        step(
            "Click it to see the whole event",
            "When it has finished, click the render's button: the Outliner turns to its Status mode — what the render was asked for, the settings it was given, and how it ended.",
            vec![Act::Until("Render finished", 1200), Act::Pause(1500), Act::Click(text("Render Image", STATUS)), Act::Pause(2000)],
            Mark::On(text("Render Image", OUTLINER)),
        ),
        step(
            "Image ▸ Save As",
            "Open the Image editor's Image menu and choose Save As… (Shift Alt S).",
            vec![Act::Click(text("Image", IMAGE_HEADER_MENUS)), Act::Hover(text("Save As", IMAGE_MENU))],
            Mark::On(text("Save As", IMAGE_MENU)),
        ),
        step(
            "Name the file and save",
            "The file browser opens over the window, in the Output tab's folder. Type a name in the field at its foot and press Save As Image to write the render.",
            vec![
                Act::Click(text("Save As", IMAGE_MENU)),
                Act::Pause(2000),
                Act::Click(text("Untitled.png", BROWSER_FOOT)),
                Act::Key("ctrl+a", 1),
                Act::Type("cube.png"),
                // On the virtual display a field shows what was typed only
                // at the next key it handles: End, which changes nothing.
                Act::Key("End", 1),
                Act::Pause(800),
            ],
            Mark::On(text("Save As Image", BROWSER_FOOT)),
        ),
        step(
            "Esc gives the area back",
            "Esc (or F11, View Render) returns the area to the editor it was before the render.",
            vec![Act::Click(text("Save As Image", BROWSER_FOOT)), Act::Pause(2000), Act::Hover(VIEWPORT), Act::Key("Escape", 1), Act::Pause(6000)],
            Mark::On(VIEWPORT),
        ),
    ]
}

/// The Rendering workspace: the Image editor on Render Result with the
/// Properties beside it — set the quality and the output, render a frame,
/// render the animation.
fn rendering_workspace() -> Vec<Step> {
    vec![
        step(
            "Open the Rendering workspace",
            "Click Rendering among the workspace tabs (Ctrl Page Down steps through them). The Image editor fills the window on Render Result, the Properties down the right; with no render yet, the scene renders at once.",
            vec![Act::Click(text("Rendering", TOPBAR)), Act::Pause(4000)],
            Mark::On(text("Rendering", TOPBAR)),
        ),
        step(
            "Open the Render tab",
            "The Properties' tabs run down their left edge; the camera back is Render.",
            vec![Act::Until("Render finished", 1200), Act::Click(PROPS_RENDER_TAB), Act::Pause(1500)],
            Mark::On(PROPS_RENDER_TAB),
        ),
        step(
            "Set the samples",
            "Sampling ▸ Render ▸ Max Samples is how many samples a pixel may take; with Adaptive Sampling on, a pixel that settles stops sooner.",
            vec![Act::Pause(500)],
            Mark::On(Find::Below("Max Samples", "Noise", PROPS)),
        ),
        step(
            "Choose the size, the frames and the folder",
            "The Output tab: Format sets Resolution X, Y and %, Frame Range which frames an animation renders, Output where they are written and as what.",
            vec![Act::Click(PROPS_OUTPUT_TAB), Act::Pause(1500)],
            Mark::On(PROPS_OUTPUT_TAB),
        ),
        step(
            "Render ▸ Render Image",
            "F12 renders the frame you are on into Render Result, here in the Image editor.",
            vec![Act::Click(text("Render", TOPBAR)), Act::Hover(text("Render Image", TOPBAR_MENU))],
            Mark::On(text("Render Image", TOPBAR_MENU)),
        ),
        step(
            "Watch it refine",
            "The info line across the Image editor counts the samples and the time left; Esc stops the render where it is.",
            vec![Act::Click(text("Render Image", TOPBAR_MENU)), Act::Until("Render pace", 600), Act::Pause(8000)],
            Mark::On(text("Frame:1", IMAGE_INFO)),
        ),
        step(
            "Render ▸ Render Animation",
            "Ctrl F12 renders Frame Start to End, every Step, and writes each frame to the Output path — or one movie when the File Format is FFmpeg Video.",
            vec![Act::Until("Render finished", 1200), Act::Pause(1500), Act::Click(text("Render", TOPBAR)), Act::Hover(text("Render Animation", TOPBAR_MENU))],
            Mark::On(text("Render Animation", TOPBAR_MENU)),
        ),
    ]
}

/// The Animation workspace: key the Cube twice and play it back.
fn animate_an_object() -> Vec<Step> {
    vec![
        step(
            "Open the Animation workspace",
            "Click Animation among the workspace tabs: the camera's view on the left, the 3D Viewport, the Dope Sheet along the bottom.",
            vec![Act::Click(text("Animation", TOPBAR)), Act::Pause(8000)],
            Mark::On(text("Animation", TOPBAR)),
        ),
        step(
            "Select the Cube",
            "Click the Cube in the Outliner, or in the viewport. Keys go on what is selected.",
            vec![Act::Click(text("Cube", OUTLINER)), Act::Pause(1500)],
            Mark::On(text("Cube", OUTLINER)),
        ),
        step(
            "Object ▸ Animation ▸ Insert Keyframe",
            "With the playhead on frame 1, open Object ▸ Animation and choose Insert Keyframe — or press I over the viewport. Location, Rotation and Scale are keyed.",
            vec![Act::Click(text("Object", ANIM_OBJECT_MENU)), Act::Hover(text("Animation", ANIM_MENUS)), Act::Hover(text("Insert Keyframe", ANIM_MENUS))],
            Mark::On(text("Insert Keyframe", ANIM_MENUS)),
        ),
        step(
            "Move the playhead",
            "Step to frame 60: the Right arrow moves one frame (drag along the Dope Sheet's numbers to go further at once). The key from frame 1 is a diamond on the Cube's row.",
            vec![Act::Click(text("Insert Keyframe", ANIM_MENUS)), Act::Until("Keyed Cube", 30), Act::Hover(ANIM_VIEWPORT), Act::Key("Right", 59), Act::Pause(6000)],
            Mark::On(text("60", DOPE_RULER)),
        ),
        step(
            "Move the Cube and key it again",
            "G moves it — Z locks it to the up axis, 2 types the distance, Enter confirms — and I keys it here. The Cube now has a key at each end of the move.",
            vec![Act::Key("g", 1), Act::Key("z", 1), Act::Type("2"), Act::Key("Return", 1), Act::Key("i", 1), Act::Until("Keyed Cube at frame 60", 30), Act::Pause(6000)],
            Mark::Cross(text("60", DOPE_RULER), text("Cube", DOPE_CHANNELS)),
        ),
        step(
            "Play it",
            "Space plays from the playhead to End and loops; Space again stops (Shift Ctrl Space plays backwards). The Cube rises between its two keys.",
            vec![Act::Key("Left", 30), Act::Pause(4000), Act::Key("space", 1), Act::Pause(1200)],
            Mark::On(text("Playback", Region(517, 1000, 1640, 1036))),
        ),
        step(
            "Render the animation",
            "Render ▸ Render Animation (Ctrl F12) renders every frame of the range to the Output tab's path.",
            vec![Act::Key("space", 1), Act::Pause(1500), Act::Click(text("Render", TOPBAR)), Act::Hover(text("Render Animation", TOPBAR_MENU))],
            Mark::On(text("Render Animation", TOPBAR_MENU)),
        ),
    ]
}

fn slug(s: &str) -> String {
    s.to_lowercase().split(|c: char| !c.is_ascii_alphanumeric()).filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-")
}

/// Do a workflow's steps on a fresh app and write its figures and page.
fn shoot(display: &Display, bin: &Path, scene: &Path, root: &Path, name: &str, steps: &[Step], deck: &BDeck) -> Res<()> {
    let dir = root.join(slug(name));
    std::fs::create_dir_all(&dir)?;
    // A step renamed or dropped leaves no figure behind.
    for old in std::fs::read_dir(&dir)?.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "png" || x == "svg")) {
        std::fs::remove_file(old.path())?;
    }
    let scratch = std::env::temp_dir().join("phiber-manual-shots");
    std::fs::create_dir_all(&scratch)?;
    let mut session = Session::open(display, bin, scene, &scratch)?;
    let mut page = format!(
        "# {name}\n\n<!-- Generated by examples/manual_shots: each figure is the app itself, driven through the step on a virtual display. Do not edit by hand. -->\n\n"
    );
    for (k, step) in steps.iter().enumerate() {
        for act in &step.acts {
            session.act(*act).map_err(|e| format!("{name}, step {} ({}): {e}", k + 1, step.title))?;
        }
        let shot = scratch.join("shot.png");
        session.capture(&shot)?;
        let mark = session.mark(&shot, step.mark).map_err(|e| format!("{name}, step {} ({}): {e}", k + 1, step.title))?;
        let file = format!("{:02}-{}.png", k + 1, slug(step.title));
        figure::deck(&shot, &dir.join(&file), Some(k + 1), mark, deck)?;
        let _ = writeln!(page, "## {}. {}\n\n![{}]({}/{file})\n\n{}\n", k + 1, step.title, step.title, slug(name), step.caption);
        println!("{name} {}: {}", k + 1, step.title);
    }
    std::fs::write(root.join(format!("{}.md", slug(name))), page)?;
    Ok(())
}

/// The editor an area is switched from to show `host` when its workspace
/// does not: the Animation editors take the Dope Sheet's area, every other
/// editor the main 3D Viewport's.
fn swapped_for(host: BHostType) -> BHostType {
    match host {
        BHostType::Ipo | BHostType::Drivers | BHostType::Nla => BHostType::Action,
        _ => BHostType::View3D,
    }
}

/// The manual's per-screen figures (`v2::manual::figures`): each workspace
/// with the editor a page is about ringed by its name in its header — the
/// editor switched into an area first when the workspace does not show it,
/// and switched back after. Preferences is a window of its own, opened from
/// the activity rail's Settings and closed again.
fn shoot_figures(display: &Display, bin: &Path, scene: &Path, out: &Path, deck: &BDeck, one: Option<&str>) -> Res<()> {
    std::fs::create_dir_all(out)?;
    if one.is_none() {
        for old in std::fs::read_dir(out)?.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "png" || x == "svg")) {
            std::fs::remove_file(old.path())?;
        }
    }
    let scratch = std::env::temp_dir().join("phiber-manual-shots");
    std::fs::create_dir_all(&scratch)?;
    let mut session = Session::open(display, bin, scene, &scratch)?;
    let mut index = String::from(
        "# Manual figures\n\n<!-- Generated by examples/manual_shots. Do not edit by hand. -->\n\nOne per screen: Phiber itself at 1920 × 1080 on the manual's scene, the editor the page is about ringed by its name.\n\n",
    );
    let below_top = Region(0, 36, 1920, 1080);
    for (figure, pages) in phiber::v2::manual::figures() {
        let (space, host) = (figure.space, figure.host);
        let label = host.spec().label;
        let name = format!("{}-{}.png", space.slug(), slug(label));
        let _ = writeln!(index, "- `{name}` — {} page(s): {}", pages.len(), pages.iter().map(|n| n.slug).collect::<Vec<_>>().join(", "));
        if one.is_some_and(|one| one != name) {
            continue;
        }
        let named = |l: &'static str| Find::Text(l, if matches!(host, BHostType::Topbar) { TOPBAR } else { below_top });
        // Preferences is the activity rail's Settings panel, headed Settings.
        let ring = match host {
            BHostType::Prefs => Find::Text("Settings", Region(64, 50, 320, 100)),
            _ => named(label),
        };
        let fail = |e: Box<dyn std::error::Error>| format!("figure {} · {label}: {e}", space.label());
        session.act(Act::Workspace(space.label())).map_err(fail)?;
        // Entering Rendering with no render yet renders the scene; the
        // figure waits for it, so none is taken with a render running.
        if session.said("Rendering the scene for the Rendering workspace") {
            session.act(Act::Until("Render finished", 1200)).map_err(fail)?;
        }
        session.act(Act::Pause(5000)).map_err(fail)?;
        let mut restore = None;
        if host == BHostType::Prefs {
            // The rail's Settings — its caption is too faint to read back.
            session.act(Act::Click(Find::At(31, 945))).map_err(fail)?;
            session.act(Act::Pause(2000)).map_err(fail)?;
        } else if !session.shows(named(label)).map_err(fail)? {
            let from = swapped_for(host);
            session.switch_editor(from.spec().label, category(host).label(), label, 36).map_err(fail)?;
            session.act(Act::Pause(5000)).map_err(fail)?;
            restore = Some(from);
        }
        let shot = scratch.join("shot.png");
        session.capture(&shot)?;
        let mark = session.mark(&shot, Mark::On(ring)).map_err(fail)?;
        figure::deck(&shot, &out.join(&name), None, mark, deck)?;
        println!("figure {name}");
        match (host, restore) {
            (BHostType::Prefs, _) => session.act(Act::Click(Find::At(31, 945))).map_err(fail)?,
            (_, Some(from)) => session.switch_editor(label, category(from).label(), from.spec().label, 36).map_err(fail)?,
            (_, None) => {}
        }
    }
    std::fs::write(out.join("README.md"), index)?;
    Ok(())
}

fn main() -> Res<()> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = std::env::var_os("PHIBER_BIN").map_or_else(|| crate_dir.join("../../target/gui/release/phiber"), PathBuf::from);
    let scene = crate_dir.join("docs/manual/data/manual.blend");
    let root = crate_dir.join("docs/manual/steps");
    std::fs::create_dir_all(&root)?;
    // The scene's Output folder (`//render/`), where Save As opens and the
    // render is written; made for the run and taken away after it.
    let renders = scene.with_file_name("render");
    std::fs::create_dir_all(&renders)?;
    let display = Display::open()?;
    let deck = BDeck::SILVER;
    let workflows = [
        ("Render an image", render_an_image()),
        ("Render in the Rendering workspace", rendering_workspace()),
        ("Animate an object", animate_an_object()),
    ];
    let mut index = String::from(
        "# Step by step\n\n<!-- Generated by examples/manual_shots. Do not edit by hand. -->\n\nEach figure is Phiber itself, taken at 1920 × 1080 while the step was done on the manual's scene (`data/manual.blend`: Blender 5.2's factory scene with Max Samples at 128 and Output at `//render/`).\n\n",
    );
    let only = std::env::var("MANUAL_ONLY").ok();
    let mut take = || -> Res<()> {
        // `figures`, or `figures:<file>` for one of them.
        if let Some(one) = only.as_deref().map_or(Some(None), |o| o.strip_prefix("figures").map(|rest| rest.strip_prefix(':'))) {
            shoot_figures(&display, &bin, &scene, &crate_dir.join("docs/manual/figures"), &deck, one)?;
        }
        for (name, steps) in &workflows {
            let _ = writeln!(index, "- [{name}]({}.md) — {} steps", slug(name), steps.len());
            if only.as_ref().is_some_and(|o| *o != slug(name)) {
                continue;
            }
            shoot(&display, &bin, &scene, &root, name, steps, &deck)?;
        }
        Ok(())
    };
    // The render folder goes whether or not every step was taken.
    let taken = take();
    std::fs::remove_dir_all(&renders)?;
    taken?;
    // The drawn figures' colour sheet went with the drawn figures.
    let _ = std::fs::remove_file(root.join("theme.css"));
    std::fs::write(root.join("README.md"), index)?;
    Ok(())
}
