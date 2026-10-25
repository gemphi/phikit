//! Focused regressions for viewport-filling Welcome and current modal forms.
//!
//! Validates: Requirements 3.4, 4.4, 4.5, 10.1, 11.1, 11.5-11.7, 12.7, 14.7-14.12

use phiber::v2::state::{AppKey, BKey as _, ModalKey, OpKey};
use phiber::v2::{
    modal::{BModalType, splash::{BSplashModal, BWelcomeTab}},
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
        BView::Thumbnail(t) => {
            icons.push(&t.icon);
            strings.push(&t.label);
            if let Some(action) = &t.action {
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

/// Every card in `view`, depth first.
fn cards(view: &BView, out: &mut Vec<BCardKind>) {
    match view {
        BView::Card(card) => {
            out.push(card.kind);
            for child in card.body.iter().chain(&card.foot) {
                cards(child, out);
            }
        }
        BView::Row { list } | BView::Column { list } | BView::ListRow { list, .. } => list.iter().for_each(|c| cards(c, out)),
        _ => {}
    }
}

/// The Welcome body with `tab` open.
fn welcome(tab: BWelcomeTab) -> BView {
    let mut state = BState::new();
    state.set(AppKey::WelcomeTab, tab.slug());
    BSplashModal.build(&state)
}

#[test]
fn welcome_leads_with_the_logo_and_paints_no_header_bar() {
    // The home (`modal/splash/mod.rs`): the 3D logo and its line on the
    // backdrop, the tabs on their one ground, then the open tab's cards.
    let view = welcome(BWelcomeTab::New);
    let BView::Column { list } = &view else {
        panic!("Welcome body must be one column")
    };
    assert!(list.len() > 4, "logo, tagline, tabs, then the rows of cards");
    // Nothing is painted under the logo or its line: neither is a card.
    for line in &list[..2] {
        let mut kinds = Vec::new();
        cards(line, &mut kinds);
        assert!(kinds.is_empty(), "the logo stands on the backdrop: {kinds:?}");
    }
    let BView::Card(tabs) = &list[2] else { panic!("the tabs are a strip") };
    assert_eq!(tabs.kind, BCardKind::Strip(BStrip::Tabs));
    assert_eq!(tabs.body.len(), BWelcomeTab::ALL.len());
    // The tab changes what is under it.
    for tab in BWelcomeTab::ALL {
        let BView::Column { list } = welcome(tab) else { panic!() };
        let rows = &list[3..];
        assert!(!rows.is_empty(), "{tab:?} shows no cards");
        for row in rows {
            let BView::Row { list: line } = row else { continue };
            let tiles = line.iter().filter(|v| matches!(v, BView::Thumbnail(_))).count();
            assert!(tiles <= phiber::v2::modal::splash::ACROSS, "{tab:?}: {tiles} cards in a row");
        }
    }
    // No filled header bar anywhere.
    let mut kinds = Vec::new();
    cards(&view, &mut kinds);
    assert!(kinds.iter().all(|k| *k != BCardKind::Strip(BStrip::Header)), "{kinds:?}");

    let mut strings = Vec::new();
    let mut icons = Vec::new();
    let mut actions = Vec::new();
    visit(&list[0], &mut strings, &mut icons, &mut actions);
    // The hero carries the name as the 3D logo, in the logo's own font
    // rather than the interface's (`brand::WORDMARK_3D`).
    assert_eq!(icons, ["phiber_wordmark_3d"]);
    let (mut strings, mut icons) = (Vec::new(), Vec::new());
    visit(&view, &mut strings, &mut icons, &mut actions);
    assert!(
        strings
            .iter()
            .all(|text| !text.to_ascii_lowercase().contains("blender"))
    );
    assert_eq!(BModalType::Splash.spec().label, "Welcome to Phiber");
    assert_eq!(BModalType::Splash.spec().icon, "phiber");
    for asset in ["phiber", "phiber_splash"] {
        assert!(crate_path(&format!("assets/icons/{asset}.svg")).is_file(), "{asset}");
    }
    for layer in ["side", "body", "pill"] {
        assert!(crate_path(&format!("assets/brand/phiber_mark_{layer}.svg")).is_file(), "{layer}");
    }
    assert!(crate_path("assets/brand/phiber_wordmark.svg").is_file());
}

#[test]
fn welcome_preserves_template_open_and_link_actions() {
    // Every tab together: what the window offers across them.
    let views: Vec<BView> = BWelcomeTab::ALL.into_iter().map(welcome).collect();
    let mut strings = Vec::new();
    let mut icons = Vec::new();
    let mut actions = Vec::new();
    for view in &views {
        visit(view, &mut strings, &mut icons, &mut actions);
    }

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
                action, BAction::Set(path, value) if path == &OpKey::NewFile.path() && value == target
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
        // The links that stay, and the home groups' rows that open a page
        // of the manual (the Quests).
        entries.len() - leaving + home_docs()
    );
}

/// The home groups' rows that open a manual page.
fn home_docs() -> usize {
    use phiber::v2::modal::splash::home::{BHomeTarget, GROUPS};
    GROUPS.iter().flat_map(|g| g.rows).filter(|r| matches!(r.goes, Some(BHomeTarget::Doc(_)))).count()
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
