//! PhiberApp — the interactive application entity.
//! Struct + Render impl + tests. No divs or views here.

use gpui::{Context, IntoElement, Render, Window};

use crate::bui::BState;
use crate::bui::theme::BPalette;
use crate::bui::views::{BViewBuilder, SpaceView};

pub struct PhiberApp {
    pub state: BState,
    first_render: bool,
}

impl PhiberApp {
    pub fn new() -> Self {
        Self {
            state: BState::default(),
            first_render: true,
        }
    }

    pub fn palette(&self) -> BPalette {
        BPalette::from_state(&self.state)
    }
}

impl Default for PhiberApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for PhiberApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        eprintln!("[render] start");
        let tree = SpaceView::new().build(&self.state);
        eprintln!("[render] tree built");
        if self.first_render {
            let p = self.palette();
            println!("=== PHIBER LAYOUT TREE (theme: {}) ===", p.name);
            tree.debug_tree();
            println!("=== END LAYOUT TREE ===");
            self.first_render = false;
        }
        eprintln!("[render] into_elm");
        let result = PhiberApp::render(self, &tree, cx);
        eprintln!("[render] done");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bui::bview::BAction;
    use crate::bui::theme::{BPalette, BTheme};
    use crate::bui::{BKey, BView};
    use crate::bui::{BViewBuilder, HostView, SpaceView};
    use crate::store::BPanelRole;
    use crate::store::hosts::BHostType;
    use crate::store::spaces::BSpaceType;

    #[test]
    fn test_default_state() {
        let app = PhiberApp::new();
        let space = app
            .state
            .get(&BKey::CtxSpace)
            .and_then(BSpaceType::from_name)
            .unwrap_or(BSpaceType::Layout);
        let host = app
            .state
            .get(&BKey::CtxHost)
            .and_then(BHostType::from_name)
            .unwrap_or(BHostType::View3D);
        assert_eq!(space, BSpaceType::Layout);
        assert_eq!(host, BHostType::View3D);
    }

    #[test]
    fn test_switch_space() {
        let mut app = PhiberApp::new();
        app.state
            .set(&BKey::CtxSpace, &format!("{:?}", BSpaceType::Modeling));
        let space = app
            .state
            .get(&BKey::CtxSpace)
            .and_then(BSpaceType::from_name)
            .unwrap_or(BSpaceType::Layout);
        assert_eq!(space, BSpaceType::Modeling);
    }

    #[test]
    fn test_toggle_section() {
        let mut s = BState::default();
        let key = BKey::Section("X".to_string());
        assert!(s.get(&key) != Some("true"));
        s.set(&key, "true");
        assert!(s.get(&key) == Some("true"));
        s.set(&key, "false");
        assert!(s.get(&key) != Some("true"));
    }

    #[test]
    fn test_tree_root_is_app() {
        let s = BState::default();
        let tree = SpaceView::new().build(&s);
        match &tree {
            BView::Panel { role, .. } => assert_eq!(*role, BPanelRole::App),
            other => panic!("expected App, got {:?}", other),
        }
    }

    #[test]
    fn test_theme_cycle() {
        use crate::bui::theme::BTheme;
        let mut s = BState::default();
        assert_eq!(s.get(&BKey::Theme), Some(BTheme::BlenderDark.name()));
        s.set(&BKey::Theme, BTheme::AppleDark.name());
        assert_eq!(s.get(&BKey::Theme), Some(BTheme::AppleDark.name()));
        s.set(&BKey::Theme, BTheme::BlenderDark.name());
        assert_eq!(s.get(&BKey::Theme), Some(BTheme::BlenderDark.name()));
    }

    #[test]
    fn test_tabs_have_actions() {
        let s = BState::default();
        let tree = SpaceView::new().build(&s);
        assert!(find_action_tab(&tree), "spacebar tabs should have actions");
    }

    #[test]
    fn test_confirm_buts_and_all_hosts_showing() {
        use crate::store::hosts::SWITCHABLE_HOSTS;
        use crate::store::parts::panel;

        let mut s = BState::default();

        // Confirm Buts (Properties) host generates sections and buttons correctly
        s.set(&BKey::CtxHost, &format!("{:?}", BHostType::Buts));
        let host = s
            .get(&BKey::CtxHost)
            .and_then(BHostType::from_name)
            .unwrap_or(BHostType::View3D);
        assert_eq!(host, BHostType::Buts);

        let buts_view = HostView::new(BHostType::Buts).build(&s);
        println!("=== DEBUG TREE: BUTS (PROPERTIES) HOST ===");
        buts_view.debug_tree();
        println!("=== END DEBUG TREE ===");

        // Verify BUTS sections exist
        let sections = panel::set::with_idx(BHostType::Buts);
        assert!(
            !sections.is_empty(),
            "Buts host should have defined sections"
        );
        let section_labels: Vec<&str> = sections.iter().map(|sec| sec.label).collect();
        assert!(
            section_labels.contains(&"Render"),
            "Buts should have Render section"
        );
        assert!(
            section_labels.contains(&"Scene"),
            "Buts should have Scene section"
        );
        assert!(
            section_labels.contains(&"World"),
            "Buts should have World section"
        );

        // Verify ALL switchable hosts render valid non-empty views
        for &h in SWITCHABLE_HOSTS {
            s.set(&BKey::CtxHost, &format!("{:?}", h));
            let v = HostView::new(h).build(&s);
            match &v {
                BView::Panel { children, .. } => {
                    assert!(
                        !children.is_empty(),
                        "Host {:?} should have non-empty children in host_view",
                        h
                    );
                }
                BView::Empty => panic!("Host {:?} rendered BView::Empty", h),
                _ => {}
            }
        }
    }

    fn find_action_tab(v: &BView) -> bool {
        match v {
            BView::Tab {
                action: Some(_), ..
            } => true,
            BView::Panel { children, .. }
            | BView::Row { children, .. }
            | BView::Column { children, .. }
            | BView::Dropdown { children, .. }
            | BView::HTabBar { children }
            | BView::VTabBar { children, .. }
            | BView::IconList { children }
            | BView::Scroll { children } => children.iter().any(find_action_tab),
            BView::Section { children, .. } => children.iter().any(find_action_tab),
            _ => false,
        }
    }
}
