//! Accordion property sections for N-Panel.

use super::super::super::bview::{BAction, BView};
use super::super::super::state::{BKey, BState};
use crate::store::parts::panel::PanelSection;

pub struct PanelAccordion;

impl PanelAccordion {
    pub fn build(state: &BState, sections: &[PanelSection], active_tab: &str) -> BView {
        let content: Vec<BView> = sections
            .iter()
            .filter(|s| s.idx == active_tab)
            .flat_map(|s| {
                s.panels.iter().map(|acc| {
                    let acc_expanded = match state.get(&BKey::Section(acc.label.to_string())) {
                        Some("true") => true,
                        Some("false") => false,
                        _ => acc.expanded,
                    };
                    let children: Vec<BView> = if acc_expanded {
                        acc.rows
                            .iter()
                            .map(|r| BView::prop_row(r.label, r.value))
                            .collect()
                    } else {
                        vec![]
                    };
                    BView::section_action(
                        acc.label,
                        Some(acc.icon.to_string()),
                        acc_expanded,
                        children,
                        BAction::Toggle(BKey::Section(acc.label.to_string())),
                    )
                })
            })
            .collect();

        BView::scroll(content)
    }
}
