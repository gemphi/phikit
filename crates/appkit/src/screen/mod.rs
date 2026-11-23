//! The screen: an app's stage in a real window. This folder is the one
//! place in appkit that names gpui; `pui` draws on a sheet and never
//! learns what shows it. The window is opened as gpui's `hello_world`
//! example opens one (`gpu/examples/hello_world.rs`), and gpui picks the
//! system from the target: Windows, macOS, Linux or the web.
//!
//! Where the app's theme gives the window a caption, on Windows the
//! system's own title bar is left out and the caption stands in its
//! place: the screen tells the system where on it the window is moved
//! by and where its three buttons are. On the other systems the
//! system's bar stays, over the caption.
//!
//! The window's size decides the mode: narrowed below the width
//! `data/mode.tsv` gives, the same stage is laid out as a mobile one.

pub mod album;
pub mod errand;
pub mod key;
pub(crate) mod play;
mod source;

use std::borrow::Cow;
use std::rc::Rc;

use gpui::{App, Bounds, Context, FocusHandle, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render, ScrollWheelEvent, TitlebarOptions, Window, WindowBackgroundAppearance, WindowBounds, WindowControlArea, WindowOptions, canvas, div, prelude::*, px, size};

use crate::pui::core::{PAsset, PEvent, PGlass, PMode, PPoint, PSheet, PSize};
use crate::pui::window::{PCaption, PGrip, PStage};
use play::Type;

/// The theme's tokens for the window a stage opens in.
pub const WIDTH: &str = "window.width";
pub const HEIGHT: &str = "window.height";
/// The word tokens for the typeface: its family, and its file among the
/// app's assets.
pub const FACE: &str = "text.face";
pub const FILE: &str = "text.file";

/// A stage in a window.
pub struct PScreen {
    stage: PStage,
    focus: FocusHandle,
    /// Whether the window was last told to let what is behind it show.
    glass: Option<bool>,
    /// The pictures it has shown.
    album: album::Album,
}

impl PScreen {
    fn press(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let at = PPoint::new(event.position.x.as_f32(), event.position.y.as_f32());
        self.stage.send(&PEvent::PointerDown(at));
        cx.notify();
    }

    /// The button let go: what was held down is held no more.
    fn lift(&mut self, event: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        let at = PPoint::new(event.position.x.as_f32(), event.position.y.as_f32());
        if self.stage.send(&PEvent::PointerUp(at)) {
            cx.notify();
        }
    }

    /// Most parts take no notice of where the pointer is: the window is
    /// drawn again only for one that did.
    fn hover(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        let at = PPoint::new(event.position.x.as_f32(), event.position.y.as_f32());
        if self.stage.send(&PEvent::PointerMove(at)) {
            cx.notify();
        }
    }

    fn wheel(&mut self, event: &ScrollWheelEvent, window: &mut Window, cx: &mut Context<Self>) {
        let (at, by) = (event.position, event.delta.pixel_delta(window.line_height()));
        self.stage.send(&PEvent::Scroll(PPoint::new(at.x.as_f32(), at.y.as_f32()), PPoint::new(by.x.as_f32(), by.y.as_f32())));
        cx.notify();
    }

    fn key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let stroke = &event.keystroke;
        // Control with Alt is how some keyboards write a letter, not a chord.
        let chord = stroke.modifiers.secondary() && !stroke.modifiers.alt;
        let typed = key::typed(&stroke.key, stroke.key_char.as_deref(), chord, || cx.read_from_clipboard().and_then(|item| item.text()));
        typed.into_iter().for_each(|key| _ = self.stage.send(&PEvent::KeyDown(key)));
        cx.notify();
    }
}

impl Render for PScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = window.viewport_size();
        let size = PSize { width: view.width.as_f32(), height: view.height.as_f32() };
        let look = Rc::clone(self.stage.theme());
        // A look with a window of glass has the system blur what is
        // behind the window; the window is told when the look changes.
        let glass = PGlass::window(look.as_ref());
        if self.glass.replace(glass) != Some(glass) {
            window.set_background_appearance(match glass {
                true => WindowBackgroundAppearance::Blurred,
                false => WindowBackgroundAppearance::Opaque,
            });
        }
        // What the parts asked for that only the system can do is done
        // now: every event that could ask is followed by a frame.
        self.stage.errand().into_iter().for_each(|deed| errand::run(deed, cx));
        // While a view has more to come by itself, another frame follows.
        if self.stage.tick() {
            window.request_animation_frame();
        }
        self.stage.lay(size, PMode::of_width(size.width));
        let face = Type::of(look.as_ref());
        let gauge = |text: &str| face.width(text, window);
        let album = self.album.clone();
        let known = |name: &str| album.size(name).map(|(width, height)| PSize { width, height });
        let mut sheet = PSheet::gauged(look.as_ref(), &gauge).sized(&known);
        self.stage.draw(&mut sheet);
        let mark = sheet.mark;
        let paint = move |window: &mut Window, cx: &mut App| {
            // A frame of a moving picture that is no longer shown is let
            // go: the renderer keeps what it was given until told.
            album.stale().into_iter().for_each(|old| _ = window.drop_image(old));
            play::play(&mark, &face, &album, window, cx);
            // A picture still being decoded is drawn in a frame to come,
            // and one that moves has another frame coming.
            if album.waiting() || album.moving() {
                window.request_animation_frame();
            }
        };
        let paint = canvas(|_, _, _| {}, move |_, _, window, cx| paint(window, cx)).size_full();
        // Over the picture, the places on the caption that are the
        // system's: a press there never reaches the stage.
        let grip = self.stage.grip().into_iter().map(|(rect, grip)| {
            let area = match grip {
                PGrip::Drag => WindowControlArea::Drag,
                PGrip::Min => WindowControlArea::Min,
                PGrip::Max => WindowControlArea::Max,
                PGrip::Close => WindowControlArea::Close,
            };
            div().absolute().left(px(rect.origin.x)).top(px(rect.origin.y)).w(px(rect.size.width)).h(px(rect.size.height)).window_control_area(area)
        });
        // And a hand where a press does something.
        let hand = self.stage.hand().into_iter().map(|rect| div().absolute().left(px(rect.origin.x)).top(px(rect.origin.y)).w(px(rect.size.width)).h(px(rect.size.height)).cursor_pointer());
        div().size_full().relative().track_focus(&self.focus).on_mouse_down(MouseButton::Left, cx.listener(Self::press)).on_mouse_up(MouseButton::Left, cx.listener(Self::lift)).on_mouse_move(cx.listener(Self::hover)).on_scroll_wheel(cx.listener(Self::wheel)).on_key_down(cx.listener(Self::key)).child(paint).children(hand).children(grip)
    }
}

/// Say that something went wrong: in the log, which a browser shows in its
/// console, and at a desk on stderr as well.
pub(crate) fn warn(text: &str) {
    log::error!("{text}");
    #[cfg(not(target_family = "wasm"))]
    eprintln!("{text}");
}

/// What a browser needs before an app runs in it: its panics and its log
/// in the console. At a desk there is nothing to do.
pub fn init() {
    #[cfg(target_family = "wasm")]
    gpui_platform::web_init();
}

/// Open a stage in a window, in its app's theme, with these assets, and
/// run until the window is closed.
pub fn run(stage: PStage, asset: impl PAsset + Send + Sync + 'static) {
    let look = Rc::clone(stage.theme());
    let face = look.word(FILE).and_then(|path| asset.load(path));
    let film = asset.film();
    // A window with a caption has no bar of the system's, where the
    // caption can do all that bar does: on Windows.
    let bare = cfg!(target_os = "windows") && PCaption::high(look.as_ref()) > 0.0;
    let stage = stage.bare(bare).lively();
    let launch = move |cx: &mut App| {
        // A family the text system does not know is drawn in a fallback
        // face, so its file is registered before the window opens.
        let added = face.map(|bytes| cx.text_system().add_fonts(vec![Cow::Owned(bytes)]));
        added.and_then(Result::err).into_iter().for_each(|error| warn(&format!("the typeface could not be registered: {error:#}")));
        let bounds = Bounds::centered(None, size(px(look.size(WIDTH).unwrap_or_default()), px(look.size(HEIGHT).unwrap_or_default())), cx);
        let titlebar = TitlebarOptions { title: Some(stage.window.title.clone().into()), appears_transparent: bare, ..Default::default() };
        let option = WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), titlebar: Some(titlebar), ..Default::default() };
        let opened = cx.open_window(option, move |window, cx| {
            cx.new(|cx| {
                let focus = cx.focus_handle();
                window.focus(&focus, cx);
                PScreen { stage, focus, glass: None, album: album::Album::of(film) }
            })
        });
        // With no window there is nothing to run: say why, the whole of it,
        // and stop.
        match opened {
            Ok(_) => cx.activate(true),
            Err(error) => {
                warn(&format!("the window could not be opened: {error:#}"));
                cx.quit();
            }
        }
    };
    // At a desk `run` stays in the system's loop until the app is quit.
    #[cfg(not(target_family = "wasm"))]
    gpui_platform::application().with_assets(source::Source(asset)).run(launch);
    // In a browser the loop is the page's: launching returns at once, and
    // the app lives as long as its handle is kept, which is as long as the
    // page. Dropped, the app and its canvas would go with it. There is one
    // thread there, so the platform is the single-threaded one.
    #[cfg(target_family = "wasm")]
    KEPT.with(|kept| *kept.borrow_mut() = Some(gpui_platform::single_threaded_web().with_assets(source::Source(asset)).run_embedded(launch)));
}

#[cfg(target_family = "wasm")]
thread_local! {
    /// The app running in the page.
    static KEPT: std::cell::RefCell<Option<gpui::ApplicationHandle>> = const { std::cell::RefCell::new(None) };
}
