//! Preferences body — settings panels for the active section.
//! Blender ref: `space_userpref/space_userpref.cc` WINDOW region.

pub mod device;
pub mod quality;
pub mod rule;

use crate::v2::state::BKey as _;
use crate::v2::state::AppKey;
use crate::v2::host::BHostType;
use crate::v2::modal::setting::BSettingsCategory;
use crate::v2::modal::setting::CATEGORIES;
use crate::v2::modal::setting::category_for;
use crate::v2::part::props::pref::BPreferenceControls;
use crate::v2::part::props::pref::BPreferenceSection;
use crate::v2::part::props::pref::SECTIONS;
use crate::v2::state::BState;
use crate::v2::theme::BPaletteName;
use crate::v2::theme::BUiStyleDefaults;
use crate::v2::part::props::BPanelItem;
use crate::v2::state::val::V_FALSE;
use crate::v2::view::prim::BColumn;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::with_peers;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

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
        // The nav bar picks a category; the body shows that category's style
        // sections and nothing else. Stacking all eight at once was why the
        // sidebar selection appeared to do nothing.
        let category = active_category(state);
        let mut cards: Vec<BView> = Vec::new();
        // Themes leads with the palette itself: choosing one rewrites every
        // colour below it, so it belongs above the values it governs.
        if category.kind == BSettingsCategory::Appearance {
            cards.push(palette_card(state));
            cards.push(dimension_card(state));
        }
        if category.kind == BSettingsCategory::System {
            cards.push(device::BRenderDevices::view(state));
        }
        if category.kind == BSettingsCategory::Viewport {
            cards.push(quality::BViewportQuality::view(state));
        }
        let sections: Vec<BView> = SECTIONS
            .iter()
            .filter(|section| category.sections.contains(&section.title()))
            .map(|section| section_view(category.label, section, state, defaults))
            .collect();
        if !sections.is_empty() {
            cards.extend(with_peers(sections));
            return BView::from(BCard::column(BColumn::Body, cards));
        }
        if !cards.is_empty() {
            return BView::from(BCard::column(BColumn::Body, cards));
        }
        if sections.is_empty() {
            return BView::from(BCard::column(BColumn::Body, vec![BCard::panel(category.label)
                    .body(vec![BView::split(vec![BView::label("No preferences here yet.")],)])
                    .into()],));
        }
        BView::from(BCard::column(BColumn::Body, cards))
    }
}

/// The theme picker — a *Theme* section like the ones under it, folding
/// the same way, listing every theme as a row with its *Use* button and a
/// check on the one in force. Blender's is `USERPREF_PT_theme`'s preset
/// list above the theme sections. The heading sits on the rail's grid
/// beside Interface, the first theme's row beside Themes.
fn palette_card(state: &BState) -> BView {
    use crate::v2::view::prim::{BAction, BControl};

    let (active, mode) = crate::v2::theme::theme_in_force(state);
    let key = BPanelItem::expand_key(BHostType::Prefs, "Appearance", "Theme");
    let open = state.get(&key) != Some(V_FALSE);
    // The mode switch — Dark / Light, one setting over every theme — sits in
    // the section's heading, where Blender's panel headers hold their
    // controls (`draw_header_preset`).
    let switch: Vec<BView> = crate::v2::theme::BThemeMode::ALL
        .into_iter()
        .map(|choice| {
            BView::Button(
                BControl::new()
                    .with_label(choice.label())
                    .with_active(choice == mode)
                    .with_action(BAction::Set(AppKey::ThemeMode.path(), choice.slug().to_string())),
            )
        })
        .collect();
    if !open {
        return BCard::panel("Theme").preset(switch).fold(key, false).into();
    }
    let rows = BPaletteName::ALL.iter().map(|palette| {
        BView::split(vec![
            BView::label(palette.label()),
            BView::Button(
                BControl::new()
                    .with_icon(if *palette == active { "check" } else { "" })
                    .with_label(if *palette == active { "Active" } else { "Use" })
                    .with_active(*palette == active)
                    .with_action(BAction::Set(AppKey::Theme.path(), palette.slug().to_string())),
            ),
        ])
    });
    BCard::panel("Theme").preset(switch).fold(key, true).body(rows.collect()).into()
}

/// The dimensional coat — a *Dimension* section beside Theme, folding the
/// same way. One switch over the whole window: `Auto` keeps each theme's own
/// depth, `Flat` is Blender's flat chrome everywhere, `Spatial` is the
/// virtual-world look where surfaces sweep and lifted things cast. Blender has
/// no equivalent; this is phiber's own axis (`theme/style/flavour`).
fn dimension_card(state: &BState) -> BView {
    use crate::v2::theme::BFlavourScope;
    use crate::v2::view::prim::{BAction, BControl};

    let scope = state
        .get(AppKey::Flavour)
        .and_then(BFlavourScope::from_slug)
        .unwrap_or_default();
    let switch: Vec<BView> = BFlavourScope::ALL
        .into_iter()
        .map(|choice| {
            BView::Button(
                BControl::new()
                    .with_label(choice.label())
                    .with_active(choice == scope)
                    .with_action(BAction::Set(AppKey::Flavour.path(), choice.slug().to_string())),
            )
        })
        .collect();
    let key = BPanelItem::expand_key(BHostType::Prefs, "Appearance", "Dimension");
    let open = state.get(&key) != Some(V_FALSE);
    if !open {
        return BCard::panel("Dimension").preset(switch).fold(key, false).into();
    }
    let note = BView::split(vec![BView::label(
        "The coat the whole window wears — flat, or the spatial virtual-world look.",
    )]);
    BCard::panel("Dimension").preset(switch).fold(key, true).body(vec![note]).into()
}

/// The category the Preferences nav bar is showing.
fn active_category(state: &BState) -> &'static crate::v2::modal::setting::BSettingsCategorySpec {
    let label = state
        .host(BHostType::Prefs)
        .get("navbar.tab")
        .unwrap_or(CATEGORIES[0].label);
    category_for(label)
}

fn defaults_for(state: &BState) -> &'static BUiStyleDefaults {
    crate::v2::theme::ui_style_defaults_for(state)
}

/// One preference section: a panel that folds, as Blender's
/// `USERPREF_PT_*` panels do. Its fold key follows the Properties editor's,
/// `ctx.prefs.<category>.<section>.expanded`, open until closed.
fn section_view(
    category: &'static str,
    section: BPreferenceSection,
    state: &BState,
    defaults: &BUiStyleDefaults,
) -> BView {
    let key = BPanelItem::expand_key(BHostType::Prefs, category, section.title());
    let open = state.get(&key) != Some(V_FALSE);
    if !open {
        return BCard::panel(section.title()).fold(key, false).into();
    }
    let rows = section.rows().map(|row| {
        BView::split(vec![
                BView::label(row.metadata().label),
                BPreferenceControls::for_row(row, state, defaults).view(state),
            ],)
    });
    BCard::panel(section.title()).fold(key, true).body(rows.collect()).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::part::props::pref::BPreferenceRow;
    use crate::v2::state::BValue;
    use crate::v2::view::BAction;
    use crate::v2::view::prim::BCtrlKind;

    fn section_card<'a>(view: &'a BView, title: &str) -> &'a [BView] {
        let BView::Card(column) = view else {
            panic!("preferences must use a bounded body column")
        };
        assert_eq!(column.kind, crate::v2::view::prim::BCardKind::Column(BColumn::Body));
        column
            .body
            .iter()
            .find_map(|card| match card {
                BView::Card(card) if card.title() == Some(title) => Some(&card.body[..]),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing preference section {title}"))
    }

    fn row_for<'a>(card: &'a [BView], label: &str) -> &'a [BView] {
        card.iter()
            .find_map(|row| {
                let BView::Split { list } = row else {
                    return None;
                };
                matches!(&list[0], BView::Label { text } if text == label)
                    .then_some(&list[..])
            })
            .unwrap_or_else(|| panic!("missing preference row {label}"))
    }

    /// A state showing one settings category, the way the nav bar leaves it.
    fn showing(category: &str) -> BState {
        let mut state = BState::new();
        state.set(
            &crate::v2::state::BPath::owned(vec![
                BHostType::Prefs.slug(),
                "navbar",
                "tab",
            ]),
            category,
        );
        state
    }

    fn row_named(label: &str) -> BPreferenceRow {
        SECTIONS
            .iter()
            .flat_map(BPreferenceSection::rows)
            .find(|row| row.metadata().label == label)
            .unwrap_or_else(|| panic!("missing declaration {label}"))
    }

    #[test]
    fn a_category_shows_its_own_style_sections_and_no_others() {
        for category in CATEGORIES {
            let view = BPrefsItem::new().build(&showing(category.label));
            let BView::Card(column) = &view else {
                panic!("preferences must be a bounded body column")
            };
            let list = &column.body;
            if category.sections.is_empty() {
                // A category with nothing to show says so, rather than
                // falling back to every section in the app.
                assert_eq!(list.len(), 1, "{}", category.label);
                continue;
            }
            // Themes carries the palette picker and the Dimension switch above
            // the values they govern; Viewport its Quality panel (`quality.rs`).
            let extra = match category.kind {
                BSettingsCategory::Appearance => 2,
                BSettingsCategory::Viewport => 1,
                _ => 0,
            };
            assert_eq!(
                list.len(),
                category.sections.len() + extra,
                "{}",
                category.label
            );
            for section in SECTIONS.iter().filter(|s| category.sections.contains(&s.title())) {
                assert_eq!(
                    section_card(&view, section.title()).len(),
                    section.len()
                );
            }
        }
    }

    #[test]
    fn choosing_a_theme_is_offered_above_the_values_it_governs() {
        use crate::v2::theme::BPaletteName;
        let view = BPrefsItem::new().build(&showing("Appearance"));
        // Blender's theme picker (`USERPREF_PT_theme`) leads the theme
        // sections: a Theme section that folds like them, one row per theme
        // with its Use button, the one in force lit.
        let BView::Card(column) = &view else { panic!() };
        let BView::Card(first) = &column.body[0] else { panic!("the picker leads") };
        assert_eq!(first.title(), Some("Theme"));
        assert!(first.fold.is_some(), "the picker folds like the sections under it");
        assert_eq!(first.body.len(), BPaletteName::ALL.len(), "every theme is listed");
        let lit: Vec<&str> = first
            .body
            .iter()
            .filter_map(|row| {
                let BView::Split { list } = row else { return None };
                let BView::Label { text } = &list[0] else { return None };
                let BView::Button(ctl) = &list[1] else { return None };
                ctl.active.then_some(text.as_str())
            })
            .collect();
        assert_eq!(lit, vec![BPaletteName::DEFAULT.label()]);
        // Folded, it is the heading alone.
        let mut state = showing("Appearance");
        state.set(&BPanelItem::expand_key(BHostType::Prefs, "Appearance", "Theme"), V_FALSE);
        let view = BPrefsItem::new().build(&state);
        let BView::Card(column) = &view else { panic!() };
        let BView::Card(first) = &column.body[0] else { panic!() };
        assert!(first.body.is_empty() && first.title() == Some("Theme"));
    }

    #[test]
    fn every_style_section_belongs_to_some_category() {
        for section in SECTIONS.iter() {
            assert!(
                CATEGORIES.iter().any(|c| c.sections.contains(&section.title())),
                "no Preferences category shows the {} section",
                section.title()
            );
        }
    }

    #[test]
    fn preferences_show_effective_palette_default_and_valid_override() {
        let mut state = showing("Appearance");
        state.set(AppKey::Theme, BPaletteName::Tokyo.slug());
        let radius = row_named("Extra Small");
        state.set(radius.key(), "23");
        let view = BPrefsItem::new().build(&state);

        // A preference control is a `BCtrl` of the kind the preference is, not
        // a hand-built field: `BPreferenceControls::view` composes the same
        // control family the Properties host uses. What the row has to carry is
        // unchanged, and is the whole point of this test — the EFFECTIVE value,
        // which is the override where there is one and the palette's default
        // where there is not.
        let colors = section_card(&view, "Colors");
        let background = row_for(colors, "Application Background");
        let BView::Row { list } = &background[1] else {
            panic!("preference controls must be a B-prefixed row")
        };
        let BView::Ctrl(ctrl) = &list[0] else {
            panic!("a preference control is a `BCtrl`: {:?}", &list[0])
        };
        let color = row_named("Application Background");
        // The defaults the PANEL draws from: `ui_style_defaults_for(state)`, the
        // theme in force in the mode in force. `ui_style_defaults(name)` is the
        // DARK ladder whatever the mode (`theme/style/mod.rs` says so), and the
        // mode defaults to Light — so asking for it here compared a light
        // background against a dark one and called the difference a bug.
        let expected = color
            .descriptor()
            .format_value(color.default_value(crate::v2::theme::ui_style_defaults_for(&state)))
            .expect("descriptor must format its default");
        // In the descriptor's own spelling, which is what the swatch reads.
        assert!(
            matches!(&ctrl.value, BValue::Text(text) if text == &expected),
            "an un-overridden colour must carry its palette default, not a blank: {:?}",
            ctrl.value,
        );

        let radii = section_card(&view, "Radii");
        let xs = row_for(radii, "Extra Small");
        let BView::Row { list } = &xs[1] else {
            panic!()
        };
        let BView::Ctrl(ctrl) = &list[0] else {
            panic!("a preference control is a `BCtrl`: {:?}", &list[0])
        };
        assert!(
            matches!(&ctrl.value, BValue::Float(px) if (px - 23.0).abs() < 1e-6),
            "the stored override must win over the default: {:?}",
            ctrl.value,
        );
    }

    #[test]
    fn preference_rows_dispatch_canonical_edit_and_reset_actions() {
        let mut state = showing("Appearance");
        let gradients = row_named("Gradients");
        state.set(gradients.key(), "true");
        let view = BPrefsItem::new().build(&state);
        let effects = section_card(&view, "Effects");
        let row = row_for(effects, "Gradients");
        let BView::Row { list } = &row[1] else {
            panic!()
        };
        // The edit is the control's now, not a button built beside it: a
        // `BCtrlKind::Check` over the preference's own key, carrying its
        // effective state. The flip to `false` is what the checkbox writes when
        // it is on, and it is the checkbox's own rule rather than an action
        // baked into this row — which is the point of the control family being
        // shared with the Properties host.
        let BView::Ctrl(ctrl) = &list[0] else {
            panic!("a preference control is a `BCtrl`: {:?}", &list[0])
        };
        assert!(matches!(ctrl.kind, BCtrlKind::Check), "{:?}", ctrl.kind);
        assert_eq!(&ctrl.key, gradients.key(), "the control edits the preference's own key");
        assert!(matches!(&ctrl.value, BValue::Bool(true)), "{:?}", ctrl.value);
        // The reset is still a button of this row's own, and still canonical.
        assert!(matches!(
            &list[1],
            BView::Button(ctl)
                if matches!(&ctl.action, Some(BAction::Reset(key)) if key == gradients.key())
        ));
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BPrefsList;

impl BPrefsList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BPrefsList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BPrefsItem::new().build(s)]
    }
}
