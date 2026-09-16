//! `BAsciiRenderer` — headless renderer producing a nested ASCII tree.
//! Useful for tests and debug printouts.

use super::{BRenderer, BView};
use crate::v2::state::BState;

pub struct BAsciiRenderer;

impl BRenderer for BAsciiRenderer {
    type Output = String;
    fn render(&self, view: &BView, state: &BState) -> String {
        let mut out = String::new();
        self.write(view, state, 0, &mut out);
        out
    }
}

impl BAsciiRenderer {
    fn write(&self, v: &BView, s: &BState, d: usize, o: &mut String) {
        let p = "  ".repeat(d);
        match v {
            BView::Space { kind, children } => {
                o.push_str(&format!("{p}Space({:?})\n", kind));
                self.each(children, s, d + 1, o);
            }
            BView::Zone { kind, children } => {
                o.push_str(&format!("{p}Zone({:?})\n", kind));
                self.each(children, s, d + 1, o);
            }
            BView::Area { kind, child } => {
                o.push_str(&format!("{p}Area({:?})\n", kind));
                self.write(child, s, d + 1, o);
            }
            BView::Host { kind, children } => {
                o.push_str(&format!("{p}Host({:?})\n", kind));
                self.each(children, s, d + 1, o);
            }
            BView::Region { kind, children } => {
                o.push_str(&format!("{p}Region({:?})\n", kind));
                self.each(children, s, d + 1, o);
            }
            BView::Pane { kind, children } => {
                o.push_str(&format!("{p}Pane({:?})\n", kind));
                self.each(children, s, d + 1, o);
            }
            BView::Part { kind, body } => {
                o.push_str(&format!("{p}Part({:?})\n", kind));
                self.write(body, s, d + 1, o);
            }
            BView::Modal { kind, body } => {
                o.push_str(&format!("{p}Modal({:?})\n", kind));
                self.write(body, s, d + 1, o);
            }
            BView::Row { children } => {
                o.push_str(&format!("{p}Row\n"));
                self.each(children, s, d + 1, o);
            }
            BView::ListRow { children, shade } => {
                o.push_str(&format!("{p}ListRow({:?})\n", shade));
                self.each(children, s, d + 1, o);
            }
            BView::TableRow { cells, shade, .. } => {
                o.push_str(&format!("{p}TableRow({:?})\n", shade));
                for cell in cells {
                    self.write(&cell.view, s, d + 1, o);
                }
            }
            BView::TableHeader { cells } => {
                o.push_str(&format!("{p}TableHeader\n"));
                for cell in cells {
                    self.write(&cell.view, s, d + 1, o);
                }
            }
            BView::SortHeader {
                label, direction, ..
            } => o.push_str(&format!("{p}SortHeader({label:?} {direction:?})\n")),
            BView::AccordionHeader {
                title, expanded, ..
            } => o.push_str(&format!(
                "{p}AccordionHeader({title:?} expanded={expanded})\n"
            )),
            BView::TreeRow {
                depth,
                expanded,
                icon,
                children,
                trail,
                shade,
                ..
            } => {
                o.push_str(&format!(
                    "{p}TreeRow(depth={} open={:?} icon={:?} {:?})\n",
                    depth, expanded, icon, shade
                ));
                self.each(children, s, d + 1, o);
                self.each(trail, s, d + 1, o);
            }
            BView::Indent { level } => o.push_str(&format!("{p}Indent({})\n", level)),
            BView::Column { children } => {
                o.push_str(&format!("{p}Column\n"));
                self.each(children, s, d + 1, o);
            }
            BView::Panel { role, children } => {
                o.push_str(&format!("{p}Panel({:?})\n", role));
                self.each(children, s, d + 1, o);
            }
            BView::Label { text } => o.push_str(&format!("{p}Label({:?})\n", text)),
            BView::VLabel { text } => o.push_str(&format!("{p}VLabel({:?})\n", text)),
            BView::Field { text } => o.push_str(&format!("{p}Field({:?})\n", text)),
            BView::SearchField { placeholder, value, .. } => o.push_str(&format!(
                "{p}SearchField(placeholder={:?} value={:?})\n",
                placeholder, value
            )),
            BView::Icon { name } => o.push_str(&format!("{p}Icon({:?})\n", name)),
            BView::Button(ctl) => o.push_str(&format!("{p}Button({:?})\n", ctl.icon)),
            BView::MenuItem(ctl) => o.push_str(&format!("{p}MenuItem({:?})\n", ctl.label)),
            BView::MenuSep => o.push_str(&format!("{p}MenuSep\n")),
            BView::Spacer => o.push_str(&format!("{p}Spacer\n")),
            BView::ZoneToggleGroup { controls } => {
                o.push_str(&format!("{p}ZoneToggleGroup\n"));
                for control in controls {
                    o.push_str(&format!(
                        "{p}  ZoneToggle({:?} visible={})\n",
                        control.zone, control.visible
                    ));
                }
            }
            BView::Tab(ctl) => {
                o.push_str(&format!("{p}Tab({:?} active={})\n", ctl.icon, ctl.active))
            }
            BView::Rail(rail) => {
                o.push_str(&format!(
                    "{p}Rail({:?} {:?} expanded={})\n",
                    rail.kind, rail.side, rail.expanded
                ));
                for e in &rail.entries {
                    o.push_str(&format!(
                        "{p}  RailEntry({:?} {:?} active={})\n",
                        e.control.icon, e.control.label, e.control.active
                    ));
                }
            }
            BView::MenuAnchor(ctl) => o.push_str(&format!(
                "{p}MenuAnchor({:?} active={})\n",
                ctl.label.as_deref().or(ctl.icon.as_deref()).unwrap_or(""),
                ctl.active
            )),
            BView::Popover {
                anchor, body, open, ..
            } => {
                o.push_str(&format!("{p}Popover(open={})\n", open));
                self.write(anchor, s, d + 1, o);
                self.each(body, s, d + 1, o);
            }
            BView::Canvas { host } => o.push_str(&format!("{p}Canvas({:?})\n", host)),
            BView::ProgressBar { fraction, label } => {
                o.push_str(&format!("{p}ProgressBar({label:?}, {fraction:.2})\n"))
            }
            BView::Empty => o.push_str(&format!("{p}Empty\n")),
        }
    }
    fn each(&self, cs: &[BView], s: &BState, d: usize, o: &mut String) {
        for c in cs {
            self.write(c, s, d, o);
        }
    }
}
