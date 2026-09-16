//! BView rendering — thin dispatcher to core view structs, universal Card, and part elements.

use super::bview::{ActionHandler, BView};
use super::card::{CardItemView, CardListView, CardView};
use super::core::*;
use super::part::switcher::SwitcherElementView;
use super::theme::BPalette;
use gpui::AnyElement;

impl BView {
    pub fn into_elm(&self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        match self {
            // Universal Card & Composable Elements
            BView::Card {
                shade,
                header,
                icon,
                badge,
                tool,
                collapsible,
                collapsed,
                collapse_action,
                body,
                footer,
                footer_action,
                width,
                height,
            } => CardView {
                shade: *shade,
                header: header.clone(),
                icon: icon.clone(),
                badge: badge.clone(),
                tool: tool.clone(),
                collapsible: *collapsible,
                collapsed: *collapsed,
                collapse_action: collapse_action.clone(),
                body: body.clone(),
                footer: footer.clone(),
                footer_action: footer_action.clone(),
                width: *width,
                height: *height,
            }
            .into_elm(p, on_action),

            BView::List { gap, items } => CardListView {
                gap: *gap,
                items: items.clone(),
            }
            .into_elm(p, on_action),

            BView::Item {
                label,
                icon,
                badge,
                active,
                action,
            } => CardItemView {
                label: label.clone(),
                icon: icon.clone(),
                badge: badge.clone(),
                active: *active,
                action: action.clone(),
            }
            .into_elm(p, on_action),

            // Standard Primitives
            BView::Empty => EmptyView::into_elm(),
            BView::Label { text, icon } => {
                LabelView::new(text.clone(), icon.clone()).into_elm(p, on_action)
            }
            BView::PropRow { label, value } => {
                PropRowView::new(label.clone(), value.clone()).into_elm(p)
            }
            BView::Separator { vertical, action } => {
                SeparatorView::new(*vertical).into_elm(p, action.as_ref(), on_action)
            }
            BView::Row { gap, children } => {
                RowView::new(*gap, children.clone()).into_elm(p, on_action)
            }
            BView::Column {
                gap,
                children,
                center,
            } => {
                let cv = if *center {
                    ColumnView::centered(*gap, children.clone())
                } else {
                    ColumnView::new(*gap, children.clone())
                };
                cv.into_elm(p, on_action)
            }
            BView::Panel {
                role,
                title,
                children,
                width,
                height,
            } => PanelView::new(*role, title.clone(), children.clone())
                .into_elm(p, *width, *height, on_action),
            BView::Dropdown {
                label,
                icon,
                icon_only,
                align_right,
                open,
                children,
                action,
                hover_action,
            } => {
                let mut dv = DropdownView::new(
                    label.clone(),
                    icon.clone(),
                    *icon_only,
                    *align_right,
                    children.clone(),
                );
                dv.hover_action = hover_action.clone();
                dv.into_elm(p, *open, action.as_ref(), on_action)
            }
            BView::HTabBar { children } => {
                HTabBarView::new(children.clone()).into_elm(p, on_action)
            }
            BView::VTabBar { width, children } => {
                VTabBarView::new(*width, children.clone()).into_elm(p, on_action)
            }
            BView::Tab {
                label,
                icon,
                active,
                icon_only,
                action,
            } => TabView::new(label.clone(), icon.clone(), *active, *icon_only).into_elm(
                p,
                action.as_ref(),
                on_action,
            ),
            BView::IconList { children } => {
                IconListView::new(children.clone()).into_elm(p, on_action)
            }
            BView::IconButton {
                icon,
                label,
                tooltip,
                action,
                active,
            } => IconButtonView::new(icon.clone(), label.clone(), tooltip.clone()).into_elm(
                p,
                *active,
                action.as_ref(),
                on_action,
            ),
            BView::Section {
                label,
                icon,
                expanded,
                children,
                action,
            } => SectionView::new(label.clone(), icon.clone(), *expanded, children.clone())
                .into_elm(p, action.as_ref(), on_action),
            BView::Scroll { children } => ScrollView::new(children.clone()).into_elm(p, on_action),
            BView::Spatial {
                mesh,
                objects,
                vertices,
                filename,
                camera,
            } => SpatialView::new(mesh.clone(), filename.clone(), *objects, *vertices)
                .with_camera(*camera)
                .into_elm(p, on_action),
            BView::Switcher {
                host,
                open,
                preview_host,
                action,
            } => SwitcherElementView::new(*host, *open, *preview_host).into_elm(
                p,
                Some(action),
                on_action,
            ),
        }
    }

    pub fn into_popup_item(&self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        match self {
            BView::Item {
                label,
                icon,
                badge: _,
                active,
                action,
            } => IconButtonView::new(icon.clone().unwrap_or_default(), Some(label.clone()), None)
                .into_popup_item(p, *active, action.as_ref(), on_action),
            BView::Label { text, icon } => {
                LabelView::new(text.clone(), icon.clone()).into_popup_item(p)
            }
            BView::Separator { vertical, .. } => SeparatorView::new(*vertical).into_popup_item(p),
            BView::Dropdown {
                label,
                icon,
                icon_only,
                align_right,
                open,
                children,
                action,
                hover_action: _,
            } => DropdownView::new(
                label.clone(),
                icon.clone(),
                *icon_only,
                *align_right,
                children.clone(),
            )
            .into_popup_item(p, *open, action.as_ref(), on_action),
            BView::IconButton {
                icon,
                label,
                tooltip: _,
                action,
                active,
            } => IconButtonView::new(icon.clone(), label.clone(), None).into_popup_item(
                p,
                *active,
                action.as_ref(),
                on_action,
            ),
            _ => self.into_elm(p, on_action),
        }
    }
}
