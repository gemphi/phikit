//! The app on a virtual display, driven the way a person drives it: the
//! pointer moved and clicked, keys pressed, text typed — `xdotool` on an X
//! display — and the screen read back as a picture (`import`) and as words
//! (`tesseract`), so a step can say "click Render Image" and the driver
//! finds Render Image where the app drew it.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// The window's size: Blender's own reference screen, and the size every
/// figure is taken at.
pub const SCREEN: (i32, i32) = (1920, 1080);

/// A part of the screen, `(left, top, right, bottom)` in pixels, that a
/// word is looked for in — so "Render" in the top bar is not the "Render"
/// of Render Result in the Image editor's header.
#[derive(Clone, Copy, Debug)]
pub struct Region(pub i32, pub i32, pub i32, pub i32);

/// Where on the screen something is.
#[derive(Clone, Copy, Debug)]
pub enum Find {
    /// The words, as the app draws them, in a region.
    Text(&'static str, Region),
    /// The words, the first time they come below the other words — the
    /// Render subpanel's Max Samples, not the Viewport's above it.
    Below(&'static str, &'static str, Region),
    /// A fixed point — for an icon, which has no words to read.
    At(i32, i32),
}

/// One thing done to the app.
#[derive(Clone, Copy, Debug)]
pub enum Act {
    Click(Find),
    /// The pointer moved there and left: a menu entry lit, a submenu open.
    Hover(Find),
    /// An X key name (`Return`, `F12`, `ctrl+F12`), pressed `n` times.
    Key(&'static str, u32),
    Type(&'static str),
    Pause(u64),
    /// Wait, up to `secs`, for the journal to say `text` after the last
    /// thing pressed or clicked.
    Until(&'static str, u64),
    /// The workspace of that label, stepped to with Ctrl Page Down as the
    /// tabs are (the top bar may not have room to show every tab).
    Workspace(&'static str),
}

/// Where a step's marker points.
#[derive(Clone, Copy, Debug)]
pub enum Mark {
    On(Find),
    /// The column of the first and the row of the second: a key in the Dope
    /// Sheet is where its frame's column meets its channel's row.
    Cross(Find, Find),
}

/// A word the screen showed, and where.
#[derive(Clone, Debug)]
pub struct Word {
    text: String,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

/// The X display the app is shown on: the caller's when `DISPLAY` is set,
/// else a virtual one started for the run and stopped after it.
pub struct Display {
    name: String,
    server: Option<Child>,
}

impl Display {
    pub fn open() -> Res<Self> {
        if let Ok(name) = std::env::var("DISPLAY") {
            return Ok(Self { name, server: None });
        }
        let name = ":91".to_string();
        let server = Command::new("Xvfb")
            .args([name.as_str(), "-screen", "0", &format!("{}x{}x24", SCREEN.0, SCREEN.1)])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Xvfb: {e} — install it, or run with DISPLAY set"))?;
        std::thread::sleep(Duration::from_secs(2));
        Ok(Self { name, server: Some(server) })
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        if let Some(server) = &mut self.server {
            let _ = server.kill();
        }
    }
}

/// The app, open on a file, and what has been read off it.
pub struct Session<'a> {
    display: &'a Display,
    app: Child,
    journal: PathBuf,
    scratch: PathBuf,
    /// The journal's length when the last key was pressed or click made.
    since: usize,
    /// Where the last click was, so a click the window missed can be made
    /// again (`Self::locate_after`).
    last_click: Option<(i32, i32)>,
}

impl<'a> Session<'a> {
    /// Launch `bin` on `file`, place its window at the screen's size and
    /// wait for the viewport's first settled image.
    pub fn open(display: &'a Display, bin: &Path, file: &Path, scratch: &Path) -> Res<Self> {
        let journal = bin.with_file_name("phiber-session.log");
        let _ = std::fs::remove_file(&journal);
        let app = Command::new(bin)
            .arg(file)
            .env("DISPLAY", &display.name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("{}: {e}", bin.display()))?;
        let session = Self { display, app, journal, scratch: scratch.to_path_buf(), since: 0, last_click: None };
        let window = session.window()?;
        session.xdo(&["windowmove", &window, "0", "0"])?;
        session.xdo(&["windowsize", &window, &SCREEN.0.to_string(), &SCREEN.1.to_string()])?;
        session.wait_for("view settled", 300)?;
        std::thread::sleep(Duration::from_secs(2));
        Ok(session)
    }

    fn window(&self) -> Res<String> {
        let began = Instant::now();
        while began.elapsed() < Duration::from_secs(60) {
            let out = Command::new("xdotool").args(["search", "--name", "Phiber"]).env("DISPLAY", &self.display.name).output()?;
            if let Some(id) = String::from_utf8_lossy(&out.stdout).lines().next() {
                return Ok(id.to_string());
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        Err("the window never opened".into())
    }

    fn xdo(&self, args: &[&str]) -> Res<()> {
        let status = Command::new("xdotool").args(args).env("DISPLAY", &self.display.name).status()?;
        match status.success() {
            true => Ok(()),
            false => Err(format!("xdotool {args:?} failed").into()),
        }
    }

    fn journal(&self) -> String {
        std::fs::read_to_string(&self.journal).unwrap_or_default()
    }

    /// Wait for the journal to say `text`, the window kept awake: with no
    /// compositor it presents a frame — and the app's polls run — only when
    /// an event arrives, so an untouched window sits on its first black
    /// frame.
    fn wait_for(&self, text: &str, secs: u64) -> Res<()> {
        let began = Instant::now();
        while began.elapsed() < Duration::from_secs(secs) {
            if self.journal().get(self.since..).is_some_and(|tail| tail.contains(text)) {
                return Ok(());
            }
            self.idle(Duration::from_millis(500))?;
        }
        Err(format!("waited {secs}s for the journal to say '{text}'").into())
    }

    /// Let `time` pass with the window awake: the pointer nudged a pixel out
    /// and back every half second (see [`Self::wait_for`]).
    fn idle(&self, time: Duration) -> Res<()> {
        let began = Instant::now();
        loop {
            self.xdo(&["mousemove_relative", "--", "1", "0"])?;
            self.xdo(&["mousemove_relative", "--", "-1", "0"])?;
            let left = time.saturating_sub(began.elapsed());
            if left.is_zero() {
                return Ok(());
            }
            std::thread::sleep(left.min(Duration::from_millis(500)));
        }
    }

    /// Move the pointer to `to` the way a hand does — across, in steps, and
    /// then still for a moment. A pointer that jumps there in one warp is
    /// seen by the window as one motion, which can land before a frame has
    /// placed the hover there: a menu entry does not light and a press that
    /// follows is lost.
    fn glide(&self, to: (i32, i32)) -> Res<()> {
        let out = Command::new("xdotool").args(["getmouselocation", "--shell"]).env("DISPLAY", &self.display.name).output()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let at = |k: &str| text.lines().find_map(|l| l.strip_prefix(k)?.parse::<i32>().ok());
        let from = (at("X=").unwrap_or(to.0), at("Y=").unwrap_or(to.1));
        const STEPS: i32 = 8;
        for i in 1..=STEPS {
            let (x, y) = (from.0 + (to.0 - from.0) * i / STEPS, from.1 + (to.1 - from.1) * i / STEPS);
            self.xdo(&["mousemove", &x.to_string(), &y.to_string()])?;
            std::thread::sleep(Duration::from_millis(40));
        }
        self.idle(Duration::from_millis(500))
    }

    /// The screen as it stands, saved to `to`.
    pub fn capture(&self, to: &Path) -> Res<()> {
        self.idle(Duration::from_millis(300))?;
        let status = Command::new("import").args(["-window", "root"]).arg(to).env("DISPLAY", &self.display.name).status()?;
        match status.success() {
            true => Ok(()),
            false => Err("import could not read the screen".into()),
        }
    }

    /// Where `find` is on the screen as it stands now.
    fn locate_now(&self, find: Find) -> Res<(i32, i32)> {
        match find {
            Find::At(x, y) => Ok((x, y)),
            Find::Text(..) | Find::Below(..) => {
                let look = self.scratch.join("look.png");
                self.capture(&look)?;
                self.locate(&look, find)
            }
        }
    }

    /// Where `find` is in the screenshot `shot`.
    pub fn locate(&self, shot: &Path, find: Find) -> Res<(i32, i32)> {
        match find {
            Find::At(x, y) => Ok((x, y)),
            Find::Text(text, region) => locate(&read_words(shot, region, &self.scratch)?, text, region),
            Find::Below(text, anchor, region) => {
                let words = read_words(shot, region, &self.scratch)?;
                let (_, under) = locate(&words, anchor, region)?;
                let below: Vec<Word> = words.into_iter().filter(|w| w.y > under).collect();
                locate(&below, text, region)
            }
        }
    }

    /// Press and release at `at`, the press held: a click is a press and a
    /// release on the same element, and gpui arms the release only once it
    /// has painted a frame after the press — and a window with no compositor
    /// paints when events arrive. So the press is held, with the pointer
    /// stirring by a pixel, until a frame has surely gone by.
    fn press(&mut self, at: (i32, i32)) -> Res<()> {
        self.glide(at)?;
        self.since = self.journal().len();
        self.xdo(&["mousedown", "1"])?;
        self.idle(Duration::from_millis(900))?;
        self.xdo(&["mouseup", "1"])?;
        self.last_click = Some(at);
        self.idle(Duration::from_millis(700))
    }

    /// Where an entry to point at is now. When it cannot be found and a
    /// click went just before, that click is made once more and the screen
    /// read again: a click the window missed leaves the menu it opens
    /// unopened. Only for pointing — a click retried blind where something
    /// else now stands would press that instead.
    fn locate_after(&mut self, find: Find) -> Res<(i32, i32)> {
        match (self.locate_now(find), self.last_click.take()) {
            (Ok(at), _) => Ok(at),
            (Err(_), Some(again)) => {
                println!("  again at {},{}", again.0, again.1);
                self.press(again)?;
                self.locate_now(find)
            }
            (Err(e), None) => Err(e),
        }
    }

    pub fn act(&mut self, act: Act) -> Res<()> {
        let pressed = |s: &mut Self| {
            s.since = s.journal().len();
            s.last_click = None;
        };
        match act {
            Act::Click(find) => {
                let at = self.locate_now(find)?;
                println!("  click {find:?} at {},{}", at.0, at.1);
                self.press(at)?;
            }
            Act::Hover(find) => {
                let (x, y) = self.locate_after(find)?;
                println!("  hover {find:?} at {x},{y}");
                self.glide((x, y))?;
                self.idle(Duration::from_millis(700))?;
            }
            Act::Key(key, n) => {
                pressed(self);
                self.xdo(&["key", "--delay", "60", "--repeat", &n.to_string(), key])?;
                std::thread::sleep(Duration::from_millis(400));
            }
            Act::Type(text) => {
                pressed(self);
                self.xdo(&["type", "--delay", "60", text])?;
                std::thread::sleep(Duration::from_millis(400));
            }
            Act::Pause(ms) => self.idle(Duration::from_millis(ms))?,
            Act::Workspace(label) => self.workspace(label)?,
            Act::Until(text, secs) => self.wait_for(text, secs)?,
        }
        Ok(())
    }

    /// Step the workspace tabs to `label`, the way that does not pass over
    /// Rendering — entering it renders the scene (`render_on_entry`), and a
    /// render nobody asked for would run under every figure after it. Where
    /// the session stands is read from the journal's last "Switched to".
    fn workspace(&mut self, label: &str) -> Res<()> {
        use phiber::v2::space::BSpaceType;
        let all = BSpaceType::ALL;
        let journal = self.journal();
        let here = match journal.contains("Switched to ") {
            true => journal.rsplit("Switched to ").next().and_then(|tail| tail.lines().next()).unwrap_or(BSpaceType::Layout.label()),
            false => BSpaceType::Layout.label(),
        };
        let index = |l: &str| all.iter().position(|s| s.label() == l);
        let (from, to) = (index(here).ok_or(format!("no workspace '{here}'"))?, index(label).ok_or(format!("no workspace '{label}'"))?);
        let n = all.len();
        let forward = (to + n - from) % n;
        let rendering = index(BSpaceType::Rendering.label()).unwrap_or(n);
        let passes = |step: usize, dir: isize| (1..step).any(|k| (from as isize + dir * k as isize).rem_euclid(n as isize) as usize == rendering);
        let (key, dir, steps) = match passes(forward, 1) {
            true => ("ctrl+Prior", -1, (n - forward) % n),
            false => ("ctrl+Next", 1, forward),
        };
        // One step at a time, each waited for: a workspace that builds
        // slowly (a first viewport trace) holds the next key back.
        for k in 1..=steps {
            let next = all[(from as isize + dir * k as isize).rem_euclid(n as isize) as usize].label();
            self.since = self.journal().len();
            self.xdo(&["key", key])?;
            self.wait_for(&format!("Switched to {next}"), 180)?;
        }
        Ok(())
    }

    /// Switch the area whose editor is named `from` (the first such header,
    /// below `top`) to the editor named `to`, through its editor-type popover:
    /// the name pressed, the `category` picked in the popover's left column,
    /// the editor in its right.
    pub fn switch_editor(&mut self, from: &'static str, category: &'static str, to: &'static str, top: i32) -> Res<()> {
        let (x, y) = self.locate_now(Find::Text(from, Region(0, top, 1920, 1080)))?;
        self.act(Act::Click(Find::At(x, y)))?;
        let popover = Region((x - 120).max(0), y + 12, (x + 420).min(1920), (y + 420).min(1080));
        self.act(Act::Click(Find::Text(category, popover)))?;
        self.act(Act::Click(Find::Text(to, popover)))
    }

    /// Whether the journal has said `text` since the last key or click.
    pub fn said(&self, text: &str) -> bool {
        self.journal().get(self.since..).is_some_and(|tail| tail.contains(text))
    }

    /// Whether `find` is on the screen as it stands.
    pub fn shows(&self, find: Find) -> Res<bool> {
        Ok(self.locate_now(find).is_ok())
    }

    /// Where the marker goes on the screenshot `shot`.
    pub fn mark(&self, shot: &Path, mark: Mark) -> Res<Option<(i32, i32)>> {
        match mark {
            Mark::On(find) => self.locate(shot, find).map(Some),
            Mark::Cross(column, row) => Ok(Some((self.locate(shot, column)?.0, self.locate(shot, row)?.1))),
        }
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        let _ = self.app.kill();
        let _ = self.app.wait();
    }
}

/// A word as compared: its letters and digits alone, so "As…" and "As..."
/// are the same word and a stray stroke on either side does not count.
fn bare(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// The words in `region` of a screenshot. The UI's small type reads far
/// better three times the size, and no one contrast reads every surface —
/// grey keeps text on the light panels, a threshold keeps the dark text on a
/// selected row's tint, a negative keeps white text on the accent — so the
/// region is read several ways and the words pooled.
fn read_words(shot: &Path, region: Region, scratch: &Path) -> Res<Vec<Word>> {
    const SCALE: i32 = 3;
    const BORDER: i32 = 30;
    let crop = format!("{}x{}+{}+{}", region.2 - region.0, region.3 - region.1, region.0, region.1);
    let mut words = Vec::new();
    // Each contrast with tesseract's page mode for it: sparse text finds
    // words scattered over a panel, but reads nothing at all in a thin strip
    // that holds a long framed field — the file browser's foot — where a
    // block of text (`6`) reads every word.
    let reads: [(&[&str], &str); 5] = [
        (&["-colorspace", "gray"], "11"),
        (&["-colorspace", "gray"], "6"),
        (&["-colorspace", "gray", "-threshold", "45%"], "11"),
        (&["-colorspace", "gray", "-threshold", "55%"], "11"),
        // White on the accent — the playhead's frame, a checked box.
        (&["-colorspace", "gray", "-negate", "-threshold", "40%"], "11"),
    ];
    for (contrast, mode) in reads {
        let big = scratch.join("ocr.png");
        let status = Command::new("convert")
            .arg(shot)
            .args(["-crop", &crop, "+repage"])
            .args(contrast)
            .args(["-resize", &format!("{}%", SCALE * 100)])
            // A margin round the crop: a word flush with the edge of a thin
            // strip is not read at all.
            .args(["-bordercolor", "white", "-border", &BORDER.to_string()])
            .arg(&big)
            .status()?;
        if !status.success() {
            return Err("convert could not read the screenshot".into());
        }
        let out = Command::new("tesseract").arg(&big).args(["-", "--psm", mode, "tsv"]).stderr(Stdio::null()).output()?;
        let tsv = String::from_utf8_lossy(&out.stdout);
        words.extend(tsv.lines().skip(1).filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            let num = |i: usize| f.get(i).and_then(|v| v.parse::<i32>().ok());
            let text = f.get(11)?.trim();
            if bare(text).is_empty() {
                return None;
            }
            Some(Word { text: text.to_string(), x: region.0 + (num(6)? - BORDER) / SCALE, y: region.1 + (num(7)? - BORDER) / SCALE, w: num(8)? / SCALE, h: num(9)? / SCALE })
        }));
    }
    Ok(words)
}

/// The centre of the first run of words that reads `find`'s text, in
/// reading order, among the words read from its region: each next word on the same line and
/// just to the right of the one before.
fn locate(words: &[Word], text: &str, region: Region) -> Res<(i32, i32)> {
    let wanted: Vec<String> = text.split_whitespace().map(bare).filter(|w| !w.is_empty()).collect();
    let mut runs: Vec<(i32, i32, i32, i32)> = Vec::new();
    for start in words.iter().filter(|w| bare(&w.text) == wanted[0]) {
        let (left, mut top, mut right, mut bottom) = (start.x, start.y, start.x + start.w, start.y + start.h);
        let mut whole = true;
        for next in &wanted[1..] {
            let follow = words.iter().find(|w| bare(&w.text) == *next && (w.y - top).abs() < 8 && w.x > right - 2 && w.x - right < 24);
            match follow {
                Some(w) => {
                    right = w.x + w.w;
                    top = top.min(w.y);
                    bottom = bottom.max(w.y + w.h);
                }
                None => {
                    whole = false;
                    break;
                }
            }
        }
        if whole {
            runs.push((left, top, right, bottom));
        }
    }
    runs.sort_by_key(|&(left, top, ..)| (top / 8, left));
    runs.first()
        .map(|&(left, top, right, bottom)| ((left + right) / 2, (top + bottom) / 2))
        .ok_or_else(|| format!("'{text}' is not on the screen in {region:?}").into())
}
