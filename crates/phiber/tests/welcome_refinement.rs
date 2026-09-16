//! Focused regressions for viewport-filling Welcome and current modal forms.
//!
//! Validates: Requirements 3.4, 4.4, 4.5, 10.1, 11.1, 11.5-11.7, 12.7, 14.7-14.12

use phiber::v2::{
    modal::{BModalType, splash::BSplashModal},
    state::{
        BState,
        key::{K_MODAL, K_SPACE},
        val::V_MODAL_FILE_BROWSER,
    },
    view::{BAction, BPanelRole, BView, BViewItem},
};
use std::{fs, path::PathBuf};

fn crate_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn source(relative: &str) -> String {
    let path = crate_path(relative);
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
        BView::Row { children }
        | BView::Column { children }
        | BView::ListRow { children, .. }
        | BView::Panel { children, .. } => {
            for child in children {
                visit(child, strings, icons, actions);
            }
        }
        _ => {}
    }
}

#[test]
fn welcome_uses_phiber_brand_icon_and_one_aligned_content_header() {
    let view = BSplashModal.build(&BState::new());
    let BView::Column { children } = &view else {
        panic!("Welcome body must be one column")
    };
    assert_eq!(children.len(), 4);
    assert!(matches!(
        &children[0],
        BView::Panel { role: BPanelRole::Header, children }
            if matches!(children.as_slice(), [BView::Icon { name }, BView::Label { text }]
                if name == "phiber" && text == "Phiber")
    ));
    assert!(matches!(&children[1], BView::Label { text } if text.contains("recent project")));

    let mut strings = Vec::new();
    let mut icons = Vec::new();
    let mut actions = Vec::new();
    visit(&view, &mut strings, &mut icons, &mut actions);
    assert!(strings.contains(&"Phiber"));
    assert!(
        strings
            .iter()
            .all(|text| !text.to_ascii_lowercase().contains("blender"))
    );
    assert!(icons.contains(&"phiber"));
    assert_eq!(BModalType::Splash.spec().label, "Welcome to Phiber");
    assert_eq!(BModalType::Splash.spec().icon, "phiber");
    assert!(crate_path("assets/icons/phiber.svg").is_file());
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
                action, BAction::Set(path, value) if path == &K_SPACE && value == target
            )),
            "missing template action for {target}"
        );
    }
    assert!(actions.iter().copied().any(|action| matches!(
        action,
        BAction::Set(path, value) if path == &K_MODAL && value == V_MODAL_FILE_BROWSER
    )));
    assert_eq!(
        actions
            .iter()
            .copied()
            .filter(|action| matches!(action, BAction::Set(path, _) if path.encode() == "ctx.link"))
            .count(),
        4
    );
}

#[test]
fn welcome_renderer_is_centered_bounded_resolved_and_fullscreen() {
    let welcome = source("src/v2/view/render/gpui/modal/welcome.rs");
    let modal = source("src/v2/view/render/gpui/modal/mod.rs");
    let fullscreen = source("src/v2/view/render/gpui/modal/fullscreen.rs");
    let title = source("src/v2/view/render/gpui/modal/title.rs");

    assert!(welcome.contains("pub struct BWelcome"));
    assert!(welcome.contains("welcome-centered-layout"));
    assert!(welcome.contains("welcome-content-column"));
    assert!(welcome.contains(".items_center()"));
    assert!(welcome.contains(".justify_center()"));
    assert!(welcome.contains(".max_w(px(BModalType::Splash.spec().width))"));
    assert!(welcome.contains("c.style.spacing.gap_lg"));
    assert!(welcome.contains("c.style.spacing.pad_lg"));
    assert!(modal.contains("BModalType::Splash | BModalType::About"));
    assert!(modal.contains("fullscreen::BFullscreen { kind, body }.render(c)"));
    assert!(fullscreen.contains("BWelcome(self.body).render(c)"));
    assert!(fullscreen.contains(".absolute()"));
    assert!(fullscreen.contains(".inset_0()"));
    assert!(fullscreen.contains(".size_full()"));
    assert!(title.contains("spec.label"));
    assert!(title.contains("spec.icon"));
}

#[test]
fn modal_popover_and_canvas_decomposition_is_thin_and_resolved() {
    let modal = source("src/v2/view/render/gpui/modal/mod.rs");
    let card = source("src/v2/view/render/gpui/modal/card.rs");
    let popover = source("src/v2/view/render/gpui/popover.rs");
    let canvas = source("src/v2/view/render/gpui/atom/canvas/mod.rs");
    let fallback = source("src/v2/view/render/gpui/atom/canvas/fallback.rs");

    for dispatcher in [&modal, &canvas] {
        assert!(!dispatcher.contains("div("));
        assert!(!dispatcher.contains("svg("));
        assert!(!dispatcher.contains(".child("));
    }
    assert!(card.contains("pub struct BCard"));
    assert!(card.contains("BStyleRole::Modal"));
    assert!(card.contains("BShadowStyle::ExtraLarge"));
    assert!(card.contains("c.style.surface(BSurfaceTone::Modal)"));
    assert!(card.contains("modal-backdrop-"));
    assert!(popover.contains("pub struct BPopover"));
    assert!(popover.contains("BStyleRole::Popover"));
    assert!(popover.contains("c.style.surface(BSurfaceTone::Card)"));
    assert!(canvas.contains("view3d::BView3d.render(c)"));
    assert!(canvas.contains("fallback::BFallback(host).render(c)"));
    assert!(fallback.contains("c.style.viewport_surface()"));

    for renderer in [card, popover, fallback] {
        assert!(!renderer.contains("c.p."));
        assert!(!renderer.contains("METRICS"));
        assert!(!renderer.contains("render_impl"));
    }
    assert!(!modal.contains("BView::Space"));
    assert!(!modal.contains("BSpaceType"));
}
