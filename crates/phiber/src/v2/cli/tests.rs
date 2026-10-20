//! `BCli` against Blender's own argument grammar.

use super::*;

const SCENE: BFrameLimit = BFrameLimit { first: 10, last: 50 };

fn parse(line: &str) -> Result<BCli, BCliError> {
    BCli::parse(&line.split_whitespace().collect::<Vec<_>>())
}

#[test]
fn the_blender_invocation_parses_as_blender_takes_it() {
    let cli = parse("-b scene.blend -f 12 -o //render/ -F PNG").expect("parses");
    assert!(cli.background);
    assert_eq!(cli.file.as_deref(), Some("scene.blend"));
    assert_eq!(cli.frame.as_deref(), Some("12"));
    assert_eq!(cli.output.as_deref(), Some("//render/"));
    assert_eq!(cli.format.as_deref(), Some("PNG"));
    assert!(cli.renders());
}

#[test]
fn long_forms_are_the_same_flags() {
    let cli = parse("--background scene.blend --render-frame 3 --render-output /tmp/o --engine CYCLES").expect("parses");
    assert!(cli.background);
    assert_eq!(cli.frame.as_deref(), Some("3"));
    assert_eq!(cli.output.as_deref(), Some("/tmp/o"));
    assert_eq!(cli.engine_name(), "CYCLES");
}

#[test]
fn flags_may_come_in_any_order_and_the_first_bare_word_is_the_file() {
    let cli = parse("-f 2 -o /tmp/o scene.blend -b").expect("parses");
    assert_eq!(cli.file.as_deref(), Some("scene.blend"));
    assert_eq!(cli.frame.as_deref(), Some("2"));
    let second = parse("a.blend b.blend").expect("parses");
    assert_eq!(second.file.as_deref(), Some("a.blend"), "the first one wins, as Blender takes it");
}

#[test]
fn dash_a_is_playback_before_a_file_and_render_anim_after_one() {
    // Blender registers `-a` twice — `arg_handle_playback_mode` in the
    // pre-file pass and `--render-anim` in the post-file pass — so the
    // same flag means two different things either side of the `.blend`.
    let before = parse("-a /tmp/frames/").expect("parses");
    assert!(before.playback && !before.animation, "before a file: playback");
    let after = parse("-b scene.blend -a").expect("parses");
    assert!(after.animation && !after.playback, "after a file: render the animation");
    assert!(after.renders());
}

#[test]
fn a_flag_that_takes_a_word_says_so_when_it_is_missing() {
    assert_eq!(
        parse("-b scene.blend -f"),
        Err(BCliError::MissingValue("-f / --render-frame".into()))
    );
    assert_eq!(
        parse("-o").unwrap_err().message(),
        "Error: -o / --render-output requires one argument"
    );
}

#[test]
fn an_engine_or_format_off_the_list_is_refused_and_help_is_allowed() {
    assert_eq!(parse("-E NOPE"), Err(BCliError::Unknown("-E".into(), "NOPE".into())));
    assert_eq!(parse("-F NOPE"), Err(BCliError::Unknown("-F".into(), "NOPE".into())));
    assert_eq!(parse("-E phiray").expect("case folds").engine_name(), "PHIRAY");
    assert!(parse("-E help").is_ok() && parse("-F help").is_ok(), "`help` lists them instead");
}

#[test]
fn the_extension_flag_is_off_only_for_zero() {
    // `arg_handle_extension_set`: `(argv[1][0] == '0')` is the only off.
    assert_eq!(parse("-x 0").expect("parses").extension, Some(false));
    assert_eq!(parse("-x 1").expect("parses").extension, Some(true));
}

#[test]
fn threads_and_jump_must_be_numbers() {
    assert_eq!(parse("-t 8").expect("parses").threads, Some(8));
    assert_eq!(parse("-j 3").expect("parses").step, Some(3));
    assert_eq!(parse("-t x"), Err(BCliError::NotANumber("-t / --threads".into(), "x".into())));
}

#[test]
fn phiray_is_the_engine_unless_the_line_says_otherwise() {
    assert_eq!(parse("-b scene.blend -f 1").expect("parses").engine_name(), "PHIRAY");
    assert_eq!(set::BRenderQuality::ENGINE, "PHIRAY");
    assert_eq!(set::ENGINE[0], "PHIRAY", "and `-E help` lists it first");
}

#[test]
fn the_samples_asked_for_are_the_samples_taken() {
    assert_eq!(set::BRenderQuality::SAMPLE_FLOOR, 2048);
    // Silent: the default.
    assert_eq!(parse("-b s.blend -f 1").expect("parses").sample_count(), 2048);
    // Asked for less or more: honoured, as Blender's `--samples` is.
    assert_eq!(parse("-b s.blend -f 1 --samples 64").expect("parses").sample_count(), 64);
    assert_eq!(parse("-b s.blend -f 1 --samples 4096").expect("parses").sample_count(), 4096);
    // `--draft` changes nothing about the count any more.
    assert_eq!(parse("-b s.blend -f 1 --samples 64 --draft").expect("parses").sample_count(), 64);
    assert_eq!(parse("-b s.blend -f 1 --draft").expect("parses").sample_count(), 2048, "draft alone changes nothing");
    assert_eq!(parse("-b s.blend -f 1 --samples 0").expect("parses").sample_count(), 1, "never zero");
}

#[test]
fn dash_f_takes_the_whole_frame_grammar() {
    let cli = parse("-b s.blend -f 1..3,10,-1").expect("parses");
    assert_eq!(cli.frame_list(SCENE), Ok(vec![1, 2, 3, 10, 50]));
}

#[test]
fn dash_a_renders_the_scenes_range_and_s_e_j_shape_it() {
    assert_eq!(parse("-b s.blend -a").expect("p").frame_list(SCENE).unwrap().len(), 41, "10 ..= 50");
    assert_eq!(parse("-b s.blend -a -s 1 -e 4").expect("p").frame_list(SCENE), Ok(vec![1, 2, 3, 4]));
    assert_eq!(parse("-b s.blend -a -s 1 -e 7 -j 3").expect("p").frame_list(SCENE), Ok(vec![1, 4, 7]));
    // `-s` and `-e` take the relative forms too.
    assert_eq!(parse("-b s.blend -a -s +0 -e +2").expect("p").frame_list(SCENE), Ok(vec![10, 11, 12]));
}

#[test]
fn an_invocation_with_no_render_flag_renders_nothing() {
    let cli = parse("-b scene.blend").expect("parses");
    assert!(!cli.renders(), "a file alone opens the app");
    assert_eq!(cli.frame_list(SCENE), Ok(Vec::new()));
    assert!(parse("").expect("parses").file.is_none());
}

#[test]
fn a_bad_frame_expression_is_reported_against_dash_f() {
    let cli = parse("-b s.blend -f 1,,2").expect("the flag parses");
    assert_eq!(cli.frame_list(SCENE), Err(BCliError::Frame(BFrameError::IncorrectCommaUse)));
    assert_eq!(
        cli.frame_list(SCENE).unwrap_err().message(),
        "Error: incorrect comma use '-f'"
    );
}

#[test]
fn every_flag_in_the_table_is_reachable_by_both_its_spellings() {
    for spec in set::ARG {
        assert!(!spec.short.is_empty() || !spec.long.is_empty(), "a flag needs a spelling");
        assert!(!spec.help.is_empty(), "{spec:?} needs a help line");
        for name in [spec.short, spec.long].into_iter().filter(|n| !n.is_empty()) {
            assert!(set::spec_of(name).is_some(), "{name} does not resolve");
        }
    }
}
