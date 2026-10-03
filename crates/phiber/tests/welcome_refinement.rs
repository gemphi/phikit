//! Focused regressions for viewport-filling Welcome and current modal forms.
//!
//! Validates: Requirements 3.4, 4.4, 4.5, 10.1, 11.1, 11.5-11.7, 12.7, 14.7-14.12

use phiber::v2::state::{BKey as _, CtxKey, ModalKey};
use phiber::v2::{
    modal::{BModalType, splash::BSplashModal},
    state::{
        BState,
        val::V_MODAL_FILE_BROWSER,
    },
    view::{BAction, BView, BViewItem},
    view::prim::{BCardKind, BStrip},
};
use std::{fs, path::PathBuf};

fn crate_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn source(relative: &str) -> String {
    let path = module_file(crate_path(relative));
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn visit<'a>(
    view: &'a BView,
    strings: &mut Vec<&'a str>,
    icons: &mut Vec<&'a str>,
    actions: &mut Vec<&'a BAction>,
) {
    match view {
        BView::Label { text } | BView::VLabel { text } | BView::Field { text } => {
            strings.push(text)
        }
        BView::Icon { name } => icons.push(name),
        BView::Button(ctl) => {
            if let Some(icon) = &ctl.icon {
                icons.push(icon);
            }
            if let Some(label) = &ctl.label {
                strings.push(label);
            }
            if let Some(action) = &ctl.action {
                actions.push(action);
            }
        }
        BView::Card(card) => {
            if let Some(title) = card.title() {
                strings.push(title);
            }
            for child in card.body.iter().chain(&card.foot) {
                visit(child, strings, icons, actions);
            }
        }
        BView::Row { list }
        | BView::Column { list }
        | BView::ListRow { list, .. } => {
            for child in list {
                visit(child, strings, icons, actions);
            }
        }
        _ => {}
    }
}

#[test]
fn welcome_uses_phiber_brand_icon_and_one_aligned_content_header() {
    let view = BSplashModal.build(&BState::new());
    let BView::Column { list } = &view else {
        panic!("Welcome body must be one column")
    };
    // Blender `WM_MT_splash`: banner, then two splits (New File | Recent
    // Files / Getting Started) and (Open… + Recover | What's New + support).
    assert_eq!(list.len(), 3);
    assert!(matches!(
        &list[0],
        BView::Card(card) if card.kind == BCardKind::Strip(BStrip::Header)
            && matches!(card.body.as_slice(), [_, BView::Icon { name }, BView::Icon { name: word }, _]
                if name == "phiber_banner" && word == "phiber_wordmark_banner")
    ));
    for split in &list[1..] {
        let BView::Row { list: cards } = split else { panic!("splash splits are rows") };
        assert_eq!(cards.len(), 2, "each split has two cards");
        assert!(cards.iter().all(|c| matches!(c, BView::Card(card) if card.fold.is_none())),
            "every splash block is one card that does not fold");
    }

    let mut strings = Vec::new();
    let mut icons = Vec::new();
    let mut actions = Vec::new();
    visit(&view, &mut strings, &mut icons, &mut actions);
    assert!(
        strings
            .iter()
            .all(|text| !text.to_ascii_lowercase().contains("blender"))
    );
    // The banner sets the mark and the name at the header's size, in a strip
    // of the header's height: no larger than any other heading.
    assert!(icons.contains(&"phiber_banner") && icons.contains(&"phiber_wordmark_banner"));
    assert_eq!(BModalType::Splash.spec().label, "Welcome to Phiber");
    assert_eq!(BModalType::Splash.spec().icon, "phiber");
    for asset in ["phiber", "phiber_splash", "phiber_warm", "phiber_cool", "phiber_line", "phiber_join"] {
        assert!(crate_path(&format!("assets/icons/{asset}.svg")).is_file(), "{asset}");
    }
    assert!(crate_path("assets/brand/phiber_wordmark.svg").is_file());
}

#[test]
fn welcome_preserves_template_open_and_link_actions() {
    let view = BSplashModal.build(&BState::new());
    let mut strings = Vec::new();
    let mut icons = Vec::new();
    let mut actions = Vec::new();
    visit(&view, &mut strings, &mut icons, &mut actions);

    for target in [
        "Layout",
        "Shading",
        "UvEditing",
        "Animation",
        "Rendering",
        "Compositing",
        "GeometryNodes",
        "Scripting",
    ] {
        assert!(
            actions.iter().copied().any(|action| matches!(
                action, BAction::Set(path, value) if path == &CtxKey::Space.path() && value == target
            )),
            "missing template action for {target}"
        );
    }
    assert!(actions.iter().copied().any(|action| matches!(
        action,
        BAction::Set(path, value) if path == &ModalKey::Open.path() && value == V_MODAL_FILE_BROWSER
    )));
    // Empty recents ⇒ "Getting Started" (5 links) + What's New / Support (2).
    // Every one but the Phiber website reads in the app: it opens a document
    // in the reader (`ctx.reader.doc`); only Phiber's own site leaves it.
    let entries: Vec<_> = phiber::v2::modal::splash::GETTING_STARTED
        .iter()
        .chain(phiber::v2::modal::splash::FOOT_LINKS)
        .collect();
    let leaving = entries.iter().filter(|e| e.target.starts_with("https://phiace.dev")).count();
    assert_eq!(
        actions
            .iter()
            .copied()
            .filter(|action| matches!(action, BAction::Set(path, _) if path.encode() == "ctx.link"))
            .count(),
        leaving
    );
    assert_eq!(
        actions
            .iter()
            .copied()
            .filter(|action| matches!(action, BAction::Set(path, _) if path.encode() == "ctx.reader.doc"))
            .count(),
        entries.len() - leaving
    );
}

#[test]
fn welcome_renderer_is_centered_bounded_resolved_and_fullscreen() {
    let welcome = source("src/v2/view/draw/modal/welcome.rs");
    let modal = source("src/v2/view/draw/modal/mod.rs");
    let fullscreen = source("src/v2/view/draw/modal/fullscreen.rs");
    let title = source("src/v2/view/draw/modal/title.rs");

    // The Welcome form's containers are `BModalPart` decisions: centred on
    // both axes and padded, a column bounded by the Splash spec's width, cards
    // spaced by the large gap.
    // The Welcome form's own layout lives with the form, not in the modal
    // vocabulary every modal shares.
    let layout = welcome
        .split("Self::WelcomeLayout =>")
        .nth(1)
        .expect("welcome layout")
        .split("Self::WelcomeColumn")
        .next()
        .expect("welcome layout end");
    assert!(welcome.contains("pub struct BWelcome"));
    assert!(welcome.contains("welcome-centered-layout"));
    assert!(welcome.contains("welcome-content-column"));
    assert!(layout.contains(".align(BAlign::Center)"));
    assert!(layout.contains(".justify(BJustify::Center)"));
    assert!(layout.contains(".pad(BSpace::PadLg)"));
    assert!(welcome.contains("max_width: BModalType::Splash.spec().width"));
    assert!(welcome.contains(".max_width(BLen::Measured(max_width))"));
    assert!(welcome.contains(".gap(BSpace::GapLg)"));
    assert!(modal.contains("BModalType::Splash | BModalType::About"));
    assert!(modal.contains("fullscreen::BFullscreen { kind, body }.render(ctx)"));
    assert!(fullscreen.contains("BWelcome(self.body).render(ctx)"));
    // Fullscreen covers the window: an overlay at full size. This one is
    // still shared modal vocabulary, so it stays in part.rs.
    let part = source("src/v2/view/draw/modal/part.rs");
    let full = part
        .split("Self::Fullscreen =>")
        .nth(1)
        .expect("fullscreen part")
        .split("Self::FullscreenBody")
        .next()
        .expect("fullscreen part end");
    assert!(full.contains(".place(BPlace::Overlay)"));
    assert!(full.contains(".full()"));
    // The title bar reads its icon and label through the modal's heading,
    // so the reader can show its document's title.
    assert!(title.contains("self.kind.heading(ctx.state)"));
}

#[test]
fn modal_popover_and_canvas_decomposition_is_thin_and_resolved() {
    let modal = source("src/v2/view/draw/modal/mod.rs");
    let card = source("src/v2/view/draw/modal/card.rs");
    let popover = source("src/v2/view/draw/popover/mod.rs");
    let canvas = source("src/v2/view/draw/canvas/mod.rs");
    let fallback = source("src/v2/view/draw/canvas/fallback.rs");

    for dispatcher in [&modal, &canvas] {
        assert!(!dispatcher.contains("div("));
        assert!(!dispatcher.contains("svg("));
        assert!(!dispatcher.contains(".child("));
    }
    // The card's surface is a `BModalPart` decision: the Modal radius role,
    // at the Modal elevation — the theme's full shadow, the ceiling every
    // level below it is a rung under — on the Modal surface tone.
    let part = source("src/v2/view/draw/modal/part.rs");
    let apply = source("src/v2/view/draw/flex/apply.rs");
    let tokens = source("src/v2/view/draw/flex/fill.rs");
    assert!(card.contains("pub struct BCard"));
    assert!(card.contains("BModalPart::Card {"));
    assert!(part.contains(".radius(BRadius::Role(BStyleRole::Modal))"));
    assert!(part.contains(".elevated(BElevation::Modal)"));
    assert!(
        apply.contains("element = shadowed(element, ctx.style.shadow_for(self.elevation))"),
        "a container casts what its elevation casts, so a modal, a dropdown and a floating pane differ"
    );
    assert!(part.contains(".fill(BFill::Tone(BSurfaceTone::Modal))"));
    assert!(tokens.contains("Self::Tone(tone) => resolved_gradient(ctx.style.surface(tone))"));
    assert!(card.contains("modal-backdrop-"));
    assert!(popover.contains("pub struct BPopover"));
    assert!(popover.contains("BStyleRole::Popover"));
    // The popover's panel is a card's frame that floats: it names the Card
    // tone as a fill token and `fill.rs` resolves it, exactly as the modal
    // card above does — not by reaching for the surface itself.
    assert!(popover.contains(".fill(BFill::Tone(BSurfaceTone::Card))"));
    assert!(canvas.contains("view3d::BView3d { camera }.render(ctx)"));
    assert!(canvas.contains("fallback::BFallback(host).render(ctx)"));
    assert!(fallback.contains("ctx.style.viewport_gradient()"), "a canvas paints the theme's canvas sweep");

    for renderer in [card, popover, fallback] {
        assert!(!renderer.contains("ctx.p."));
        assert!(!renderer.contains("METRICS"));
        assert!(!renderer.contains("render_impl"));
    }
    assert!(!modal.contains("BView::Space"));
    assert!(!modal.contains("BSpaceType"));
}

/// A module is a file or a folder; read whichever spelling exists.
fn module_file(path: std::path::PathBuf) -> std::path::PathBuf {
    if path.exists() {
        return path;
    }
    // `x.rs` asked for, `x/mod.rs` exists.
    let folder = path.with_extension("").join("mod.rs");
    if folder.exists() {
        return folder;
    }
    // `x/mod.rs` asked for, `x.rs` exists.
    if path.file_name().is_some_and(|n| n == "mod.rs") {
        if let Some(parent) = path.parent() {
            let file = parent.with_extension("rs");
            if file.exists() {
                return file;
            }
        }
    }
    path
}
