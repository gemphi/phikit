//! Focused semantic-role catalog examples for requirements 4.1-4.7.
//!
//! Feature: phiber-resolved-ui-style-settings, Property 7
//! Validates: Requirements 4.2, 4.6, 4.7, 13.4.

use phiber::v2::theme::{
    BPaletteName, BRadiusScale, BRadiusToken, BResolvedUiStyle, BStyleRole, ui_style_defaults,
};
use proptest::prelude::*;

const SCALE_TOKENS: [BRadiusToken; 5] = [
    BRadiusToken::Xs,
    BRadiusToken::Sm,
    BRadiusToken::Md,
    BRadiusToken::Lg,
    BRadiusToken::Xl,
];

fn arb_radius_scale() -> impl Strategy<Value = BRadiusScale> {
    (
        0.0f32..100.0,
        0.125f32..20.0,
        0.125f32..20.0,
        0.125f32..20.0,
        0.125f32..20.0,
    )
        .prop_map(|(xs, sm_step, md_step, lg_step, xl_step)| {
            let sm = xs + sm_step;
            let md = sm + md_step;
            let lg = md + lg_step;
            let xl = lg + xl_step;
            BRadiusScale { xs, sm, md, lg, xl }
        })
}

fn with_changed_token(mut scale: BRadiusScale, token: BRadiusToken, value: f32) -> BRadiusScale {
    match token {
        BRadiusToken::Xs => scale.xs = value,
        BRadiusToken::Sm => scale.sm = value,
        BRadiusToken::Md => scale.md = value,
        BRadiusToken::Lg => scale.lg = value,
        BRadiusToken::Xl => scale.xl = value,
        BRadiusToken::None => unreachable!("None is not a mutable scale token"),
    }
    scale
}

fn representative_treatments_are_distinct(style: &BResolvedUiStyle) -> bool {
    let control = style.radius_for(BStyleRole::Button);
    let nested = style.radius_for(BStyleRole::Panel);
    let outer = style.radius_for(BStyleRole::Area);
    control != nested && control != outer && nested != outer
}

#[test]
fn catalog_defines_every_required_role() {
    assert_eq!(
        BStyleRole::ALL,
        [
            BStyleRole::Workspace,
            BStyleRole::Zone,
            BStyleRole::Area,
            BStyleRole::Host,
            BStyleRole::Region,
            BStyleRole::Panel,
            BStyleRole::Card,
            BStyleRole::Button,
            BStyleRole::Field,
            BStyleRole::MenuItem,
            BStyleRole::Tab,
            BStyleRole::Popover,
            BStyleRole::Modal,
        ]
    );
}

#[test]
fn catalog_maps_every_role_to_its_declared_treatment() {
    let roles = ui_style_defaults(BPaletteName::Sphere).roles;
    let expected = [
        (BStyleRole::Workspace, BRadiusToken::None),
        (BStyleRole::Zone, BRadiusToken::Lg),
        (BStyleRole::Area, BRadiusToken::Lg),
        (BStyleRole::Host, BRadiusToken::Lg),
        (BStyleRole::Region, BRadiusToken::Md),
        (BStyleRole::Panel, BRadiusToken::Md),
        (BStyleRole::Card, BRadiusToken::Lg),
        (BStyleRole::Button, BRadiusToken::Sm),
        (BStyleRole::Field, BRadiusToken::Sm),
        (BStyleRole::MenuItem, BRadiusToken::Sm),
        (BStyleRole::Tab, BRadiusToken::Sm),
        (BStyleRole::Popover, BRadiusToken::Md),
        (BStyleRole::Modal, BRadiusToken::Xl),
    ];
    for (role, token) in expected {
        assert_eq!(
            roles.token_for(role),
            token,
            "unexpected token for {role:?}"
        );
    }
}

#[test]
fn tokens_resolve_against_the_supplied_scale() {
    let scale = BRadiusScale {
        xs: 1.0,
        sm: 3.0,
        md: 5.0,
        lg: 7.0,
        xl: 9.0,
    };
    assert_eq!(scale.resolve(BRadiusToken::None), 0.0);
    assert_eq!(scale.resolve(BRadiusToken::Xs), 1.0);
    assert_eq!(scale.resolve(BRadiusToken::Sm), 3.0);
    assert_eq!(scale.resolve(BRadiusToken::Md), 5.0);
    assert_eq!(scale.resolve(BRadiusToken::Lg), 7.0);
    assert_eq!(scale.resolve(BRadiusToken::Xl), 9.0);
}

#[test]
fn defaults_preserve_distinct_semantic_treatments() {
    let style = BResolvedUiStyle::from(ui_style_defaults(BPaletteName::Sphere));
    assert_eq!(style.radius_for(BStyleRole::Button), style.radii.sm);
    assert_eq!(style.radius_for(BStyleRole::Panel), style.radii.md);
    assert_eq!(style.radius_for(BStyleRole::Area), style.radii.lg);
    assert_eq!(style.radius_for(BStyleRole::Modal), style.radii.xl);
    assert_eq!(style.radius_for(BStyleRole::Popover), style.radii.md);
}

#[test]
fn changed_scale_resolves_affected_roles_without_catalog_changes() {
    let mut style = BResolvedUiStyle::from(ui_style_defaults(BPaletteName::Sphere));
    // Panel sits on `md`, so it is the control for an `sm` change.
    let unchanged_panel = style.radius_for(BStyleRole::Panel);
    style.radii.sm = 23.0;
    // Chrome controls share one radius: buttons, fields, menu items and tabs.
    assert_eq!(style.radius_for(BStyleRole::Button), 23.0);
    assert_eq!(style.radius_for(BStyleRole::Field), 23.0);
    assert_eq!(style.radius_for(BStyleRole::MenuItem), 23.0);
    assert_eq!(style.radius_for(BStyleRole::Tab), 23.0);
    assert_eq!(style.radius_for(BStyleRole::Panel), unchanged_panel);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 7: Semantic role resolution is total and scale-sensitive.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 7
    /// **Validates: Requirements 4.2, 4.6, 4.7, 13.4**
    #[test]
    fn semantic_role_resolution_is_total_and_scale_sensitive(scale in arb_radius_scale()) {
        let mut style = BResolvedUiStyle::from(ui_style_defaults(BPaletteName::Sphere));
        style.radii = scale;

        for role in BStyleRole::ALL {
            let token = style.roles.token_for(role);
            let resolved = style.radius_for(role);
            prop_assert_eq!(
                resolved,
                scale.resolve(token),
                "{:?} did not resolve from {:?}",
                role,
                token,
            );
            prop_assert!(resolved.is_finite(), "{role:?} resolved to non-finite {resolved}");
            prop_assert!(resolved >= 0.0, "{role:?} resolved to negative {resolved}");
        }
        prop_assert!(
            representative_treatments_are_distinct(&style),
            "control, nested, and outer treatments must be distinct for {scale:?}",
        );

        for (index, changed_token) in SCALE_TOKENS.into_iter().enumerate() {
            let replacement = scale.xl + 100.0 + index as f32;
            let changed_scale = with_changed_token(scale, changed_token, replacement);
            let mut changed_style = style.clone();
            changed_style.radii = changed_scale;

            for role in BStyleRole::ALL {
                let before = style.radius_for(role);
                let after = changed_style.radius_for(role);
                let should_change = style.roles.token_for(role) == changed_token;
                prop_assert_eq!(
                    after != before,
                    should_change,
                    "changing {:?} had the wrong effect on {:?}: {} -> {}",
                    changed_token,
                    role,
                    before,
                    after,
                );
                prop_assert_eq!(
                    after,
                    changed_scale.resolve(style.roles.token_for(role)),
                    "{:?} stopped resolving through its declared token",
                    role,
                );
            }
            prop_assert!(
                representative_treatments_are_distinct(&changed_style),
                "representative treatments collided after changing {changed_token:?}",
            );
        }
    }
}
