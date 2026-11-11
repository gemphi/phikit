use super::*;

fn tally(keys: &[(&str, bool)]) -> BMissingTexture {
    let mut m = BMissingTexture::default();
    for (key, found) in keys {
        m.note(key, *found);
    }
    m
}

#[test]
fn a_file_with_every_image_says_nothing() {
    let m = tally(&[("a.png", true), ("b.png", true)]);
    assert!(m.is_empty() && m.lines().is_empty());
}

#[test]
fn the_count_leads_and_the_folders_are_ranked_by_what_they_owe() {
    // Pose 26's shape: many images, a few folders, one owing most.
    let mut keys: Vec<(String, bool)> = Vec::new();
    for i in 0..20 {
        keys.push((format!("//../../../Female-01-Texture-File/skin/map{i}.png"), false));
    }
    for i in 0..5 {
        keys.push((format!("E:/Graphigem/Outfits/Textures/cloth{i}.png"), false));
    }
    keys.push(("packed.png".into(), true));
    let m = tally(&keys.iter().map(|(k, f)| (k.as_str(), *f)).collect::<Vec<_>>());
    let lines = m.lines();
    assert!(lines[0].starts_with("25 of 26 images not found"), "{}", lines[0]);
    assert!(lines[0].contains("textures"), "it says what to do: {}", lines[0]);
    // The folder owing 20 comes before the one owing 5.
    assert!(lines[1].starts_with("  20 from") && lines[1].ends_with("Female-01-Texture-File/skin"), "{}", lines[1]);
    assert!(lines[2].starts_with("  5 from"), "{}", lines[2]);
    // Blender's own line survives as evidence, and the rest are counted.
    assert!(lines.iter().any(|l| l.starts_with("Cannot read '")), "{lines:?}");
    assert!(lines.iter().any(|l| l == "… and 19 more images"), "{lines:?}");
    // The whole report is a screenful, not 25 lines.
    assert!(lines.len() <= 3 + FOLDERS + EVIDENCE, "{} lines", lines.len());
}

#[test]
fn one_image_asked_for_twice_is_one_image_missing() {
    // `attach` walks material slots, so the same key arrives repeatedly.
    let m = tally(&[("a.png", false), ("a.png", false), ("b.png", false)]);
    assert_eq!(m.len(), 2);
    assert!(m.lines()[0].starts_with("2 of 3 images not found"));
}

#[test]
fn a_long_path_keeps_the_end_that_names_it_and_a_bare_name_says_beside_the_file() {
    assert_eq!(BMissingTexture::short("E:/a/b/c/d/e"), "…/c/d/e");
    assert_eq!(BMissingTexture::short("a/b"), "a/b");
    assert_eq!(BMissingTexture::short(""), "(beside the file)");
    // `//..\..` is Blender's relative prefix; the `..` parts carry no
    // information and are dropped rather than shown.
    assert_eq!(BMissingTexture::short("//../../Patterns"), "Patterns");
}
