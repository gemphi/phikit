//! BView debug tree printing — separated from bview.rs for size.

use super::bview::BView;

impl BView {
    pub fn debug_tree(&self) {
        self.debug_tree_indent(0);
        println!();
    }

    fn debug_tree_indent(&self, depth: usize) {
        let indent = "  ".repeat(depth);
        match self {
            BView::Card {
                shade,
                header,
                icon,
                badge,
                tool,
                collapsible,
                collapsed,
                collapse_action: _,
                body,
                footer,
                footer_action,
                width,
                height,
            } => {
                println!(
                    "{}Card(shade={:?} header={:?} icon={:?} badge={:?} coll={}/{} w={:?} h={:?})",
                    indent, shade, header, icon, badge, collapsible, collapsed, width, height
                );
                if let Some(t) = tool {
                    t.debug_tree_indent(depth + 1);
                }
                if let Some(b) = body {
                    b.debug_tree_indent(depth + 1);
                }
                if let Some(f) = footer {
                    println!("{}  Footer({:?})", indent, f);
                }
                if let Some(fa) = footer_action {
                    fa.debug_tree_indent(depth + 1);
                }
            }
            BView::List { gap, items } => {
                println!("{}List(gap={:?}) [{} items]", indent, gap, items.len());
                for item in items {
                    item.debug_tree_indent(depth + 1);
                }
            }
            BView::Item {
                label,
                icon,
                badge,
                active,
                action,
            } => {
                println!(
                    "{}Item({:?} icon={:?} badge={:?} active={} action={:?})",
                    indent, label, icon, badge, active, action
                );
            }
            BView::Empty => println!("{}Empty", indent),
            BView::Label { text, icon } => println!("{}Label({:?} icon={:?} )", indent, text, icon),
            BView::PropRow { label, value } => {
                println!("{}PropRow({:?} = {:?})", indent, label, value)
            }
            BView::Separator { vertical, action } => {
                println!("{}Separator(v={} action={:?})", indent, vertical, action)
            }
            BView::Row { gap, children } => {
                println!("{}Row(gap={:?}) [{} children]", indent, gap, children.len());
                for c in children {
                    c.debug_tree_indent(depth + 1);
                }
            }
            BView::Column { gap, children, .. } => {
                println!(
                    "{}Column(gap={:?}) [{} children]",
                    indent,
                    gap,
                    children.len()
                );
                for c in children {
                    c.debug_tree_indent(depth + 1);
                }
            }
            BView::Panel {
                role,
                title,
                children,
                width,
                height,
            } => {
                println!(
                    "{}Panel(role={:?} title={:?}) [{} children]",
                    indent,
                    role,
                    title,
                    children.len()
                );
                let _ = (width, height);
                for c in children {
                    c.debug_tree_indent(depth + 1);
                }
            }
            BView::Dropdown {
                label,
                icon,
                icon_only: _,
                align_right: _,
                open,
                children,
                action,
                hover_action,
            } => {
                println!(
                    "{}Dropdown({:?} icon={:?} open={} action={:?} hover={:?}) [{} children]",
                    indent,
                    label,
                    icon,
                    open,
                    action,
                    hover_action,
                    children.len()
                );
                for c in children {
                    c.debug_tree_indent(depth + 1);
                }
            }
            BView::HTabBar { children } => {
                println!("{}HTabBar [{} children]", indent, children.len());
                for c in children {
                    c.debug_tree_indent(depth + 1);
                }
            }
            BView::VTabBar { width, children } => {
                println!(
                    "{}VTabBar(w={:?}) [{} children]",
                    indent,
                    width,
                    children.len()
                );
                for c in children {
                    c.debug_tree_indent(depth + 1);
                }
            }
            BView::Tab {
                label,
                icon,
                active,
                icon_only,
                action,
            } => println!(
                "{}Tab({:?} icon={:?} active={} icon_only={} action={:?})",
                indent, label, icon, active, icon_only, action
            ),
            BView::IconList { children } => {
                println!("{}IconList [{} children]", indent, children.len());
                for c in children {
                    c.debug_tree_indent(depth + 1);
                }
            }
            BView::IconButton {
                icon,
                label,
                tooltip,
                action,
                active,
            } => println!(
                "{}IconButton({:?} label={:?} tooltip={:?} action={:?} active={})",
                indent, icon, label, tooltip, action, active
            ),
            BView::Section {
                label,
                icon,
                expanded,
                children,
                action,
            } => {
                println!(
                    "{}Section({:?} icon={:?} exp={} action={:?}) [{} children]",
                    indent,
                    label,
                    icon,
                    expanded,
                    action,
                    children.len()
                );
                for c in children {
                    c.debug_tree_indent(depth + 1);
                }
            }
            BView::Scroll { children } => {
                println!("{}Scroll [{} children]", indent, children.len());
                for c in children {
                    c.debug_tree_indent(depth + 1);
                }
            }
            BView::Spatial {
                mesh,
                objects,
                vertices,
                filename,
                camera,
            } => {
                let has_mesh = mesh.is_some();
                println!(
                    "{}Spatial({:?} objs={} verts={} mesh={} cam={:?})",
                    indent, filename, objects, vertices, has_mesh, camera
                );
            }
            BView::Switcher {
                host,
                open,
                preview_host,
                action,
            } => {
                println!(
                    "{}Switcher(host={:?} open={} preview={:?} action={:?})",
                    indent, host, open, preview_host, action
                );
            }
        }
    }
}
