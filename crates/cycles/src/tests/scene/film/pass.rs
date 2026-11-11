// SPDX-License-Identifier: Apache-2.0
use super::*;

/// The passes after the film worked them out, from `asked`.
fn worked_out(asked: Vec<Pass>, integrator: &Integrator, film: &mut Film, input: FilmPassInput) -> Vec<Pass> {
    let mut pass = asked;
    film.update_pass(&mut pass, integrator, &input);
    pass
}

fn plain(asked: Vec<Pass>) -> Vec<Pass> {
    worked_out(asked, &Integrator::new(), &mut Film::new(), FilmPassInput::default())
}

fn type_name(pass: &[Pass]) -> Vec<&'static str> {
    pass.iter().map(|pass| pass.pass_type.name().unwrap_or("?")).collect()
}

fn denoising() -> Integrator {
    let mut integrator = Integrator::new();
    integrator.node.set("use_denoise", Value::Boolean(true));
    integrator
}

#[test]
fn adaptive_sampling_adds_its_two_passes_and_wide_passes_come_first() {
    let pass = plain(vec![Film::default_pass()]);
    assert_eq!(type_name(&pass), ["PASS_COMBINED", "PASS_ADAPTIVE_AUX_BUFFER", "PASS_SAMPLE_COUNT"]);
    // The combined pass was asked for; the film added the other two.
    assert_eq!(pass.iter().map(|pass| pass.is_auto).collect::<Vec<bool>>(), [false, true, true]);
}

#[test]
fn working_the_passes_out_again_gives_the_same_passes() {
    let (integrator, mut film) = (Integrator::new(), Film::new());
    let once = worked_out(vec![Film::default_pass()], &integrator, &mut film, FilmPassInput::default());
    let twice = worked_out(once.clone(), &integrator, &mut film, FilmPassInput::default());
    assert_eq!(once, twice);
}

#[test]
fn with_no_pass_asked_for_there_is_still_the_combined_pass() {
    let mut integrator = Integrator::new();
    integrator.node.set("use_adaptive_sampling", Value::Boolean(false));
    let pass = worked_out(Vec::new(), &integrator, &mut Film::new(), FilmPassInput::default());
    assert_eq!(type_name(&pass), ["PASS_COMBINED"]);
    assert!(pass[0].is_auto);
}

#[test]
fn a_denoiser_adds_its_guiding_passes_and_a_denoised_twin() {
    let pass = worked_out(vec![Film::default_pass()], &denoising(), &mut Film::new(), FilmPassInput::default());
    let expect = ["PASS_COMBINED", "PASS_COMBINED", "PASS_ADAPTIVE_AUX_BUFFER", "PASS_DENOISING_ALBEDO", "PASS_DENOISING_NORMAL", "PASS_SAMPLE_COUNT"];
    assert_eq!(type_name(&pass), expect);
    assert_eq!((pass[0].mode, pass[1].mode), (PassMode::Noisy, PassMode::Denoised));
}

#[test]
fn a_pass_composed_from_others_brings_them_along() {
    let pass = plain(vec![Film::default_pass(), noisy("PASS_DIFFUSE")]);
    let expect = ["PASS_COMBINED", "PASS_ADAPTIVE_AUX_BUFFER", "PASS_DIFFUSE", "PASS_DIFFUSE_DIRECT", "PASS_DIFFUSE_INDIRECT", "PASS_DIFFUSE_COLOR", "PASS_SAMPLE_COUNT"];
    assert_eq!(type_name(&pass), expect);
}

#[test]
fn a_shadow_catcher_adds_its_passes_and_the_background_to_put_back() {
    let catcher = FilmPassInput { has_shadow_catcher: true, ..FilmPassInput::default() };
    let pass = worked_out(vec![Film::default_pass()], &Integrator::new(), &mut Film::new(), catcher);
    assert!(["PASS_SHADOW_CATCHER", "PASS_SHADOW_CATCHER_SAMPLE_COUNT", "PASS_SHADOW_CATCHER_MATTE"].iter().all(|name| type_name(&pass).contains(name)));
    assert!(!type_name(&pass).contains(&"PASS_BACKGROUND"));
    let mut film = Film::new();
    film.node.set("use_approximate_shadow_catcher", Value::Boolean(true));
    let approximate = worked_out(vec![Film::default_pass()], &Integrator::new(), &mut film, catcher);
    assert!(type_name(&approximate).contains(&"PASS_BACKGROUND"));
}

#[test]
fn volumes_add_their_guiding_passes_denoised_even_without_a_denoiser() {
    let pass = worked_out(vec![Film::default_pass()], &Integrator::new(), &mut Film::new(), FilmPassInput { has_volume: true, ..FilmPassInput::default() });
    let scatter = PassType::named("PASS_VOLUME_SCATTER").expect("in the table");
    let twin = pass.iter().find(|pass| pass.pass_type == scatter && pass.mode == PassMode::Denoised).expect("the denoised scatter pass");
    assert_eq!(twin.name, "Volume Scatter");
    assert!(type_name(&pass).contains(&"PASS_VOLUME_MAJORANT") && type_name(&pass).contains(&"PASS_VOLUME_MAJORANT_SAMPLE_COUNT"));
    assert_eq!(type_name(&pass).iter().filter(|name| **name == "PASS_SAMPLE_COUNT").count(), 1);
}

#[test]
fn baking_adds_its_passes_under_the_names_cycles_gives_them() {
    let baking = FilmPassInput { baking: true, bake_use_seed: true, ..FilmPassInput::default() };
    let pass = worked_out(vec![Film::default_pass()], &Integrator::new(), &mut Film::new(), baking);
    let name: Vec<&str> = pass.iter().map(|pass| pass.name.as_str()).filter(|name| !name.is_empty()).collect();
    assert_eq!(name.len(), 3);
    assert!(["BakePrimitive", "BakeDifferential", "BakeSeed"].iter().all(|label| name.contains(label)), "{name:?}");
}

#[test]
fn two_passes_of_one_type_are_one_unless_their_names_differ() {
    let merged = plain(vec![Film::default_pass(), named("PASS_COMBINED", "Beauty"), noisy("PASS_DEPTH"), noisy("PASS_DEPTH")]);
    assert_eq!(type_name(&merged), ["PASS_COMBINED", "PASS_ADAPTIVE_AUX_BUFFER", "PASS_DEPTH", "PASS_SAMPLE_COUNT"]);
    assert_eq!(merged[0].name, "Beauty");
    let apart = plain(vec![named("PASS_AOV_VALUE", "a"), named("PASS_AOV_VALUE", "b")]);
    assert_eq!(apart.iter().filter(|pass| pass.name == "a" || pass.name == "b").count(), 2);
}

#[test]
fn a_denoised_pass_asked_for_without_a_denoiser_is_a_noisy_one() {
    let pass = plain(vec![denoised("PASS_COMBINED")]);
    assert_eq!((type_name(&pass)[0], pass[0].mode), ("PASS_COMBINED", PassMode::Noisy));
}

#[test]
fn a_uv_pass_coming_or_going_is_reported_once() {
    let (integrator, mut film) = (Integrator::new(), Film::new());
    let mut pass = vec![Film::default_pass(), noisy("PASS_UV")];
    assert!(film.update_pass(&mut pass, &integrator, &FilmPassInput::default()).uv_pass);
    assert_eq!(film.update_pass(&mut pass, &integrator, &FilmPassInput::default()), FilmPassChange::default());
    pass.retain(|pass| pass.pass_type != PassType::named("PASS_UV").expect("in the table"));
    assert!(film.update_pass(&mut pass, &integrator, &FilmPassInput::default()).uv_pass);
}

#[test]
fn the_film_counts_as_changed_after_its_passes_were_worked_out() {
    let mut film = Film::new();
    film.node.clear_modified();
    film.update_pass(&mut vec![Film::default_pass()], &Integrator::new(), &FilmPassInput::default());
    assert!(film.node.is_modified());
}
