//! `BAsciiRenderer` — headless renderer producing a nested ASCII tree.
//! Useful for tests and debug printouts.

use super::{BView, layout::BRenderer};
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
            BView::Space { kind, list } => {
                o.push_str(&format!("{p}Space({:?})\n", kind));
                self.each(list, s, d + 1, o);
            }
            BView::Zone { kind, list } => {
                o.push_str(&format!("{p}Zone({:?})\n", kind));
                self.each(list, s, d + 1, o);
            }
            BView::Area { kind, child } => {
                o.push_str(&format!("{p}Area({:?})\n", kind));
                self.write(child, s, d + 1, o);
            }
            BView::Host { kind, list } => {
                o.push_str(&format!("{p}Host({:?})\n", kind));
                self.each(list, s, d + 1, o);
            }
            BView::Region { kind, list } => {
                o.push_str(&format!("{p}Region({:?})\n", kind));
                self.each(list, s, d + 1, o);
            }
            BView::Pane { kind, list } => {
                o.push_str(&format!("{p}Pane({:?})\n", kind));
                self.each(list, s, d + 1, o);
            }
            BView::Part { kind, body } => {
                o.push_str(&format!("{p}Part({:?})\n", kind));
                self.write(body, s, d + 1, o);
            }
            BView::Modal { kind, body } => {
                o.push_str(&format!("{p}Modal({:?})\n", kind));
                self.write(body, s, d + 1, o);
            }
            BView::Row { list } => {
                o.push_str(&format!("{p}Row\n"));
                self.each(list, s, d + 1, o);
            }
            BView::ListRow { list, shade } => {
                o.push_str(&format!("{p}ListRow({:?})\n", shade));
                self.each(list, s, d + 1, o);
            }
            BView::TableRow { cell_list, shade, .. } => {
                o.push_str(&format!("{p}TableRow({:?})\n", shade));
                for cell in cell_list {
                    self.write(&cell.view, s, d + 1, o);
                }
            }
            BView::TableHeader { cell_list } => {
                o.push_str(&format!("{p}TableHeader\n"));
                for cell in cell_list {
                    self.write(&cell.view, s, d + 1, o);
                }
            }
            BView::SortHeader {
                label, direction, ..
            } => o.push_str(&format!("{p}SortHeader({label:?} {direction:?})\n")),
            BView::Card(card) => {
                let fold = card.fold.as_ref().map(|f| if f.open { " open" } else { " closed" }).unwrap_or("");
                o.push_str(&format!("{p}Card({:?} {:?}{fold})
", card.title().unwrap_or(""), card.kind));
                if card.is_open() {
                    for v in card.body.iter().chain(&card.foot) {
                        self.write(v, s, d + 1, o);
                    }
                }
            }
            BView::TreeRow {
                depth,
                expanded,
                lead,
                icon,
                list,
                trail,
                shade,
                ..
            } => {
                o.push_str(&format!(
                    "{p}TreeRow(depth={} open={:?} icon={:?} {:?})\n",
                    depth, expanded, icon, shade
                ));
                self.each(lead, s, d + 1, o);
                self.each(list, s, d + 1, o);
                self.each(trail, s, d + 1, o);
            }
            BView::Indent { level } => o.push_str(&format!("{p}Indent({})\n", level)),
            BView::Column { list } => {
                o.push_str(&format!("{p}Column\n"));
                self.each(list, s, d + 1, o);
            }
            // The view it holds, at this depth: sharing is not structure.
            BView::Shared(view) => self.write(view, s, d, o),
            BView::Split { list } => {
                o.push_str(&format!("{p}Split\n"));
                self.each(list, s, d + 1, o);
            }
            BView::Label { text } => o.push_str(&format!("{p}Label({:?})\n", text)),
            BView::Pair { label, value } => o.push_str(&format!("{p}Pair({label:?}, {value:?})\n")),
            BView::Ctrl(ctrl) => o.push_str(&format!("{p}Ctrl({:?}, {}, {:?})\n", ctrl.kind, ctrl.key.encode(), ctrl.value)),
            // The share is what a reader checks: a track that is 0.75 full is
            // saying the value is three quarters along, which is the claim.
            BView::Track { track, share, text } => {
                o.push_str(&format!("{p}Track({}, {share:.2}, {:?})\n", track.key.encode(), text))
            }
            // Where the cursor sits on the wheel, and the value, are the claim.
            BView::Wheel { wheel } => {
                let (across, down) = wheel.marker();
                o.push_str(&format!("{p}Wheel({}, {across:.2}, {down:.2}, v={:.2})\n", wheel.key.encode(), wheel.hsv[2]))
            }
            BView::Prose { role, text } => o.push_str(&format!("{p}Prose({role:?}, {:?})\n", text)),
            BView::VLabel { text } => o.push_str(&format!("{p}VLabel({:?})\n", text)),
            BView::Field { text } => o.push_str(&format!("{p}Field({:?})\n", text)),
            BView::SearchField { placeholder, value, .. } => o.push_str(&format!(
                "{p}SearchField(placeholder={:?} value={:?})\n",
                placeholder, value
            )),
            BView::Icon { name } => o.push_str(&format!("{p}Icon({:?})\n", name)),
            // `active` on both, as `Tab` / `RailEntry` / `MenuAnchor` already
            // print it. Without it no snapshot could tell a ticked row from an
            // unticked one, which is how fourteen call sites came to say "on"
            // with a glyph and nothing else and never be caught
            // (`on_state_budget`).
            BView::Button(ctl) => o.push_str(&format!("{p}Button({:?}{})\n", ctl.icon, if ctl.active { " active" } else { "" })),
            BView::MenuItem(ctl) => o.push_str(&format!("{p}MenuItem({:?}{})\n", ctl.label, if ctl.active { " active" } else { "" })),
            BView::MenuSep => o.push_str(&format!("{p}MenuSep\n")),
            BView::Spacer => o.push_str(&format!("{p}Spacer\n")),
            BView::ZoneToggleGroup { controls } => {
                o.push_str(&format!("{p}ZoneToggleGroup\n"));
                for control in controls {
                    o.push_str(&format!(
                        "{p}  ZoneToggle({:?} visible={})\n",
                        control.zone, control.control.active
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
                if let Some(panels) = &rail.panels {
                    o.push_str(&format!("{p}  RailPanels\n"));
                    self.each(panels, s, d + 2, o);
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
            BView::WindowControls { leading } => o.push_str(&format!("{p}WindowControls(leading={leading})\n")),
            BView::Thumbnail(t) => o.push_str(&format!("{p}Thumbnail({:?})\n", t.label)),
            BView::Grid { list } => {
                o.push_str(&format!("{p}Grid\n"));
                self.each(list, s, d + 1, o);
            }
            BView::Canvas { host, camera } => o.push_str(&format!("{p}Canvas({:?}{})\n", host, if camera.through { ", camera" } else { "" })),
            BView::Log { open, .. } => o.push_str(&format!("{p}Log(open={open})\n")),
            BView::Avatar { path, size } => o.push_str(&format!("{p}Avatar({path:?}, {size})\n")),
            BView::Badge { icon, count } => o.push_str(&format!("{p}Badge({icon}, {count})\n")),
            BView::ProgressBar { fraction, label, .. } => {
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
