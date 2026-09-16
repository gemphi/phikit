use crate::v2::part::panel::prefs::{BPreferenceControls, BPreferenceSection, SECTIONS};
use crate::v2::state::{BState, key::K_THEME};
use crate::v2::theme::{BPaletteName, BUiStyleDefaults, ui_style_defaults};
use crate::v2::view::{BPanelRole, BView, BViewItem};

#[derive(Clone, Copy, Debug, Default)]
pub struct BPrefsItem;

impl BPrefsItem {
    pub fn new() -> Self {
        Self
    }
}

impl BViewItem for BPrefsItem {
    fn build(&self, state: &BState) -> BView {
        let defaults = defaults_for(state);
        let sections = SECTIONS
            .iter()
            .map(|section| section_view(section, state, defaults))
            .collect();
        BView::panel(BPanelRole::Body, sections)
    }
}

fn defaults_for(state: &BState) -> &'static BUiStyleDefaults {
    let palette = state
        .get(&K_THEME)
        .and_then(BPaletteName::from_slug)
        .unwrap_or(BPaletteName::DEFAULT);
    ui_style_defaults(palette)
}

fn section_view(section: BPreferenceSection, state: &BState, defaults: &BUiStyleDefaults) -> BView {
    let mut children = Vec::with_capacity(section.len() + 1);
    children.push(BView::panel(
        BPanelRole::CardHeader,
        vec![BView::label(section.title())],
    ));
    children.extend(section.rows().map(|row| {
        BView::panel(
            BPanelRole::Section,
            vec![
                BView::label(row.metadata().label),
                BPreferenceControls::for_row(row, state, defaults).view(),
            ],
        )
    }));
    BView::panel(BPanelRole::Card, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::part::panel::prefs::BPreferenceRow;
    use crate::v2::view::BAction;

    fn section_card<'a>(view: &'a BView, title: &str) -> &'a [BView] {
        let BView::Panel {
            role: BPanelRole::Body,
            children,
        } = view
        else {
            panic!("preferences must use a bounded body panel")
        };
        children
            .iter()
            .find_map(|card| {
                let BView::Panel {
                    role: BPanelRole::Card,
                    children,
                } = card
                else {
                    return None;
                };
                let BView::Panel {
                    role: BPanelRole::CardHeader,
                    children: header,
                } = &children[0]
                else {
                    return None;
                };
                matches!(&header[0], BView::Label { text } if text == title)
                    .then_some(&children[..])
            })
            .unwrap_or_else(|| panic!("missing preference section {title}"))
    }

    fn row_for<'a>(card: &'a [BView], label: &str) -> &'a [BView] {
        card[1..]
            .iter()
            .find_map(|row| {
                let BView::Panel {
                    role: BPanelRole::Section,
                    children,
                } = row
                else {
                    return None;
                };
                matches!(&children[0], BView::Label { text } if text == label)
                    .then_some(&children[..])
            })
            .unwrap_or_else(|| panic!("missing preference row {label}"))
    }

    fn row_named(label: &str) -> BPreferenceRow {
        SECTIONS
            .iter()
            .flat_map(BPreferenceSection::rows)
            .find(|row| row.metadata().label == label)
            .unwrap_or_else(|| panic!("missing declaration {label}"))
    }

    #[test]
    fn preferences_replace_canvas_with_bounded_declaration_panels() {
        let view = BPrefsItem::new().build(&BState::new());
        let BView::Panel {
            role: BPanelRole::Body,
            children,
        } = &view
        else {
            panic!("preferences must be a bounded body panel")
        };
        assert_eq!(children.len(), SECTIONS.len());
        for section in SECTIONS.iter() {
            assert_eq!(
                section_card(&view, section.title()).len(),
                section.len() + 1
            );
        }
    }

    #[test]
    fn preferences_show_effective_palette_default_and_valid_override() {
        let mut state = BState::new();
        state.set(&K_THEME, BPaletteName::TokyoNight.slug());
        let radius = row_named("Extra Small");
        state.set(radius.key(), "23");
        let view = BPrefsItem::new().build(&state);

        let colors = section_card(&view, "Colors");
        let background = row_for(colors, "Application Background");
        let BView::Row { children } = &background[1] else {
            panic!("preference controls must be a B-prefixed row")
        };
        let BView::Field { text } = &children[0] else {
            panic!("color control must display its effective value")
        };
        let color = row_named("Application Background");
        let expected = color
            .descriptor()
            .format_value(color.default_value(ui_style_defaults(BPaletteName::TokyoNight)))
            .expect("descriptor must format its default");
        assert_eq!(text, &expected);

        let radii = section_card(&view, "Radii");
        let xs = row_for(radii, "Extra Small");
        let BView::Row { children } = &xs[1] else {
            panic!()
        };
        assert!(matches!(&children[0], BView::Field { text } if text == "23 px"));
    }

    #[test]
    fn preference_rows_dispatch_canonical_edit_and_reset_actions() {
        let mut state = BState::new();
        let gradients = row_named("Gradients");
        state.set(gradients.key(), "true");
        let view = BPrefsItem::new().build(&state);
        let effects = section_card(&view, "Effects");
        let row = row_for(effects, "Gradients");
        let BView::Row { children } = &row[1] else {
            panic!()
        };
        assert!(matches!(
            &children[0],
            BView::Button(ctl)
                if matches!(&ctl.action, Some(BAction::Set(key, value)) if key == gradients.key() && value == "false")
        ));
        assert!(matches!(
            &children[1],
            BView::Button(ctl)
                if matches!(&ctl.action, Some(BAction::Reset(key)) if key == gradients.key())
        ));
    }
}
