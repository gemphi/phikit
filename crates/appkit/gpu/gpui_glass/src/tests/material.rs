use crate::*;

#[test]
fn the_table_names_every_style() {
    for style in [GlassStyle::Regular, GlassStyle::Clear] {
        assert!(GlassMaterial::of(style).blur_radius > 0.0, "{} is missing from data/style.tsv", style.name());
    }
}

#[test]
fn a_material_is_found_by_the_name_apple_gives_it() {
    let thin = GlassMaterial::named("thin").expect("thin is in data/style.tsv");
    let thick = GlassMaterial::named("thick").expect("thick is in data/style.tsv");
    assert!(thin.blur_radius < thick.blur_radius && thin.tint_amount < thick.tint_amount);
    assert!(GlassMaterial::named("no such glass").is_none());
}

#[test]
fn a_theme_gives_the_colours_and_the_style_keeps_its_shape() {
    let made = GlassMaterial::themed(GlassStyle::Regular, [0.1, 0.2, 0.3, 0.5], Some(0.4), [0.9; 3], [0.0; 3]);
    assert_eq!(made.tint, [0.1, 0.2, 0.3]);
    assert!((made.tint_amount - 0.2).abs() < 1e-6, "the theme's share times the tint's alpha");
    assert_eq!(made.blur_radius, GlassMaterial::of(GlassStyle::Regular).blur_radius);
}

#[test]
fn clear_glass_takes_a_smaller_part_of_the_tint() {
    let tint = [0.0, 0.0, 0.0, 1.0];
    let regular = GlassMaterial::themed(GlassStyle::Regular, tint, Some(0.4), [1.0; 3], [0.0; 3]);
    let clear = GlassMaterial::themed(GlassStyle::Clear, tint, Some(0.4), [1.0; 3], [0.0; 3]);
    assert!(clear.tint_amount < regular.tint_amount);
}
